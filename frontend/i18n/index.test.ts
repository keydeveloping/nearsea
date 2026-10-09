// TC-040 · aturan proyek: semua copy UI lewat i18n (code-standards.md §10)
import { describe, expect, it } from "vitest";

import { format, t } from "@/i18n";

import type { MessageKey } from "@/i18n";

describe("i18n", () => {
  it("resolves a namespaced key to its English copy", () => {
    expect(t("common.appName")).toBe("NearSea");
  });

  it("resolves a nested key by walking every segment", () => {
    expect(t("auth.error.rejected")).toBe("Connection cancelled in your wallet.");
  });

  it("rejects an unknown key instead of rendering a blank string", () => {
    // Cast disengaja: menguji guard runtime untuk kunci yang tidak ada.
    const unknownKey = "common.missingKey" as MessageKey;

    expect(() => t(unknownKey)).toThrowError("Missing i18n message: common.missingKey");
  });

  it("rejects a key whose path stops at a namespace object", () => {
    // Cast disengaja: menguji guard runtime untuk kunci yang menunjuk objek, bukan string.
    const branchKey = "auth.error" as MessageKey;

    expect(() => t(branchKey)).toThrowError("Missing i18n message: auth.error");
  });

  it("substitutes named placeholders", () => {
    expect(format(t("common.networkBanner"), { network: "testnet" })).toBe("Network: testnet");
  });
});
