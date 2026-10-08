// TC-040 · AC: metadata dari luar tidak boleh menyuntikkan apa pun ke halaman
import { describe, expect, it } from "vitest";

import { safeMediaUrl } from "./media";

describe("safeMediaUrl", () => {
  it("accepts an https media URL", () => {
    expect(safeMediaUrl("https://ipfs.io/ipfs/bafy123")).toBe("https://ipfs.io/ipfs/bafy123");
  });

  it("returns null for a missing media field", () => {
    expect(safeMediaUrl(null)).toBeNull();
  });

  it("rejects non-https schemes so metadata cannot smuggle in a script or inline payload", () => {
    expect(safeMediaUrl("javascript:alert(1)")).toBeNull();
    expect(safeMediaUrl("data:image/svg+xml,<svg onload=alert(1)>")).toBeNull();
    expect(safeMediaUrl("http://ipfs.io/ipfs/bafy123")).toBeNull();
    expect(safeMediaUrl("blob:https://example.com/abc")).toBeNull();
  });

  it("rejects values that are not URLs at all", () => {
    expect(safeMediaUrl("not a url")).toBeNull();
    expect(safeMediaUrl("/relative/path.png")).toBeNull();
  });
});
