// TC-040 · jalur emas M1: browse → list → buy (komponen + hook, tanpa wallet nyata)
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ListingGrid } from "./components/ListingGrid";
import { buildFakeChain, renderWithMarket } from "./test-utils";
import { TokenPage } from "./TokenPage";

const MARKET = "market.testnet";
const NFT = "nft.testnet";

function rawSale(tokenId: string, ownerId = "alice.testnet", price = "1000000000000000000000000") {
  return {
    nft_contract_id: NFT,
    token_id: tokenId,
    owner_id: ownerId,
    approval_id: 7,
    price_yocto: price,
    allowed_buyer: null,
    listed_at: 1_000,
  };
}

function rawToken(tokenId: string, ownerId: string, approvalId: number | null = 7) {
  return {
    token_id: tokenId,
    owner_id: ownerId,
    approved_account_ids: approvalId === null ? {} : { [MARKET]: approvalId },
    title: `#${tokenId}`,
    description: null,
    media: null,
  };
}

/** View call yang dibutuhkan modal listing supaya tidak gagal di jalur normal. */
function primeListViews(chain: ReturnType<typeof buildFakeChain>) {
  chain.viewResults.storage_balance_bounds = { min: "5000000000000000000000", max: null };
  chain.viewResults.storage_balance_of = { total: "0", available: "0" };
  chain.viewResults.nft_token = rawToken("42", "alice.testnet");
}

/** Promise yang diselesaikan test secara manual — membuat state antara bisa diamati. */
function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

describe("browse (grid)", () => {
  it("renders listings read from the contract, newest first", async () => {
    const chain = buildFakeChain();
    chain.viewResults.get_sales = [
      { ...rawSale("1"), listed_at: 1_000 },
      { ...rawSale("2"), listed_at: 2_000 },
    ];
    chain.viewResults.nft_token = rawToken("1", "alice.testnet");

    renderWithMarket(<ListingGrid />, chain);

    // Grid membaca dari view call kontrak, bukan data statis.
    expect(chain.views.some((view) => view.method === "get_sales")).toBe(true);
    const links = await screen.findAllByRole("link");
    expect(links[0].getAttribute("href")).toBe(`/token/${NFT}/2`);
  });

  it("hides a stale listing from discovery", async () => {
    const chain = buildFakeChain();
    chain.viewResults.get_sales = [rawSale("1")];
    // Kepemilikan sudah pindah → listing basi.
    chain.viewResults.nft_token = rawToken("1", "bob.testnet");

    renderWithMarket(<ListingGrid />, chain);

    await waitFor(() => expect(screen.getByText("No items found")).toBeDefined());
    expect(screen.queryByText(/#1/)).toBeNull();
  });

  it("shows a retryable error instead of an empty grid when the read fails", async () => {
    const chain = buildFakeChain();
    chain.viewError = new Error("RPC down");

    renderWithMarket(<ListingGrid />, chain);

    expect(await screen.findByRole("alert")).toBeDefined();
    expect(screen.getByRole("button", { name: "Retry" })).toBeDefined();
  });
});

describe("token page — owner vs non-owner", () => {
  it("shows Sell to the owner", async () => {
    const chain = buildFakeChain({ accountId: "alice.testnet" });
    chain.viewResults.get_sale = null;
    chain.viewResults.nft_token = rawToken("42", "alice.testnet");
    primeListViews(chain);

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    expect(await screen.findByRole("button", { name: "Sell" })).toBeDefined();
    expect(screen.queryByRole("button", { name: "Buy now" })).toBeNull();
  });

  it("shows Buy to a non-owner, and never a Sell button", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet" });
    chain.viewResults.get_sale = rawSale("42");
    chain.viewResults.nft_token = rawToken("42", "alice.testnet");

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    expect(await screen.findByRole("button", { name: "Buy now" })).toBeDefined();
    expect(screen.queryByRole("button", { name: "Sell" })).toBeNull();
  });

  it("shows 'no longer available' on a stale listing instead of a Buy button", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet" });
    chain.viewResults.get_sale = rawSale("42");
    chain.viewResults.nft_token = rawToken("42", "carol.testnet");

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    expect((await screen.findAllByText("This listing is no longer valid")).length).toBeGreaterThan(
      0,
    );
    expect(screen.queryByRole("button", { name: "Buy now" })).toBeNull();
  });

  it("renders untrusted metadata as text, never as HTML", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet" });
    chain.viewResults.get_sale = null;
    chain.viewResults.nft_token = {
      ...rawToken("42", "alice.testnet"),
      title: "<img src=x onerror=alert(1)>",
    };

    const { container } = renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    // Judul muncul sebagai teks apa adanya; tidak ada elemen img yang dibuat dari metadata.
    expect(await screen.findByText("<img src=x onerror=alert(1)>")).toBeDefined();
    expect(container.querySelector("img")).toBeNull();
  });

  it("renders https media through an img element", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet" });
    chain.viewResults.get_sale = null;
    chain.viewResults.nft_token = {
      ...rawToken("42", "alice.testnet"),
      media: "https://ipfs.io/ipfs/bafy123",
    };

    const { container } = renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    const image = await waitFor(() => {
      const found = container.querySelector("img");
      expect(found).not.toBeNull();
      return found as HTMLImageElement;
    });
    expect(image.getAttribute("src")).toBe("https://ipfs.io/ipfs/bafy123");
  });

  it("refuses media whose URL uses a non-https scheme", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet" });
    chain.viewResults.get_sale = null;
    chain.viewResults.nft_token = {
      ...rawToken("42", "alice.testnet"),
      media: "javascript:alert(1)",
    };

    const { container } = renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    await waitFor(() => expect(container.querySelector("h1")?.textContent).toBe("#42"));
    expect(container.querySelector("img")).toBeNull();
  });

  it("disables the action while the account is not proven on this network", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet", transactionsDisabled: true });
    chain.viewResults.get_sale = rawSale("42");
    chain.viewResults.nft_token = rawToken("42", "alice.testnet");

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    const buy = await screen.findByRole("button", { name: "Buy now" });
    expect((buy as HTMLButtonElement).disabled).toBe(true);
  });
});

describe("list flow — two signing steps", () => {
  it("approves then lists, showing progress 1/2 and 2/2, and passes the concrete approval id", async () => {
    const chain = buildFakeChain({ accountId: "alice.testnet" });
    chain.viewResults.get_sale = null;
    primeListViews(chain);

    // Tahan transaksi approve supaya state "langkah 1/2" benar-benar teramati, bukan
    // bergantung pada kecepatan microtask. Perekaman panggilan tetap terjadi lebih dulu.
    const approveGate = deferred();
    const originalCall = chain.callFunction;
    chain.callFunction = vi.fn(async (params) => {
      const pending = originalCall(params);
      if (params.method === "nft_approve") await approveGate.promise;
      return pending;
    });

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    fireEvent.click(await screen.findByRole("button", { name: "Sell" }));
    fireEvent.change(screen.getByLabelText(/Price/), { target: { value: "1" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    // Langkah 1/2 terlihat selama approve berjalan, dan tombol terkunci.
    expect(await screen.findByText("Step 1 of 2: Approve the marketplace")).toBeDefined();
    expect((screen.getByRole("button", { name: "Continue" }) as HTMLButtonElement).disabled).toBe(
      true,
    );

    // Hanya approve yang terkirim sejauh ini.
    expect(chain.calls.map((call) => call.method)).toEqual(["nft_approve"]);

    approveGate.resolve();

    await waitFor(() => expect(screen.getByText("Your item is listed")).toBeDefined());

    // Dua transaksi, urutannya approve → list.
    expect(chain.calls.map((call) => call.method)).toEqual(["nft_approve", "list_nft_for_sale"]);

    const approve = chain.calls[0];
    expect(approve.contractId).toBe(NFT);
    expect(approve.args).toMatchObject({ token_id: "42", account_id: MARKET });
    expect(approve.deposit).toBe("1");

    const list = chain.calls[1];
    expect(list.contractId).toBe(MARKET);
    // approval_id konkret (bukan null) — tanpa ini listing mustahil dibeli.
    expect(list.args).toMatchObject({
      nft_contract_id: NFT,
      token_id: "42",
      approval_id: 7,
      price: "1000000000000000000000000",
      allowed_buyer: null,
    });
    // Deposit = storage NEP-145, bukan 1 yocto.
    expect(list.deposit).toBe("5000000000000000000000");
  });

  it("shows step 2/2 while the listing transaction is being signed", async () => {
    const chain = buildFakeChain({ accountId: "alice.testnet" });
    chain.viewResults.get_sale = null;
    primeListViews(chain);

    const listGate = deferred();
    const originalCall = chain.callFunction;
    chain.callFunction = vi.fn(async (params) => {
      const pending = originalCall(params);
      if (params.method === "list_nft_for_sale") await listGate.promise;
      return pending;
    });

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    fireEvent.click(await screen.findByRole("button", { name: "Sell" }));
    fireEvent.change(screen.getByLabelText(/Price/), { target: { value: "1" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("Step 2 of 2: Confirm the listing")).toBeDefined();
    expect(screen.queryByText("Your item is listed")).toBeNull();

    listGate.resolve();
    await waitFor(() => expect(screen.getByText("Your item is listed")).toBeDefined());
  });

  it("rejects a price below the 0.01 Ⓝ minimum before signing anything", async () => {
    const chain = buildFakeChain({ accountId: "alice.testnet" });
    chain.viewResults.get_sale = null;
    primeListViews(chain);

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    fireEvent.click(await screen.findByRole("button", { name: "Sell" }));
    fireEvent.change(screen.getByLabelText(/Price/), { target: { value: "0.001" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("Enter a valid amount")).toBeDefined();
    expect(chain.calls).toHaveLength(0);
  });

  it("surfaces a rejected signature and stays open for a retry", async () => {
    const chain = buildFakeChain({ accountId: "alice.testnet" });
    chain.viewResults.get_sale = null;
    primeListViews(chain);
    chain.callError = new Error("User rejected the request");

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    fireEvent.click(await screen.findByRole("button", { name: "Sell" }));
    fireEvent.change(screen.getByLabelText(/Price/), { target: { value: "1" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByRole("alert")).toBeDefined();
    expect(screen.getByText("Transaction cancelled in your wallet.")).toBeDefined();
    expect(screen.queryByText("Your item is listed")).toBeNull();
  });

  it("stops before the second transaction when the approval is not visible on chain", async () => {
    const chain = buildFakeChain({ accountId: "alice.testnet" });
    chain.viewResults.get_sale = null;
    primeListViews(chain);
    // Approve sukses, tapi state koleksi tidak menunjukkan approval → listing akan rusak.
    chain.viewResults.nft_token = rawToken("42", "alice.testnet", null);

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    fireEvent.click(await screen.findByRole("button", { name: "Sell" }));
    fireEvent.change(screen.getByLabelText(/Price/), { target: { value: "1" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => expect(screen.getByRole("alert")).toBeDefined());
    expect(chain.calls.map((call) => call.method)).toEqual(["nft_approve"]);
  });
});

describe("buy flow — breakdown before confirmation", () => {
  it("shows the platform fee and creator royalty before the user confirms", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet" });
    chain.viewResults.get_sale = rawSale("42");
    chain.viewResults.nft_token = rawToken("42", "alice.testnet");
    chain.viewResults.get_fee_bps = 200;
    chain.viewResults.royalty_config = { receiver: "creator.testnet", bps: 500 };

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    fireEvent.click(await screen.findByRole("button", { name: "Buy now" }));

    // 1 Ⓝ dengan fee 2% dan royalti 5%.
    expect(await screen.findByText("Platform fee (2%)")).toBeDefined();
    expect(screen.getByText("Creator royalty")).toBeDefined();
    expect(screen.getByText("Seller receives")).toBeDefined();
    expect(screen.getByText("0.02 Ⓝ")).toBeDefined();
    expect(screen.getByText("0.05 Ⓝ")).toBeDefined();
    expect(screen.getByText("0.93 Ⓝ")).toBeDefined();
  });

  it("sends deposit equal to the price and refreshes data after the final receipt", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet" });
    chain.viewResults.get_sale = rawSale("42");
    chain.viewResults.nft_token = rawToken("42", "alice.testnet");
    chain.viewResults.get_fee_bps = 200;
    chain.viewResults.royalty_config = { receiver: "creator.testnet", bps: 500 };

    const { queryClient } = renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    fireEvent.click(await screen.findByRole("button", { name: "Buy now" }));
    await screen.findByText("Platform fee (2%)");

    const invalidate = vi.spyOn(queryClient, "invalidateQueries");
    fireEvent.click(screen.getByRole("button", { name: "Confirm and pay" }));

    await waitFor(() => expect(screen.getByText("Purchase complete")).toBeDefined());

    expect(chain.calls).toHaveLength(1);
    expect(chain.calls[0].method).toBe("buy");
    expect(chain.calls[0].contractId).toBe(MARKET);
    expect(chain.calls[0].args).toEqual({ nft_contract_id: NFT, token_id: "42" });
    expect(chain.calls[0].deposit).toBe("1000000000000000000000000");

    // Data di-refresh setelah receipt final, bukan saat submit.
    expect(invalidate).toHaveBeenCalled();
  });

  it("refuses to sign when the price changed since the modal opened", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet" });
    chain.viewResults.get_sale = rawSale("42");
    chain.viewResults.nft_token = rawToken("42", "alice.testnet");
    chain.viewResults.get_fee_bps = 200;
    chain.viewResults.royalty_config = { receiver: "creator.testnet", bps: 500 };

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    fireEvent.click(await screen.findByRole("button", { name: "Buy now" }));
    await screen.findByText("Platform fee (2%)");

    // Harga berubah di antara membuka modal dan menekan konfirmasi.
    chain.viewResults.get_sale = rawSale("42", "alice.testnet", "2000000000000000000000000");
    fireEvent.click(screen.getByRole("button", { name: "Confirm and pay" }));

    expect(await screen.findByText("The price changed — review and confirm again")).toBeDefined();
    expect(chain.calls).toHaveLength(0);
  });

  it("refuses to sign when the listing went stale since the modal opened", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet" });
    chain.viewResults.get_sale = rawSale("42");
    chain.viewResults.nft_token = rawToken("42", "alice.testnet");
    chain.viewResults.get_fee_bps = 200;
    chain.viewResults.royalty_config = { receiver: "creator.testnet", bps: 500 };

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    fireEvent.click(await screen.findByRole("button", { name: "Buy now" }));
    await screen.findByText("Platform fee (2%)");

    // Seller memindahkan token sebelum pembeli menekan konfirmasi.
    chain.viewResults.nft_token = rawToken("42", "carol.testnet");
    fireEvent.click(screen.getByRole("button", { name: "Confirm and pay" }));

    expect(await screen.findByText("This listing is no longer valid")).toBeDefined();
    expect(chain.calls).toHaveLength(0);
  });

  it("maps a sold-out listing to the catalog message, not a raw panic string", async () => {
    const chain = buildFakeChain({ accountId: "bob.testnet" });
    chain.viewResults.get_sale = rawSale("42");
    chain.viewResults.nft_token = rawToken("42", "alice.testnet");
    chain.viewResults.get_fee_bps = 200;
    chain.viewResults.royalty_config = { receiver: "creator.testnet", bps: 500 };
    chain.callError = new Error("Smart contract panicked: CONFLICT_SOLD");

    renderWithMarket(<TokenPage nftContractId={NFT} tokenId="42" />, chain);

    fireEvent.click(await screen.findByRole("button", { name: "Buy now" }));
    await screen.findByText("Platform fee (2%)");
    fireEvent.click(screen.getByRole("button", { name: "Confirm and pay" }));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Already sold");
    expect(alert.textContent).not.toContain("CONFLICT_SOLD");
  });
});
