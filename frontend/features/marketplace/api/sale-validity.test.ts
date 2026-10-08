// TC-040 · AC: listing stale disembunyikan dari discovery; halaman token tetap menampilkannya
import { describe, expect, it } from "vitest";

import { asYoctoNear } from "@/lib/format/money";

import { compareNewestFirst, isPublicListing, isSaleStale } from "./sale-validity";
import type { NftToken, Sale } from "../types/marketplace.types";

const MARKET = "market.testnet";

function sale(overrides: Partial<Sale> = {}): Sale {
  return {
    nftContractId: "nft.testnet",
    tokenId: "1",
    ownerId: "alice.testnet",
    approvalId: 7,
    priceYocto: asYoctoNear("1000000000000000000000000"),
    allowedBuyer: null,
    listedAt: 1_000,
    ...overrides,
  };
}

function token(overrides: Partial<NftToken> = {}): NftToken {
  return {
    tokenId: "1",
    ownerId: "alice.testnet",
    approvedAccountIds: { [MARKET]: 7 },
    title: null,
    description: null,
    media: null,
    ...overrides,
  };
}

describe("isSaleStale", () => {
  it("treats a matching owner and approval id as live", () => {
    expect(isSaleStale(sale(), token(), MARKET)).toBe(false);
  });

  it("detects ownership that moved after listing", () => {
    expect(isSaleStale(sale(), token({ ownerId: "bob.testnet" }), MARKET)).toBe(true);
  });

  it("detects a revoked approval", () => {
    expect(isSaleStale(sale(), token({ approvedAccountIds: {} }), MARKET)).toBe(true);
  });

  it("detects an approval re-issued under a different id", () => {
    // Approval lama invalid setelah transfer/revoke (NEP-178), jadi id yang berbeda = stale.
    expect(isSaleStale(sale(), token({ approvedAccountIds: { [MARKET]: 9 } }), MARKET)).toBe(true);
  });

  it("treats a missing token as stale", () => {
    expect(isSaleStale(sale(), null, MARKET)).toBe(true);
  });

  it("accepts any live approval when the listing recorded no approval id", () => {
    expect(
      isSaleStale(
        sale({ approvalId: null }),
        token({ approvedAccountIds: { [MARKET]: 3 } }),
        MARKET,
      ),
    ).toBe(false);
    expect(isSaleStale(sale({ approvalId: null }), token({ approvedAccountIds: {} }), MARKET)).toBe(
      true,
    );
  });
});

describe("isPublicListing", () => {
  it("hides private listings from the public grid", () => {
    expect(isPublicListing(sale())).toBe(true);
    expect(isPublicListing(sale({ allowedBuyer: "bob.testnet" }))).toBe(false);
  });
});

describe("compareNewestFirst", () => {
  it("orders by listed_at descending", () => {
    const older = sale({ tokenId: "a", listedAt: 1 });
    const newer = sale({ tokenId: "b", listedAt: 2 });
    expect([older, newer].sort(compareNewestFirst)).toEqual([newer, older]);
  });

  it("breaks ties on token_id ascending so the order is deterministic", () => {
    const a = sale({ tokenId: "a", listedAt: 5 });
    const b = sale({ tokenId: "b", listedAt: 5 });
    expect([b, a].sort(compareNewestFirst)).toEqual([a, b]);
  });
});
