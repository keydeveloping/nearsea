// TC-040 · AC: data kontrak dipetakan dengan aman (yocto tetap string, data menyimpang ditolak)
import { describe, expect, it } from "vitest";

import {
  parseNftToken,
  parseRoyaltyConfig,
  parseSale,
  parseSaleList,
  parseStorageAvailable,
  parseStorageBounds,
} from "./market-api";

const RAW_SALE = {
  nft_contract_id: "nft.testnet",
  token_id: "42",
  owner_id: "alice.testnet",
  approval_id: 7,
  price_yocto: "1000000000000000000000000",
  allowed_buyer: null,
  listed_at: 1799016000000000000,
};

describe("parseSale", () => {
  it("maps contract snake_case into the camelCase shape, keeping yocto as a string", () => {
    expect(parseSale(RAW_SALE)).toEqual({
      nftContractId: "nft.testnet",
      tokenId: "42",
      ownerId: "alice.testnet",
      approvalId: 7,
      priceYocto: "1000000000000000000000000",
      allowedBuyer: null,
      listedAt: 1799016000000000000,
    });
  });

  it("keeps a private listing's allowed buyer", () => {
    expect(parseSale({ ...RAW_SALE, allowed_buyer: "bob.testnet" }).allowedBuyer).toBe(
      "bob.testnet",
    );
  });

  it("accepts a null approval id", () => {
    expect(parseSale({ ...RAW_SALE, approval_id: null }).approvalId).toBeNull();
  });

  it("rejects a price that is a number instead of a yocto string", () => {
    // Kontrak mengirim U128 sebagai string; angka = kontrak/ABI tak terduga, jangan ditebak.
    expect(() => parseSale({ ...RAW_SALE, price_yocto: 1000 })).toThrowError(
      /Unexpected sale.price_yocto/,
    );
  });

  it("rejects a price that is not a valid yocto amount", () => {
    expect(() => parseSale({ ...RAW_SALE, price_yocto: "1.5" })).toThrowError(
      /Unexpected sale.price_yocto/,
    );
  });
});

describe("parseSaleList", () => {
  it("rejects a non-array payload", () => {
    expect(() => parseSaleList({})).toThrowError(/Unexpected sales list/);
  });

  it("maps every entry", () => {
    expect(parseSaleList([RAW_SALE, { ...RAW_SALE, token_id: "43" }])).toHaveLength(2);
  });
});

describe("parseNftToken", () => {
  it("returns null when the token does not exist", () => {
    expect(parseNftToken(null)).toBeNull();
  });

  it("reads the approval map and flattened metadata", () => {
    const token = parseNftToken({
      token_id: "42",
      owner_id: "alice.testnet",
      approved_account_ids: { "market.testnet": 7 },
      title: "#42",
      description: null,
      media: null,
    });

    expect(token?.approvedAccountIds).toEqual({ "market.testnet": 7 });
    expect(token?.title).toBe("#42");
  });

  it("tolerates a token without an approvals map", () => {
    const token = parseNftToken({ token_id: "42", owner_id: "alice.testnet" });
    expect(token?.approvedAccountIds).toEqual({});
  });
});

describe("parseRoyaltyConfig", () => {
  it("reads receiver and bps", () => {
    expect(parseRoyaltyConfig({ receiver: "creator.testnet", bps: 500 })).toEqual({
      receiverId: "creator.testnet",
      bps: 500,
    });
  });

  it("returns null when the collection has no royalty config", () => {
    expect(parseRoyaltyConfig(null)).toBeNull();
  });
});

describe("storage views", () => {
  it("reads the storage bounds", () => {
    expect(parseStorageBounds({ min: "5000000000000000000000", max: null })).toEqual({
      minYocto: "5000000000000000000000",
      maxYocto: null,
    });
  });

  it("treats an unregistered account as zero available storage", () => {
    expect(parseStorageAvailable(null)).toBe("0");
    expect(parseStorageAvailable({ total: "1", available: "1" })).toBe("1");
  });
});
