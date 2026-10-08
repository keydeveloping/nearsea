/**
 * Verifikasi pembelian tepat sebelum signing (SEC-ORDER-003, P0).
 *
 * Argumen yang akan ditandatangani harus berasal dari view call **segar**, bukan dari data yang
 * ditampilkan modal: harga bisa berubah dan listing bisa menjadi basi di antara keduanya.
 * Fungsi ini mengembalikan `Sale` terbaru atau melempar `KnownTxError` berisi kode registry —
 * tidak pernah mengembalikan data yang tidak layak ditandatangani.
 */

import { KnownTxError, priceChangedFailure, staleFailure } from "@/lib/errors/market-errors";

import type { MarketChain, Sale } from "../types/marketplace.types";
import { fetchSale, fetchToken } from "./market-api";
import { isSaleStale } from "./sale-validity";

export async function verifySaleForPurchase(
  chain: MarketChain,
  marketContractId: string,
  sale: Sale,
): Promise<Sale> {
  const [fresh, token] = await Promise.all([
    fetchSale(chain, marketContractId, sale.nftContractId, sale.tokenId),
    fetchToken(chain, sale.nftContractId, sale.tokenId),
  ]);

  if (fresh === null || isSaleStale(fresh, token, marketContractId)) {
    throw new KnownTxError(staleFailure());
  }

  if (fresh.priceYocto !== sale.priceYocto) {
    throw new KnownTxError(priceChangedFailure());
  }

  return fresh;
}
