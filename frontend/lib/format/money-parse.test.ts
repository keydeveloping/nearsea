// TC-040 · AC: harga Ⓝ dari input user tanpa aritmetika float
import { describe, expect, it } from "vitest";

import { parseNearInput } from "./money";

describe("parseNearInput", () => {
  it("converts a decimal Ⓝ amount to yoctoNEAR exactly", () => {
    expect(parseNearInput("1")).toBe("1000000000000000000000000");
    expect(parseNearInput("0.01")).toBe("10000000000000000000000");
    expect(parseNearInput("1.5")).toBe("1500000000000000000000000");
  });

  it("trims surrounding whitespace", () => {
    expect(parseNearInput("  0.01  ")).toBe("10000000000000000000000");
  });

  it("rejects anything that is not a plain decimal amount", () => {
    expect(parseNearInput("")).toBeNull();
    expect(parseNearInput("abc")).toBeNull();
    expect(parseNearInput("-1")).toBeNull();
    expect(parseNearInput("1e24")).toBeNull();
    expect(parseNearInput("1.2.3")).toBeNull();
    expect(parseNearInput(".5")).toBeNull();
  });

  it("rejects more precision than yoctoNEAR can represent", () => {
    // 24 desimal adalah batas; 25 tidak bisa direpresentasikan.
    expect(parseNearInput("0.0000000000000000000000001")).toBeNull();
  });
});
