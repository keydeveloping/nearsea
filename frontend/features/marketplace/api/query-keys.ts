/**
 * Kunci cache TanStack Query (docs/architecture/frontend-architecture.md §Data fetching).
 * Satu tempat mendefinisikan bentuk kunci supaya invalidasi tidak menulis string lepas.
 */

import type { QueryClient } from "@tanstack/react-query";

export const marketKeys = {
  listings: () => ["listings"] as const,
  token: (nftContractId: string, tokenId: string) => ["token", nftContractId, tokenId] as const,
  royalty: (nftContractId: string) => ["royalty", nftContractId] as const,
  feeBps: () => ["marketFeeBps"] as const,
  storageRequirement: (accountId: string) => ["storageRequirement", accountId] as const,
};

/** Akar kunci yang harus disegarkan setelah transaksi market berhasil. */
const INVALIDATION_ROOTS: ReadonlyArray<readonly string[]> = [
  ["listings"],
  ["token"],
  ["storageRequirement"],
];

/** Invalidasi **setelah receipt final** — bukan saat submit (frontend-architecture.md §12). */
export async function invalidateMarketQueries(queryClient: QueryClient): Promise<void> {
  await Promise.all(
    INVALIDATION_ROOTS.map((root) => queryClient.invalidateQueries({ queryKey: [...root] })),
  );
}
