import { describe, expect, it } from "vitest";

import { format, t } from "@/i18n";

describe("i18n", () => {
  it("resolves a namespaced key to its English copy", () => {
    expect(t("common.appName")).toBe("NearSea");
  });

  it("rejects an unknown key instead of rendering a blank string", () => {
    expect(() => t("common.missingKey" as never)).toThrowError(
      "Missing i18n message: common.missingKey",
    );
  });

  it("substitutes named placeholders", () => {
    expect(format(t("common.networkBanner"), { network: "testnet" })).toBe("Network: testnet");
  });
});
