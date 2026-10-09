// TC-040 · AC-WALLET-1 (format saldo) · aturan proyek: nilai Ⓝ selalu BigInt, tanpa float
import { describe, expect, it } from "vitest";

import { asYoctoNear, formatNear } from "@/lib/format/money";

const YOCTO = 10n ** 24n;

describe("asYoctoNear", () => {
  it("accepts a digits-only string", () => {
    expect(asYoctoNear("0")).toBe("0");
    expect(asYoctoNear("1000000000000000000000000")).toBe("1000000000000000000000000");
  });

  it("rejects anything that is not a non-negative integer string", () => {
    // Nilai yoctoNEAR tidak pernah berupa number/float di boundary (AGENTS.md).
    expect(() => asYoctoNear("1.5")).toThrowError(/Invalid yoctoNEAR amount/);
    expect(() => asYoctoNear("-1")).toThrowError(/Invalid yoctoNEAR amount/);
    expect(() => asYoctoNear("")).toThrowError(/Invalid yoctoNEAR amount/);
    expect(() => asYoctoNear("1e24")).toThrowError(/Invalid yoctoNEAR amount/);
  });
});

describe("formatNear", () => {
  it("formats whole amounts without a decimal part", () => {
    expect(formatNear(asYoctoNear("0"))).toBe("0");
    expect(formatNear(asYoctoNear(YOCTO.toString()))).toBe("1");
    expect(formatNear(asYoctoNear((5n * YOCTO).toString()))).toBe("5");
  });

  it("formats fractions using BigInt only — no float rounding", () => {
    expect(formatNear(asYoctoNear((YOCTO / 2n).toString()))).toBe("0.5");
    expect(formatNear(asYoctoNear("750000000000000000000000"))).toBe("0.75");
  });

  it("truncates (never rounds up) beyond maxFractionDigits", () => {
    // 0.1234567… Ⓝ → 4 digit, dipotong; pembulatan ke atas akan melebihkan saldo.
    const value = asYoctoNear("123456789012345678901234");
    expect(formatNear(value, 4)).toBe("0.1234");
    expect(formatNear(value, 2)).toBe("0.12");
  });

  it("trims trailing zeros in the fraction", () => {
    expect(formatNear(asYoctoNear("1500000000000000000000000"))).toBe("1.5");
    expect(formatNear(asYoctoNear("1000000000000000000000001"))).toBe("1");
  });

  it("handles amounts too small to show at the requested precision", () => {
    expect(formatNear(asYoctoNear("1"))).toBe("0");
    expect(formatNear(asYoctoNear("1"), 24)).toBe("0.000000000000000000000001");
  });

  it("returns the whole part when no fraction digits are requested", () => {
    expect(formatNear(asYoctoNear("1500000000000000000000000"), 0)).toBe("1");
  });
});
