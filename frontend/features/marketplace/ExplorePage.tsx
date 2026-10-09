"use client";

import { t } from "@/i18n";

import { ListingGrid } from "./components/ListingGrid";

/** Halaman Explore — grid listing dari kontrak (route `/`). */
export function ExplorePage() {
  return (
    <main className="mx-auto flex w-full max-w-6xl flex-1 flex-col gap-6 px-6 py-10">
      <h1 className="text-3xl font-semibold tracking-tight">{t("marketplace.grid.title")}</h1>
      <ListingGrid />
    </main>
  );
}
