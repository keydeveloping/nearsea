// TC-040 · AC: listing stale disembunyikan dari discovery, kegagalan RPC tidak mengosongkan grid
import { describe, expect, it } from "vitest";

import { asYoctoNear } from "@/lib/format/money";

import { buildFakeChain } from "../test-utils";

import { loadVisibleListings } from "./discovery";

import type { Sale } from "../types/marketplace.types";

const MARKET = "market.testnet";

function rawSale(tokenId: string, ownerId = "alice.testnet", approvalId: number | null = 7) {
  return {
    nft_contract_id: "nft.testnet",
    token_id: tokenId,
    owner_id: ownerId,
    approval_id: approvalId,
    price_yocto: "1000000000000000000000000",
    allowed_buyer: null,
    listed_at: 1_000,
  };
}

describe("loadVisibleListings", () => {
  it("keeps only public, still-valid listings", async () => {
    const chain = buildFakeChain();
    chain.viewResults.get_sales = [
      rawSale("1"),
      // privat → tidak di grid publik
      { ...rawSale("2"), allowed_buyer: "bob.testnet" },
    ];
    chain.viewResults.nft_token = {
      token_id: "1",
      owner_id: "alice.testnet",
      approved_account_ids: { [MARKET]: 7 },
    };

    const visible = await loadVisibleListings(chain, MARKET);

    expect(visible.map((sale: Sale) => sale.tokenId)).toEqual(["1"]);
  });

  it("hides a listing whose owner moved away", async () => {
    const chain = buildFakeChain();
    chain.viewResults.get_sales = [rawSale("1")];
    chain.viewResults.nft_token = {
      token_id: "1",
      owner_id: "bob.testnet",
      approved_account_ids: { [MARKET]: 7 },
    };

    expect(await loadVisibleListings(chain, MARKET)).toEqual([]);
  });

  it("hides a listing whose approval was revoked", async () => {
    const chain = buildFakeChain();
    chain.viewResults.get_sales = [rawSale("1")];
    chain.viewResults.nft_token = {
      token_id: "1",
      owner_id: "alice.testnet",
      approved_account_ids: {},
    };

    expect(await loadVisibleListings(chain, MARKET)).toEqual([]);
  });

  it("hides a listing whose token no longer exists", async () => {
    const chain = buildFakeChain();
    chain.viewResults.get_sales = [rawSale("1")];
    chain.viewResults.nft_token = null;

    expect(await loadVisibleListings(chain, MARKET)).toEqual([]);
  });

  it("keeps a listing when verification is inconclusive rather than emptying the grid", async () => {
    // RPC sakit ≠ listing basi: "tidak tahu" tidak boleh menyembunyikan apa pun.
    const chain = buildFakeChain();
    chain.viewResults.get_sales = [rawSale("1"), rawSale("3")];
    chain.viewErrors.nft_token = new Error("RPC timeout");

    const visible = await loadVisibleListings(chain, MARKET);
    expect(visible).toHaveLength(2);
    expect(visible[0].priceYocto).toBe(asYoctoNear("1000000000000000000000000"));
  });
});
