"use client";

import Link from "next/link";

import { t } from "@/i18n";
import { MARKET_CONTRACT_ID } from "@/lib/near/contracts";

import { useTokenDetail } from "./hooks/useTokenDetail";
import { PriceChip } from "./components/PriceChip";
import { TokenActions } from "./components/TokenActions";
import { TokenMedia } from "./components/TokenMedia";

/**
 * Halaman detail token: metadata, harga, dan status listing.
 *
 * Listing yang sudah stale **tidak disembunyikan** di sini — halaman token justru tempat user
 * melihat "no longer available" (docs/features/marketplace.md §Discovery). Penyembunyian
 * hanya berlaku di grid.
 */
export function TokenPage({ nftContractId, tokenId }: { nftContractId: string; tokenId: string }) {
  const { data, isPending, isError } = useTokenDetail(nftContractId, tokenId);

  if (isPending) {
    return (
      <main className="mx-auto w-full max-w-3xl flex-1 px-6 py-10">
        <p role="status" className="text-sm text-zinc-500">
          {t("marketplace.detail.loading")}
        </p>
      </main>
    );
  }

  if (isError || data === undefined || data.token === null) {
    return (
      <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col gap-4 px-6 py-10">
        <p role="alert" className="text-sm text-zinc-600 dark:text-zinc-400">
          {t("marketplace.detail.notFound")}
        </p>
        <Link href="/" className="text-sm underline">
          {t("marketplace.detail.back")}
        </Link>
      </main>
    );
  }

  const { token, sale, stale } = data;

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col gap-6 px-6 py-10">
      <Link href="/" className="text-sm underline">
        {t("marketplace.detail.back")}
      </Link>

      <div className="flex flex-col gap-6 sm:flex-row">
        <TokenMedia
          media={token.media}
          name={`#${token.tokenId}`}
          className="aspect-square w-full max-w-xs rounded-lg text-5xl sm:w-64"
        />

        <div className="flex flex-1 flex-col gap-4">
          <h1 className="text-2xl font-semibold tracking-tight">
            {token.title ?? `#${token.tokenId}`}
          </h1>

          <dl className="flex flex-col gap-1 text-sm">
            <div className="flex gap-2">
              <dt className="text-zinc-500">{t("marketplace.detail.collection")}</dt>
              <dd className="font-mono">{nftContractId}</dd>
            </div>
            <div className="flex gap-2">
              <dt className="text-zinc-500">{t("marketplace.detail.owner")}</dt>
              <dd className="font-mono">{token.ownerId}</dd>
            </div>
            <div className="flex gap-2">
              <dt className="text-zinc-500">{t("marketplace.detail.status")}</dt>
              <dd>
                {sale === null
                  ? t("marketplace.detail.notListed")
                  : stale
                    ? t("marketplace.stale.badge")
                    : t("marketplace.detail.listed")}
              </dd>
            </div>
            {sale !== null && !stale ? (
              <div className="flex gap-2">
                <dt className="text-zinc-500">{t("marketplace.detail.price")}</dt>
                <dd>
                  <PriceChip amountYocto={sale.priceYocto} />
                </dd>
              </div>
            ) : null}
          </dl>

          {token.description !== null ? (
            <p className="text-sm text-zinc-600 dark:text-zinc-400">{token.description}</p>
          ) : null}

          {MARKET_CONTRACT_ID === null ? (
            <p role="status" className="text-sm text-zinc-500">
              {t("marketplace.config.body")}
            </p>
          ) : (
            <TokenActions nftContractId={nftContractId} token={token} sale={sale} stale={stale} />
          )}
        </div>
      </div>
    </main>
  );
}
