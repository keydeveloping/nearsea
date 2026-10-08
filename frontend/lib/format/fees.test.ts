// TC-040 · AC: breakdown fee + royalti tanpa aritmetika float
import { describe, expect, it } from "vitest";

import { asYoctoNear } from "@/lib/format/money";

import { bpsOf, computeSaleBreakdown, percentFromBps } from "./fees";

const ONE_NEAR = asYoctoNear("1000000000000000000000000");
const TEN_NEAR = asYoctoNear("10000000000000000000000000");

describe("bpsOf", () => {
  it("computes the platform fee exactly as the contract does", () => {
    // Contoh kerja docs/features/payments.md §Fee: 10 Ⓝ, fee 2% = 0.2 Ⓝ.
    expect(bpsOf(TEN_NEAR, 200)).toBe("200000000000000000000000");
  });

  it("rounds down so the displayed cut is never larger than the real one", () => {
    // 1 yocto × 2% = 0.02 yocto → 0, bukan 1.
    expect(bpsOf(asYoctoNear("1"), 200)).toBe("0");
  });

  it("rejects basis points outside 0..10000", () => {
    expect(() => bpsOf(ONE_NEAR, 10_001)).toThrowError(/Invalid basis points/);
    expect(() => bpsOf(ONE_NEAR, -1)).toThrowError(/Invalid basis points/);
    expect(() => bpsOf(ONE_NEAR, 1.5)).toThrowError(/Invalid basis points/);
  });
});

describe("computeSaleBreakdown", () => {
  it("splits 10 Ⓝ with 2% fee and 5% royalty exactly like the contract", () => {
    const breakdown = computeSaleBreakdown(TEN_NEAR, 200, 500);

    expect(breakdown).not.toBeNull();
    expect(breakdown?.fee).toBe("200000000000000000000000");
    expect(breakdown?.royalty).toBe("500000000000000000000000");
    // Residual = yang benar-benar diterima seller.
    expect(breakdown?.sellerProceeds).toBe("9300000000000000000000000");
  });

  it("keeps the total equal to the price (fee is deducted from it, not added)", () => {
    const breakdown = computeSaleBreakdown(TEN_NEAR, 200, 250);

    expect(breakdown).not.toBeNull();
    const sum =
      BigInt(breakdown!.fee) + BigInt(breakdown!.royalty) + BigInt(breakdown!.sellerProceeds);
    expect(sum).toBe(BigInt(TEN_NEAR));
    expect(breakdown?.price).toBe(TEN_NEAR);
  });

  it("returns null instead of showing a negative seller residual", () => {
    // Fee 50% + royalti 60% = 110% dari harga: kontrak menolak payout seperti ini.
    expect(computeSaleBreakdown(TEN_NEAR, 5_000, 6_000)).toBeNull();
  });

  it("returns null when the royalty rate is unknown, rather than assuming zero", () => {
    // Royalti nol akan melebihkan "seller receives"; lebih baik tidak menampilkan breakdown.
    expect(computeSaleBreakdown(TEN_NEAR, 200, null)).toBeNull();
  });

  it("returns null for basis points that are not integers", () => {
    expect(computeSaleBreakdown(TEN_NEAR, 2.5, 500)).toBeNull();
    expect(computeSaleBreakdown(TEN_NEAR, -200, 0)).toBeNull();
  });
});

describe("percentFromBps", () => {
  it("formats whole and fractional percentages without floats", () => {
    expect(percentFromBps(200)).toBe("2");
    expect(percentFromBps(250)).toBe("2.5");
    expect(percentFromBps(1_000)).toBe("10");
    expect(percentFromBps(0)).toBe("0");
  });
});
