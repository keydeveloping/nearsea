/**
 * Perakitan data discovery: `get_sales` + satu `nft_token` per listing untuk membuktikan
 * keabsahannya. Deteksi stale bersifat **lazy di sisi baca** — kontrak tidak punya cron
 * (docs/features/marketplace.md §Stale Detection), jadi penyaringan harus dilakukan di sini.
 *
 * Kegagalan RPC saat memeriksa satu listing **tidak** menghapus listing itu dari hasil:
 * "tidak tahu" bukan "basi". Hanya bukti positif (token tidak ada, pemilik pindah, approval
 * hilang) yang menyembunyikan listing — kalau tidak, RPC yang sedang sakit akan mengosongkan
 * seluruh marketplace.
 */

import type { MarketChain, Sale } from "../types/marketplace.types";
import { fetchSales, fetchToken } from "./market-api";
import { isPublicListing, isSaleStale } from "./sale-validity";

export async function loadVisibleListings(
  chain: MarketChain,
  marketContractId: string,
): Promise<Sale[]> {
  const sales = await fetchSales(chain, marketContractId);
  const publicSales = sales.filter(isPublicListing);

  const checks = await Promise.all(
    publicSales.map(async (sale) => {
      try {
        const token = await fetchToken(chain, sale.nftContractId, sale.tokenId);
        return !isSaleStale(sale, token, marketContractId);
      } catch {
        // Verifikasi tidak pasti (RPC/koleksi tidak bisa dihubungi): listing tetap tampil.
        return true;
      }
    }),
  );

  return publicSales.filter((_, index) => checks[index]);
}
