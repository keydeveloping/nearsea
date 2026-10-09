/**
 * Pemetaan kode error → kunci i18n. Kode **verbatim** dari registry
 * (docs/development/error-handling.md §8): `errors.<CODE>`.
 *
 * Tabel eksplisit (bukan `errors.${code}` yang dirakit) supaya kode baru tanpa kunci
 * = error build, bukan pesan kosong saat runtime.
 */

import type { MessageKey } from "@/i18n";
import type { MarketErrorCode, TxFailure } from "@/lib/errors/market-errors";

const ERROR_KEYS: Record<MarketErrorCode, MessageKey> = {
  INVALID_PRICE: "errors.INVALID_PRICE",
  CONFLICT_SOLD: "errors.CONFLICT_SOLD",
  CONFLICT_STALE: "errors.CONFLICT_STALE",
  CONFLICT_ALREADY_LISTED: "errors.CONFLICT_ALREADY_LISTED",
  CONFLICT_PRICE_CHANGED: "errors.CONFLICT_PRICE_CHANGED",
  FORBIDDEN_SELF_BUY: "errors.FORBIDDEN_SELF_BUY",
  FORBIDDEN_BUYER: "errors.FORBIDDEN_BUYER",
  CHAIN_INSUFFICIENT_DEPOSIT: "errors.CHAIN_INSUFFICIENT_DEPOSIT",
  CHAIN_PAUSED: "errors.CHAIN_PAUSED",
  CHAIN_REVERT: "errors.CHAIN_REVERT",
  CHAIN_TIMEOUT: "errors.CHAIN_TIMEOUT",
  NOT_FOUND_TOKEN: "errors.NOT_FOUND_TOKEN",
  SERVER_ERROR: "errors.SERVER_ERROR",
};

/** Kunci pesan untuk sebuah kegagalan transaksi; pembatalan di wallet bukan error kontrak. */
export function failureKey(failure: TxFailure): MessageKey {
  return failure.kind === "rejected" ? "marketplace.tx.cancelled" : ERROR_KEYS[failure.code];
}
