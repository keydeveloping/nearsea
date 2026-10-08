// TC-040 · AC: semua angka Ⓝ diformat dari string yoctoNEAR (tanpa float)
import { describe, expect, it } from "vitest";

import { asYoctoNear } from "./money";
import { storageShortfall } from "./storage";

describe("storageShortfall", () => {
  it("returns the gap between the storage bound and the balance already held", () => {
    // 0.005 Ⓝ dibutuhkan, 0.002 Ⓝ tersedia → kurang 0.003 Ⓝ.
    expect(
      storageShortfall(
        asYoctoNear("5000000000000000000000"),
        asYoctoNear("2000000000000000000000"),
      ),
    ).toBe("3000000000000000000000");
  });

  it("asks for nothing when the balance already covers the bound", () => {
    expect(
      storageShortfall(
        asYoctoNear("5000000000000000000000"),
        asYoctoNear("5000000000000000000000"),
      ),
    ).toBe("0");
    expect(
      storageShortfall(
        asYoctoNear("5000000000000000000000"),
        asYoctoNear("9000000000000000000000"),
      ),
    ).toBe("0");
  });
});
