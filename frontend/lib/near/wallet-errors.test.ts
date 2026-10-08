// TC-040 · AC-WALLET-2, AC-WALLET-3 · SEC-FE-002
import { describe, expect, it } from "vitest";

import {
  classifyAccountProbe,
  classifyAccountProbeError,
  classifyConnectError,
  isMissingAccountError,
} from "@/lib/near/wallet-errors";

describe("classifyConnectError", () => {
  it("maps a user rejection to `rejected` (local, no API code)", () => {
    expect(classifyConnectError(new Error("User rejected the request"))).toBe("rejected");
    expect(classifyConnectError(new Error("User denied"))).toBe("rejected");
    expect(classifyConnectError(new Error("User rejected"))).toBe("rejected");
  });

  it("maps a missing/unavailable wallet to `wallet_unavailable`", () => {
    expect(classifyConnectError(new Error("Wallet not installed"))).toBe("wallet_unavailable");
    expect(classifyConnectError(new Error("No accounts found"))).toBe("wallet_unavailable");
  });

  it("falls back to `connect_failed` for anything else, including non-errors", () => {
    expect(classifyConnectError(new Error("Network timeout"))).toBe("connect_failed");
    expect(classifyConnectError("weird string failure")).toBe("connect_failed");
    expect(classifyConnectError(undefined)).toBe("connect_failed");
    expect(classifyConnectError(null)).toBe("connect_failed");
  });

  it("reads RPC-style typed errors ({ type, message })", () => {
    expect(classifyConnectError({ type: "UserRejected", message: "rejected by user" })).toBe(
      "rejected",
    );
  });
});

describe("isMissingAccountError", () => {
  it("detects the RPC error for an account that does not exist on this network", () => {
    expect(isMissingAccountError(new Error("Account does not exist while viewing"))).toBe(true);
    expect(isMissingAccountError({ type: "UNKNOWN_ACCOUNT", message: "unknown account" })).toBe(
      true,
    );
  });

  it("does not treat a transient RPC failure as a missing account", () => {
    expect(isMissingAccountError(new Error("Timeout"))).toBe(false);
  });
});

describe("classifyAccountProbe", () => {
  it("carries the balance as a yoctoNEAR string (never a number)", () => {
    const result = classifyAccountProbe(123456789012345678901234567890n);

    expect(result).toEqual({ kind: "ready", balanceYocto: "123456789012345678901234567890" });
  });

  it("classifies a missing account as `missing` — the network-mismatch signal", () => {
    expect(classifyAccountProbeError(new Error("Account does not exist while viewing"))).toEqual({
      kind: "missing",
    });
  });

  it("classifies any other probe failure as `unavailable`, not as a mismatch", () => {
    expect(classifyAccountProbeError(new Error("RPC timeout"))).toEqual({ kind: "unavailable" });
  });
});
