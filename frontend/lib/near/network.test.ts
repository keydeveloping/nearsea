// TC-040 · AC-WALLET-1 · SEC-FE-001 (konfigurasi jaringan dari env, bukan hardcoded)
import { describe, expect, it } from "vitest";

import { resolveNetwork, rpcProviderUrls } from "@/lib/near/network";

describe("resolveNetwork", () => {
  it("defaults to testnet when the env var is absent", () => {
    expect(resolveNetwork(undefined)).toBe("testnet");
    expect(resolveNetwork("")).toBe("testnet");
    expect(resolveNetwork("   ")).toBe("testnet");
  });

  it("accepts both supported networks", () => {
    expect(resolveNetwork("testnet")).toBe("testnet");
    expect(resolveNetwork("mainnet")).toBe("mainnet");
  });

  it("throws on an unsupported network instead of silently falling back", () => {
    // Nilai salah tulis = transaksi bisa di-sign di jaringan yang salah.
    expect(() => resolveNetwork("Testnet")).toThrowError(/Invalid NEXT_PUBLIC_NEAR_NETWORK/);
    expect(() => resolveNetwork("localnet")).toThrowError(/Invalid NEXT_PUBLIC_NEAR_NETWORK/);
  });
});

describe("rpcProviderUrls", () => {
  it("falls back to the documented default list per network", () => {
    expect(rpcProviderUrls("testnet", undefined, undefined)).toEqual([
      "https://test.rpc.fastnear.com",
      "https://rpc.testnet.near.org",
    ]);
    expect(rpcProviderUrls("mainnet", "", "")).toEqual([
      "https://free.rpc.fastnear.com",
      "https://rpc.mainnet.near.org",
    ]);
  });

  it("puts the configured primary first and appends trimmed fallbacks", () => {
    expect(
      rpcProviderUrls(
        "testnet",
        "https://primary.example",
        " https://a.example , https://b.example ",
      ),
    ).toEqual(["https://primary.example", "https://a.example", "https://b.example"]);
  });

  it("ignores empty entries in a comma-separated list", () => {
    expect(rpcProviderUrls("testnet", "https://primary.example", ",,")).toEqual([
      "https://primary.example",
    ]);
  });
});
