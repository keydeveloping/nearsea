"use client";

import Link from "next/link";

import { PriceChip } from "./PriceChip";

import type { Sale } from "../types/marketplace.types";

/**
 * Satu kartu listing di grid. Klik = menuju halaman token (route `/token/[contract]/[tokenId]`).
 *
 * Gambar & judul tidak dibaca di sini: `get_sales` tidak membawanya, dan satu `nft_token` per
 * kartu akan menggandakan jumlah view call grid. Kartu menampilkan identitas on-chain apa adanya
 * (`token_id` + koleksi) alih-alih menebak judul.
 */
export function ListingCard({ sale }: { sale: Sale }) {
  return (
    <Link
      href={`/token/${encodeURIComponent(sale.nftContractId)}/${encodeURIComponent(sale.tokenId)}`}
      className="group flex flex-col overflow-hidden rounded-lg border border-zinc-200 transition hover:border-zinc-400 dark:border-zinc-800 dark:hover:border-zinc-600"
    >
      <div
        aria-hidden="true"
        className="flex aspect-square items-center justify-center bg-zinc-100 text-3xl font-mono text-zinc-400 dark:bg-zinc-900"
      >
        #{sale.tokenId}
      </div>

      <div className="flex flex-col gap-1 p-3">
        <span className="truncate text-sm font-medium">#{sale.tokenId}</span>
        <span className="truncate font-mono text-xs text-zinc-500">{sale.nftContractId}</span>
        <PriceChip amountYocto={sale.priceYocto} className="text-sm font-semibold" />
      </div>
    </Link>
  );
}
