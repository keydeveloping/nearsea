"use client";

import { t } from "@/i18n";
import { MARKET_CONTRACT_ID } from "@/lib/near/contracts";

import { useListings } from "../hooks/useListings";

import { ListingCard } from "./ListingCard";

/**
 * Grid discovery: listing dibaca langsung dari kontrak lewat view call RPC
 * (docs/features/marketplace.md §Discovery — MVP tanpa endpoint discovery).
 */
export function ListingGrid() {
  const { data, isPending, isError, refetch } = useListings();

  if (MARKET_CONTRACT_ID === null) {
    return (
      <div className="flex flex-col items-center gap-2 rounded-lg border border-dashed border-zinc-300 p-12 text-center dark:border-zinc-700">
        <h2 className="font-semibold">{t("marketplace.config.title")}</h2>
        <p className="max-w-md text-sm text-zinc-500">{t("marketplace.config.body")}</p>
      </div>
    );
  }

  if (isPending) {
    return (
      <p role="status" className="py-12 text-center text-sm text-zinc-500">
        {t("marketplace.grid.loading")}
      </p>
    );
  }

  if (isError) {
    return (
      <div className="flex flex-col items-center gap-2 py-12 text-center">
        <p role="alert" className="text-sm text-red-600 dark:text-red-400">
          {t("marketplace.grid.error")}
        </p>
        <button
          type="button"
          onClick={() => void refetch()}
          className="rounded-md border border-zinc-300 px-3 py-1.5 text-sm font-medium dark:border-zinc-700"
        >
          {t("marketplace.grid.retry")}
        </button>
      </div>
    );
  }

  if (data.length === 0) {
    return <p className="py-12 text-center text-sm text-zinc-500">{t("marketplace.grid.empty")}</p>;
  }

  return (
    <ul className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {data.map((sale) => (
        <li key={`${sale.nftContractId}:${sale.tokenId}`}>
          <ListingCard sale={sale} />
        </li>
      ))}
    </ul>
  );
}
