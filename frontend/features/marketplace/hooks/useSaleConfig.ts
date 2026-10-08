"use client";

import { useQuery } from "@tanstack/react-query";

import { MARKET_CONTRACT_ID } from "@/lib/near/contracts";

import { fetchFeeBps, fetchRoyaltyConfig } from "../api/market-api";
import { marketKeys } from "../api/query-keys";
import { useMarketChain } from "./useMarketChain";

/**
 * Parameter ekonomi untuk breakdown pembayaran: fee platform (dibaca dari market, tidak
 * di-hardcode) dan tingkat royalti koleksi.
 *
 * Keduanya **bisa gagal sendiri-sendiri**: koleksi pihak ketiga mungkin tidak punya
 * `royalty_config`. Karena itu hasilnya boleh `null` dan pemanggil menampilkan breakdown
 * tanpa baris royalti, bukan gagal total.
 */
export function useSaleConfig(nftContractId: string) {
  const chain = useMarketChain();
  const marketContractId = MARKET_CONTRACT_ID;

  const fee = useQuery<number | null>({
    queryKey: marketKeys.feeBps(),
    enabled: marketContractId !== null,
    queryFn: async () => (marketContractId === null ? null : fetchFeeBps(chain, marketContractId)),
  });

  const royalty = useQuery<number | null>({
    queryKey: marketKeys.royalty(nftContractId),
    queryFn: async () => {
      try {
        const config = await fetchRoyaltyConfig(chain, nftContractId);
        return config?.bps ?? null;
      } catch {
        // Koleksi tanpa `royalty_config` bukan kegagalan halaman — breakdown tampil tanpa royalti.
        return null;
      }
    },
  });

  return {
    feeBps: fee.data ?? null,
    royaltyBps: royalty.data ?? null,
    isLoading: fee.isLoading || royalty.isLoading,
  };
}
