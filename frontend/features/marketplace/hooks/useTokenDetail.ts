"use client";

import { useQuery } from "@tanstack/react-query";

import { MARKET_CONTRACT_ID } from "@/lib/near/contracts";

import { fetchSale, fetchToken } from "../api/market-api";
import { marketKeys } from "../api/query-keys";
import { isSaleStale } from "../api/sale-validity";

import { useMarketChain } from "./useMarketChain";

import type { NftToken, Sale } from "../types/marketplace.types";

export interface TokenDetail {
  token: NftToken | null;
  sale: Sale | null;
  /** Listing ada tapi sudah tidak sah → halaman token menampilkan "no longer available". */
  stale: boolean;
}

/**
 * Detail satu token untuk halaman `/token/[contract]/[tokenId]`: metadata + status listing.
 *
 * `stale` di sini **tidak** menyembunyikan apa pun — halaman token justru tempat user
 * melihat "tidak lagi tersedia" (docs/features/marketplace.md §Discovery); penyembunyian
 * hanya berlaku di grid.
 */
export function useTokenDetail(nftContractId: string, tokenId: string) {
  const chain = useMarketChain();
  const marketContractId = MARKET_CONTRACT_ID;

  return useQuery<TokenDetail>({
    queryKey: marketKeys.token(nftContractId, tokenId),
    queryFn: async () => {
      const token = await fetchToken(chain, nftContractId, tokenId);
      const sale =
        marketContractId === null
          ? null
          : await fetchSale(chain, marketContractId, nftContractId, tokenId);

      const stale =
        sale !== null && marketContractId !== null
          ? isSaleStale(sale, token, marketContractId)
          : false;

      return { token, sale, stale };
    },
  });
}
