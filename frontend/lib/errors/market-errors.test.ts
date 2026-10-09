// TC-040 · AC: kegagalan tx memetakan ke kode registry (bukan pesan ad-hoc)
import { describe, expect, it } from "vitest";

import {
  KnownTxError,
  classifyTxError,
  configFailure,
  priceChangedFailure,
  staleFailure,
} from "./market-errors";

describe("classifyTxError", () => {
  it("maps a wallet rejection to its own kind, not a contract error", () => {
    expect(classifyTxError(new Error("User rejected the request"))).toEqual({ kind: "rejected" });
    expect(classifyTxError(new Error("Request was cancelled in wallet"))).toEqual({
      kind: "rejected",
    });
  });

  it("matches contract panic codes as whole tokens", () => {
    const cases: Array<[string, string]> = [
      ["CONFLICT_ALREADY_LISTED", "CONFLICT_ALREADY_LISTED"],
      ["CONFLICT_SOLD", "CONFLICT_SOLD"],
      ["FORBIDDEN_SELF_BUY", "FORBIDDEN_SELF_BUY"],
      ["FORBIDDEN_BUYER", "FORBIDDEN_BUYER"],
      ["INVALID_PRICE", "INVALID_PRICE"],
      ["CHAIN_PAUSED", "CHAIN_PAUSED"],
      ["CHAIN_INSUFFICIENT_DEPOSIT", "CHAIN_INSUFFICIENT_DEPOSIT"],
    ];

    for (const [message, code] of cases) {
      expect(classifyTxError(new Error(`Smart contract panicked: ${message}`))).toEqual({
        kind: "code",
        code,
      });
    }
  });

  it("does not match a code that is only a substring of another token", () => {
    // "XCONFLICT_SOLDX" bukan kode; harus jatuh ke fallback, bukan dipetakan sebagai sold.
    expect(classifyTxError(new Error("XCONFLICT_SOLDX"))).toEqual({
      kind: "code",
      code: "CHAIN_REVERT",
    });
  });

  it("keeps a suffixed library revert as CHAIN_REVERT", () => {
    expect(classifyTxError(new Error("CHAIN_REVERT: storage accounting failed"))).toEqual({
      kind: "code",
      code: "CHAIN_REVERT",
    });
  });

  it("classifies insufficient balance and timeouts", () => {
    expect(classifyTxError(new Error("insufficient balance for the transaction"))).toEqual({
      kind: "code",
      code: "CHAIN_INSUFFICIENT_DEPOSIT",
    });
    expect(classifyTxError(new Error("Timeout waiting for receipt"))).toEqual({
      kind: "code",
      code: "CHAIN_TIMEOUT",
    });
  });

  it("falls back to CHAIN_REVERT for anything unrecognised", () => {
    expect(classifyTxError(new Error("weird provider failure"))).toEqual({
      kind: "code",
      code: "CHAIN_REVERT",
    });
    expect(classifyTxError(undefined)).toEqual({ kind: "code", code: "CHAIN_REVERT" });
  });

  it("honours a failure the caller already knows, without re-parsing a message", () => {
    // Jalur seperti re-verify membawa kodenya langsung; tidak ada teks yang perlu ditebak.
    expect(classifyTxError(new KnownTxError(staleFailure()))).toEqual({
      kind: "code",
      code: "CONFLICT_STALE",
    });
    expect(classifyTxError(new KnownTxError(priceChangedFailure()))).toEqual({
      kind: "code",
      code: "CONFLICT_PRICE_CHANGED",
    });
  });
});

describe("pre-sign failures", () => {
  it("reports a stale listing and a changed price as their own codes", () => {
    expect(staleFailure()).toEqual({ kind: "code", code: "CONFLICT_STALE" });
    expect(priceChangedFailure()).toEqual({ kind: "code", code: "CONFLICT_PRICE_CHANGED" });
    expect(configFailure()).toEqual({ kind: "code", code: "SERVER_ERROR" });
  });
});
