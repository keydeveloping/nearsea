"use client";

import { t, type MessageKey } from "@/i18n";

import { failureKey } from "../error-messages";

import type { TxFailure } from "@/lib/errors/market-errors";

/**
 * Umpan balik transaksi: sukses dan gagal **sama-sama terbaca**, tidak ada yang diam
 * (tiket 10 AC). Gagal = `role="alert"` (persisten, dengan aksi), sukses = `role="status"`.
 */
export function TxFeedback({
  failure,
  successKey,
  onRetry,
  onDismiss,
}: {
  failure: TxFailure | null;
  successKey?: MessageKey;
  onRetry?: () => void;
  onDismiss?: () => void;
}) {
  if (failure !== null) {
    return (
      <p
        role="alert"
        className="flex flex-wrap items-center gap-2 text-sm text-red-600 dark:text-red-400"
      >
        {t(failureKey(failure))}
        {onRetry ? (
          <button type="button" onClick={onRetry} className="underline">
            {t("marketplace.tx.retry")}
          </button>
        ) : null}
        {onDismiss ? (
          <button type="button" onClick={onDismiss} className="underline">
            {t("marketplace.tx.dismiss")}
          </button>
        ) : null}
      </p>
    );
  }

  if (successKey === undefined) return null;

  return (
    <p role="status" className="text-sm text-green-700 dark:text-green-400">
      {t(successKey)}
    </p>
  );
}
