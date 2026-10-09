"use client";

import { useState } from "react";

import { t } from "@/i18n";

import { useMarketChain } from "../hooks/useMarketChain";

import { BuyModal } from "./BuyModal";
import { ListModal } from "./ListModal";
import { PriceChip } from "./PriceChip";

import type { NftToken, Sale } from "../types/marketplace.types";

/**
 * Aksi yang tersedia untuk satu token, ditentukan oleh hubungan pemanggil dengan token itu:
 * pemilik melihat **Sell**, bukan-pemilik melihat **Buy** (tiket 10 AC).
 *
 * `transactionsDisabled` (akun belum terbukti ada di jaringan terkonfigurasi) mematikan
 * kedua tombol — hard block, bukan sekadar banner (features/auth.md §Account).
 */
export function TokenActions({
  nftContractId,
  token,
  sale,
  stale,
}: {
  nftContractId: string;
  token: NftToken | null;
  sale: Sale | null;
  stale: boolean;
}) {
  const chain = useMarketChain();
  const [listOpen, setListOpen] = useState(false);
  const [buyOpen, setBuyOpen] = useState(false);

  if (token === null) return null;

  const isOwner = chain.accountId !== null && chain.accountId === token.ownerId;
  const isSelf = chain.accountId !== null && sale !== null && chain.accountId === sale.ownerId;
  const blocked = chain.transactionsDisabled;

  if (isOwner && sale === null) {
    return (
      <>
        <button
          type="button"
          onClick={() => setListOpen(true)}
          disabled={blocked}
          className="rounded-md bg-zinc-900 px-4 py-2 text-sm font-medium text-white disabled:opacity-60 dark:bg-zinc-100 dark:text-zinc-900"
        >
          {t("marketplace.list.button")}
        </button>
        <ListModal
          nftContractId={nftContractId}
          token={token}
          open={listOpen}
          onClose={() => setListOpen(false)}
        />
      </>
    );
  }

  if (isOwner) {
    return <p className="text-sm text-zinc-500">{t("marketplace.detail.youOwn")}</p>;
  }

  if (sale === null || stale) {
    return (
      <p role="status" className="text-sm text-zinc-500">
        {stale ? t("marketplace.stale.notice") : t("marketplace.detail.notListed")}
      </p>
    );
  }

  return (
    <>
      <div className="flex items-center gap-3">
        <PriceChip amountYocto={sale.priceYocto} className="text-xl font-semibold" />
        <button
          type="button"
          onClick={() => setBuyOpen(true)}
          disabled={blocked || isSelf}
          title={isSelf ? t("errors.FORBIDDEN_SELF_BUY") : undefined}
          className="rounded-md bg-zinc-900 px-4 py-2 text-sm font-medium text-white disabled:opacity-60 dark:bg-zinc-100 dark:text-zinc-900"
        >
          {t("marketplace.buy.button")}
        </button>
      </div>
      <BuyModal sale={sale} open={buyOpen} onClose={() => setBuyOpen(false)} />
    </>
  );
}
