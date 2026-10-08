/**
 * Klasifikasi kegagalan transaksi kontrak menjadi **kode** registry
 * (docs/development/error-handling.md §3). Pemetaan kode → pesan user dilakukan komponen
 * lewat i18n, bukan di sini — pola yang sama dengan `lib/near/wallet-errors.ts`.
 *
 * Panic kontrak dibandingkan sebagai **token kode**, bukan substring teks manusia
 * (error-handling.md §8: teks boleh berubah, kode stabil). Kode tak dikenal = `CHAIN_REVERT`,
 * dan kode mentah tidak pernah ditampilkan ke user.
 */

export type MarketErrorCode =
  | "INVALID_PRICE"
  | "CONFLICT_SOLD"
  | "CONFLICT_STALE"
  | "CONFLICT_ALREADY_LISTED"
  | "CONFLICT_PRICE_CHANGED"
  | "FORBIDDEN_SELF_BUY"
  | "FORBIDDEN_BUYER"
  | "CHAIN_INSUFFICIENT_DEPOSIT"
  | "CHAIN_PAUSED"
  | "CHAIN_REVERT"
  | "CHAIN_TIMEOUT"
  | "NOT_FOUND_TOKEN"
  | "SERVER_ERROR";

/**
 * `rejected` dipisah dari `code` karena ia bukan kegagalan kontrak: user membatalkan di wallet,
 * jadi copy-nya "dibatalkan", bukan "transaksi gagal".
 */
export type TxFailure = { kind: "rejected" } | { kind: "code"; code: MarketErrorCode };

/** Urutan penting: kode yang lebih spesifik diperiksa lebih dulu. */
const CODE_PATTERNS: ReadonlyArray<readonly [MarketErrorCode, RegExp]> = [
  ["CONFLICT_ALREADY_LISTED", /\bCONFLICT_ALREADY_LISTED\b/],
  ["CONFLICT_SOLD", /\bCONFLICT_SOLD\b/],
  ["CONFLICT_STALE", /\bCONFLICT_STALE\b/],
  ["FORBIDDEN_SELF_BUY", /\bFORBIDDEN_SELF_BUY\b/],
  ["FORBIDDEN_BUYER", /\bFORBIDDEN_BUYER\b/],
  ["CHAIN_INSUFFICIENT_DEPOSIT", /\bCHAIN_INSUFFICIENT_DEPOSIT\b/],
  ["CHAIN_PAUSED", /\bCHAIN_PAUSED\b/],
  ["INVALID_PRICE", /\bINVALID_PRICE\b/],
  ["CHAIN_REVERT", /\bCHAIN_REVERT\b/],
];

const REJECTED_PATTERN = /user rejected|user denied|rejected by user|declined by user|cancell?ed/i;
const TIMEOUT_PATTERN = /timeout|timed out|timed-out/i;
const BALANCE_PATTERN = /insufficient (balance|funds|allowance)|not enough (balance|near|funds)/i;

function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (typeof error !== "object" || error === null) return "";

  const record = error as Record<string, unknown>;
  const parts = [record.type, record.message, record.kind].filter(
    (part): part is string => typeof part === "string",
  );
  return parts.join(" ");
}

export function classifyTxError(error: unknown): TxFailure {
  if (isKnownTxError(error)) return error.failure;

  const text = errorText(error);

  if (REJECTED_PATTERN.test(text)) return { kind: "rejected" };

  for (const [code, pattern] of CODE_PATTERNS) {
    if (pattern.test(text)) return { kind: "code", code };
  }

  if (BALANCE_PATTERN.test(text)) return { kind: "code", code: "CHAIN_INSUFFICIENT_DEPOSIT" };
  if (TIMEOUT_PATTERN.test(text)) return { kind: "code", code: "CHAIN_TIMEOUT" };

  return { kind: "code", code: "CHAIN_REVERT" };
}

/**
 * Kegagalan yang **tidak berasal** dari tx: data view tidak bisa dipercaya lagi (listing
 * sudah tidak valid / harga berubah sejak modal dibuka). Dipakai jalur re-verify sebelum
 * signing (SEC-ORDER-003) — user tidak boleh menandatangani argumen basi.
 */
export function staleFailure(): TxFailure {
  return { kind: "code", code: "CONFLICT_STALE" };
}

export function priceChangedFailure(): TxFailure {
  return { kind: "code", code: "CONFLICT_PRICE_CHANGED" };
}

/** Kontrak market belum dikonfigurasi (`NEXT_PUBLIC_MARKET_CONTRACT_ID` kosong). */
export function configFailure(): TxFailure {
  return { kind: "code", code: "SERVER_ERROR" };
}

/**
 * Kode yang **diketahui pemanggil** tanpa perlu membaca teks error (mis. state on-chain
 * tidak sesuai harapan setelah satu langkah alur). Membawa kode registry langsung, jadi tidak
 * ada pesan ad-hoc yang harus ditebak ulang oleh regex.
 */
export class KnownTxError extends Error {
  constructor(readonly failure: TxFailure) {
    super("Known transaction failure");
  }
}

/** Peringatan tipe supaya `classifyTxError` bisa membedakannya dari error biasa. */
export function isKnownTxError(error: unknown): error is KnownTxError {
  return error instanceof KnownTxError;
}
