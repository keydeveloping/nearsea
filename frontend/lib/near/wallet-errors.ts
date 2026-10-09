import { asYoctoNear, type YoctoNear } from "@/lib/format/money";

/**
 * Klasifikasi error wallet/RPC menjadi kode **lokal** (tanpa kode API — features/auth.md
 * §Error Cases). Pemetaan kode → pesan user dilakukan komponen lewat i18n, bukan di sini.
 */

/** Error saat connect/disconnect. */
export type WalletConnectErrorCode = "rejected" | "wallet_unavailable" | "connect_failed";

/** Hasil probe akun ke RPC jaringan yang dikonfigurasi. */
export type AccountProbeResult =
  { kind: "ready"; balanceYocto: YoctoNear } | { kind: "missing" } | { kind: "unavailable" };

const REJECTED_PATTERN = /user rejected|user denied|rejected by user|cancell?ed/i;
const UNAVAILABLE_PATTERN = /not installed|not found|unavailable|no accounts found|no wallet/i;
const MISSING_ACCOUNT_PATTERN = /does not exist|account not found|unknown account/i;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (!isRecord(error)) return "";

  const message = typeof error.message === "string" ? error.message : "";
  const type = typeof error.type === "string" ? error.type : "";
  return `${type} ${message}`.trim();
}

export function classifyConnectError(error: unknown): WalletConnectErrorCode {
  const text = errorText(error);
  if (REJECTED_PATTERN.test(text)) return "rejected";
  if (UNAVAILABLE_PATTERN.test(text)) return "wallet_unavailable";
  return "connect_failed";
}

/**
 * Akun yang tidak ada di jaringan yang dikonfigurasi = wallet sedang di jaringan lain
 * (atau akun belum di-fund). Dipakai untuk peringatan network mismatch (ticket 09 AC 4).
 */
export function isMissingAccountError(error: unknown): boolean {
  return MISSING_ACCOUNT_PATTERN.test(errorText(error));
}

export function classifyAccountProbe(balanceYocto: bigint): AccountProbeResult {
  return { kind: "ready", balanceYocto: asYoctoNear(balanceYocto.toString()) };
}

export function classifyAccountProbeError(error: unknown): AccountProbeResult {
  return isMissingAccountError(error) ? { kind: "missing" } : { kind: "unavailable" };
}
