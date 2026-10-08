// TC-040 · AC-WALLET-1, AC-WALLET-2, AC-WALLET-4
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";

import Home from "@/app/page";
import { t } from "@/i18n";

import { AppHeader } from "./AppHeader";
import { WalletProvider } from "./WalletProvider";
import { useWallet } from "../hooks/useWallet";
import { nearDouble } from "../near-connect.double";

vi.mock("near-connect-hooks", async () => {
  const double = await import("../near-connect.double");
  return { NearProvider: double.NearProvider, useNearWallet: double.useNearWallet };
});

/** Konsumen test-only: memaparkan nilai konteks supaya logika turunan bisa diasersi. */
function WalletProbe() {
  const wallet = useWallet();

  return (
    <dl>
      <dd data-testid="status">{wallet.status}</dd>
      <dd data-testid="account">{wallet.accountId ?? "-"}</dd>
      <dd data-testid="balance">{wallet.balanceYocto ?? "-"}</dd>
      <dd data-testid="accountStatus">{wallet.accountStatus}</dd>
      <dd data-testid="txDisabled">{String(wallet.transactionsDisabled)}</dd>
      <dd data-testid="errorCode">{wallet.errorCode ?? "-"}</dd>
    </dl>
  );
}

/**
 * Halaman beranda kini halaman Explore marketplace (tiket 10), jadi ia butuh provider
 * server-state juga — bukan lagi halaman scaffold statis.
 */
function renderApp() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

  return render(
    <QueryClientProvider client={queryClient}>
      <WalletProvider>
        <WalletProbe />
        <AppHeader />
        <Home />
      </WalletProvider>
    </QueryClientProvider>,
  );
}

const read = (testId: string) => screen.getByTestId(testId).textContent;

/**
 * Header app. Query yang mencari `status`/`alert`/`Retry` dipersempit ke sini: halaman
 * Explore punya keadaan loading & error sendiri dengan peran ARIA yang sama, dan test ini
 * menguji kontrol wallet — bukan halaman marketplace.
 */
const header = () => within(screen.getByRole("banner"));

beforeEach(() => nearDouble.reset());

describe("wallet state derivation", () => {
  it("starts disconnected once the wallet layer is mounted", async () => {
    renderApp();

    await waitFor(() => expect(read("status")).toBe("disconnected"));
    expect(read("account")).toBe("-");
  });

  it("derives connected + ready account, and enables transactions", async () => {
    nearDouble.balance = 1500000000000000000000000n;
    renderApp();

    fireEvent.click(await screen.findByRole("button", { name: "Connect Wallet" }));

    await waitFor(() => expect(read("status")).toBe("connected"));
    expect(read("account")).toBe("alice.testnet");
    expect(read("balance")).toBe("1500000000000000000000000");
    expect(read("accountStatus")).toBe("ready");
    expect(read("txDisabled")).toBe("false");
    expect(screen.getByText("1.5 Ⓝ")).toBeDefined();
    // Alamat juga tampil di header, bukan hanya di probe.
    expect(screen.getAllByText("alice.testnet").length).toBeGreaterThan(1);
  });

  it("disables transactions when the account is not found on the configured network", async () => {
    // Sinyal mismatch: RPC jaringan ini tidak mengenal akun tersebut.
    nearDouble.probeError = new Error("Account does not exist while viewing");
    renderApp();

    fireEvent.click(await screen.findByRole("button", { name: "Connect Wallet" }));

    await waitFor(() => expect(read("accountStatus")).toBe("not_found"));
    expect(read("txDisabled")).toBe("true");
    expect(header().getByRole("status").textContent).toContain("Account not found on testnet.");
  });

  it("keeps transactions disabled but does not claim a mismatch when the probe fails otherwise", async () => {
    // RPC timeout ≠ jaringan salah. Tx tetap disabled (akun belum terbukti ada), tetapi
    // banner tidak boleh mengklaim mismatch.
    nearDouble.probeError = new Error("Timeout");
    renderApp();

    fireEvent.click(await screen.findByRole("button", { name: "Connect Wallet" }));

    await waitFor(() => expect(read("accountStatus")).toBe("unreachable"));
    expect(read("txDisabled")).toBe("true");
    expect(header().getByRole("status").textContent).toBe("Network: testnet");
  });

  it("reports the action that failed so Retry repeats it, not connect", async () => {
    nearDouble.signOutError = new Error("User rejected");
    renderApp();

    fireEvent.click(await screen.findByRole("button", { name: "Connect Wallet" }));
    await waitFor(() => expect(read("status")).toBe("connected"));

    fireEvent.click(screen.getByRole("button", { name: "Disconnect" }));

    await waitFor(() => expect(read("errorCode")).toBe("rejected"));
    // Disconnect gagal → user belum lepas, jadi status utama tetap connected.
    expect(read("status")).toBe("connected");
    expect(nearDouble.signIn).toHaveBeenCalledTimes(1);

    fireEvent.click(header().getByRole("button", { name: "Retry" }));

    await waitFor(() => expect(nearDouble.signOut).toHaveBeenCalledTimes(2));
    // Retry mengulang disconnect — bukan menambah pemanggilan connect.
    expect(nearDouble.signIn).toHaveBeenCalledTimes(1);
  });

  it("maps a rejected sign-in to the error state with a mapped code", async () => {
    nearDouble.signInError = new Error("User rejected the request");
    renderApp();

    fireEvent.click(await screen.findByRole("button", { name: "Connect Wallet" }));

    await waitFor(() => expect(read("errorCode")).toBe("rejected"));
    expect(read("status")).toBe("error");
    expect(header().getByRole("alert").textContent).toContain(
      "Connection cancelled in your wallet.",
    );
  });

  it("returns to disconnected after a successful sign-out", async () => {
    renderApp();

    fireEvent.click(await screen.findByRole("button", { name: "Connect Wallet" }));
    await waitFor(() => expect(read("status")).toBe("connected"));

    fireEvent.click(screen.getByRole("button", { name: "Disconnect" }));

    await waitFor(() => expect(read("status")).toBe("disconnected"));
    expect(read("account")).toBe("-");
    expect(screen.queryByText("alice.testnet")).toBeNull();
  });
});

describe("app without a wallet (read-only mode)", () => {
  it("renders the page content and a usable Connect button", async () => {
    renderApp();

    expect(screen.getByRole("heading", { name: t("marketplace.grid.title") })).toBeDefined();
    expect(screen.getByText(t("auth.readOnlyNotice"))).toBeDefined();

    const connect = await screen.findByRole("button", { name: "Connect Wallet" });
    await waitFor(() => expect((connect as HTMLButtonElement).disabled).toBe(false));
  });

  it("keeps the network indicator visible before any wallet exists", async () => {
    renderApp();

    expect(header().getByRole("status").textContent).toBe("Network: testnet");
  });
});
