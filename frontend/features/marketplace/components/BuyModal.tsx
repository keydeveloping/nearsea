"use client";

import { Modal } from "@/components/ui/Modal";
import { format, t } from "@/i18n";
import { computeSaleBreakdown, percentFromBps } from "@/lib/format/fees";

import { useBuyNft } from "../hooks/useBuyNft";
import { useSaleConfig } from "../hooks/useSaleConfig";

import { PriceChip } from "./PriceChip";
import { TxFeedback } from "./TxFeedback";

import type { Sale } from "../types/marketplace.types";
import type { YoctoNear } from "@/lib/format/money";

function BreakdownRow({ label, amountYocto }: { label: string; amountYocto: YoctoNear }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-zinc-600 dark:text-zinc-400">{label}</span>
      <PriceChip amountYocto={amountYocto} className="font-mono" />
    </div>
  );
}

/**
 * Konfirmasi pembelian dengan **breakdown fee + royalti sebelum konfirmasi**
 * (docs/02-product-requirements.md §UI copy buy). Angka breakdown dihitung dari data view
 * kontrak (fee bps dari market, royalti dari koleksi) — bukan dari cache atau konstanta
 * yang ditanam di FE.
 */
export function BuyModal({
  sale,
  open,
  onClose,
}: {
  sale: Sale;
  open: boolean;
  onClose: () => void;
}) {
  const { status, failure, buy, reset } = useBuyNft();
  const { feeBps, royaltyBps, isLoading } = useSaleConfig(sale.nftContractId);

  const breakdown =
    feeBps === null ? null : computeSaleBreakdown(sale.priceYocto, feeBps, royaltyBps);

  const busy = status === "verifying" || status === "signing";

  function close() {
    reset();
    onClose();
  }

  return (
    <Modal open={open} title={t("marketplace.buy.title")} onClose={close} dismissible={!busy}>
      {status === "success" ? (
        <div className="flex flex-col gap-4">
          <TxFeedback failure={null} successKey="marketplace.buy.success" />
          <button
            type="button"
            onClick={close}
            className="rounded-md bg-zinc-900 px-3 py-2 text-sm font-medium text-white dark:bg-zinc-100 dark:text-zinc-900"
          >
            {t("marketplace.tx.close")}
          </button>
        </div>
      ) : (
        <div className="flex flex-col gap-4">
          <PriceChip amountYocto={sale.priceYocto} className="text-2xl font-semibold" />

          <dl className="flex flex-col gap-1 text-sm">
            {breakdown === null ? (
              // Royalti belum diketahui: hanya fee yang bisa dinyatakan, dan sisa untuk seller
              // sengaja tidak ditampilkan — menghitungnya seolah royalti nol akan melebihkan.
              <div className="flex items-center justify-between gap-4">
                <span className="text-zinc-600 dark:text-zinc-400">
                  {isLoading ? t("marketplace.buy.total") : t("marketplace.buy.royaltyUnavailable")}
                </span>
                <PriceChip amountYocto={sale.priceYocto} className="font-mono" />
              </div>
            ) : (
              <>
                <BreakdownRow
                  label={format(t("marketplace.buy.fee"), {
                    percent: percentFromBps(breakdown.feeBps),
                  })}
                  amountYocto={breakdown.fee}
                />
                <BreakdownRow
                  label={t("marketplace.buy.royalty")}
                  amountYocto={breakdown.royalty}
                />
                <BreakdownRow
                  label={t("marketplace.buy.sellerReceives")}
                  amountYocto={breakdown.sellerProceeds}
                />
                <div className="mt-1 flex items-center justify-between gap-4 border-t border-zinc-200 pt-2 font-semibold dark:border-zinc-800">
                  <span>{t("marketplace.buy.total")}</span>
                  <PriceChip amountYocto={breakdown.price} />
                </div>
              </>
            )}
          </dl>

          <p className="text-xs text-zinc-500">{t("marketplace.buy.estimateNote")}</p>

          {busy ? (
            <p role="status" className="text-sm text-zinc-600 dark:text-zinc-400">
              {status === "verifying"
                ? t("marketplace.buy.verifying")
                : t("marketplace.buy.pending")}
            </p>
          ) : null}

          <TxFeedback failure={failure} onDismiss={close} />

          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={close}
              disabled={busy}
              className="rounded-md border border-zinc-300 px-3 py-2 text-sm font-medium disabled:opacity-60 dark:border-zinc-700"
            >
              {t("marketplace.tx.cancel")}
            </button>
            <button
              type="button"
              onClick={() => void buy(sale)}
              disabled={busy}
              aria-busy={busy}
              className="rounded-md bg-zinc-900 px-3 py-2 text-sm font-medium text-white disabled:opacity-60 dark:bg-zinc-100 dark:text-zinc-900"
            >
              {t("marketplace.buy.confirm")}
            </button>
          </div>
        </div>
      )}
    </Modal>
  );
}
