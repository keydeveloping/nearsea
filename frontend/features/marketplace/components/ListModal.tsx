"use client";

import { useState } from "react";

import { Modal } from "@/components/ui/Modal";
import { format, t } from "@/i18n";
import { asYoctoNear, formatNear, parseNearInput } from "@/lib/format/money";
import { MARKET_CONTRACT_ID } from "@/lib/near/contracts";

import { useListNft } from "../hooks/useListNft";
import { useMarketChain } from "../hooks/useMarketChain";
import { useStorageRequirement } from "../hooks/useStorageRequirement";

import { TxFeedback } from "./TxFeedback";

import type { NftToken } from "../types/marketplace.types";

/** Harga minimum listing = 0.01 Ⓝ (konstanta kontrak `MIN_PRICE_YOCTO`). */
const MIN_PRICE_YOCTO = asYoctoNear("10000000000000000000000");

const ACCOUNT_ID_PATTERN = /^(([a-z\d]+[-_])*[a-z\d]+\.)*([a-z\d]+[-_])*[a-z\d]+$/;

function isAccountId(value: string): boolean {
  return value.length >= 2 && value.length <= 64 && ACCOUNT_ID_PATTERN.test(value);
}

/**
 * Modal listing dengan progres **dua langkah tanda tangan** (docs/02-product-requirements.md
 * §UI copy listing): harga diisi di form ini, lalu approve (1/2) dan list (2/2). Deposit
 * storage ditampilkan **sebelum** signing supaya tidak muncul sebagai kejutan (temuan UX B4).
 */
export function ListModal({
  nftContractId,
  token,
  open,
  onClose,
}: {
  nftContractId: string;
  token: NftToken;
  open: boolean;
  onClose: () => void;
}) {
  const chain = useMarketChain();
  const { step, failure, submit, reset } = useListNft();
  const storage = useStorageRequirement(chain.accountId);

  const [priceInput, setPriceInput] = useState("");
  const [isPrivate, setIsPrivate] = useState(false);
  const [buyerInput, setBuyerInput] = useState("");
  const [formError, setFormError] = useState<"price" | "buyer" | null>(null);

  const busy = step === "approving" || step === "listing";
  const priceYocto = parseNearInput(priceInput);

  function close() {
    reset();
    setPriceInput("");
    setIsPrivate(false);
    setBuyerInput("");
    setFormError(null);
    onClose();
  }

  async function onSubmit() {
    if (priceYocto === null || BigInt(priceYocto) < BigInt(MIN_PRICE_YOCTO)) {
      setFormError("price");
      return;
    }
    if (isPrivate && !isAccountId(buyerInput.trim())) {
      setFormError("buyer");
      return;
    }
    setFormError(null);

    await submit({
      nftContractId,
      tokenId: token.tokenId,
      priceYocto,
      allowedBuyer: isPrivate ? buyerInput.trim() : null,
      storageDepositYocto: storage.data ?? asYoctoNear("0"),
    });
  }

  return (
    <Modal open={open} title={t("marketplace.list.title")} onClose={close} dismissible={!busy}>
      {step === "success" ? (
        <div className="flex flex-col gap-4">
          <TxFeedback failure={null} successKey="marketplace.list.success" />
          <button
            type="button"
            onClick={close}
            className="rounded-md bg-zinc-900 px-3 py-2 text-sm font-medium text-white dark:bg-zinc-100 dark:text-zinc-900"
          >
            {t("marketplace.tx.close")}
          </button>
        </div>
      ) : (
        <form
          className="flex flex-col gap-4"
          onSubmit={(event) => {
            event.preventDefault();
            void onSubmit();
          }}
        >
          <label className="flex flex-col gap-1 text-sm">
            {t("marketplace.list.priceLabel")}
            <input
              type="text"
              inputMode="decimal"
              value={priceInput}
              onChange={(event) => setPriceInput(event.target.value)}
              disabled={busy}
              className="rounded-md border border-zinc-300 px-3 py-2 font-mono dark:border-zinc-700 dark:bg-zinc-900"
            />
            <span className="text-xs text-zinc-500">{t("marketplace.list.priceMin")}</span>
          </label>

          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={isPrivate}
              onChange={(event) => setIsPrivate(event.target.checked)}
              disabled={busy}
            />
            {t("marketplace.list.privateToggle")}
          </label>

          {isPrivate ? (
            <label className="flex flex-col gap-1 text-sm">
              {t("marketplace.list.allowedBuyerLabel")}
              <input
                type="text"
                value={buyerInput}
                onChange={(event) => setBuyerInput(event.target.value)}
                disabled={busy}
                className="rounded-md border border-zinc-300 px-3 py-2 font-mono dark:border-zinc-700 dark:bg-zinc-900"
              />
            </label>
          ) : null}

          {formError !== null ? (
            <p role="alert" className="text-sm text-red-600 dark:text-red-400">
              {formError === "price"
                ? t("marketplace.list.priceInvalid")
                : t("marketplace.list.allowedBuyerInvalid")}
            </p>
          ) : null}

          {storage.data != null && BigInt(storage.data) > 0n ? (
            <p className="text-xs text-zinc-500">
              {format(t("marketplace.list.storageNote"), { amount: formatNear(storage.data) })}
            </p>
          ) : null}

          {busy ? (
            <p role="status" className="text-sm text-zinc-600 dark:text-zinc-400">
              {step === "approving" ? t("marketplace.list.step1") : t("marketplace.list.step2")}
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
              type="submit"
              disabled={busy || MARKET_CONTRACT_ID === null}
              aria-busy={busy}
              className="rounded-md bg-zinc-900 px-3 py-2 text-sm font-medium text-white disabled:opacity-60 dark:bg-zinc-100 dark:text-zinc-900"
            >
              {t("marketplace.list.submit")}
            </button>
          </div>
        </form>
      )}
    </Modal>
  );
}
