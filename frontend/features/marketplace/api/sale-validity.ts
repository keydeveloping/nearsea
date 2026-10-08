/**
 * Menafsirkan data view menjadi keputusan tampilan: mana listing yang masih sah, mana yang
 * privat, dan urutan deterministik.
 *
 * Deteksi stale adalah evaluasi **lazy di sisi baca** (docs/features/marketplace.md §Stale
 * Detection): kontrak tidak punya cron, jadi listing basi masih muncul di `get_sales` sampai
 * ada yang menyentuhnya. Dua kasus yang harus dicek (INV-016) — kepemilikan pindah **atau**
 * approval dicabut/diterbitkan ulang.
 */

import type { NftToken, Sale } from "../types/marketplace.types";

/**
 * `true` bila listing tidak lagi bisa dibeli.
 *
 * `token === null` berarti koleksi menjawab "token tidak ada" → listing pasti basi.
 * Kegagalan RPC **bukan** stale: pemanggil harus membedakannya (lihat `api/market-api.ts`).
 */
export function isSaleStale(sale: Sale, token: NftToken | null, marketContractId: string): boolean {
  if (token === null) return true;
  if (token.ownerId !== sale.ownerId) return true;

  const approvedId = token.approvedAccountIds[marketContractId];
  // `approval_id` kosong di listing = "market cukup ter-approve", sama dengan semantik
  // `nft_is_approved(.., None)` yang dipakai dual verification kontrak.
  if (sale.approvalId === null) return approvedId === undefined;
  return approvedId !== sale.approvalId;
}

/** Listing privat (`allowed_buyer` diisi) tidak pernah masuk grid publik. */
export function isPublicListing(sale: Sale): boolean {
  return sale.allowedBuyer === null;
}

function byTokenIdAsc(a: Sale, b: Sale): number {
  return a.tokenId.localeCompare(b.tokenId);
}

/**
 * Urutan deterministik MVP: `newest` = `listed_at` desc, tie-break `token_id` asc
 * (docs/features/marketplace.md §Discovery). Urutan dari kontrak tidak dijamin (iterasi map),
 * jadi pengurutan dilakukan di sini.
 */
export function compareNewestFirst(a: Sale, b: Sale): number {
  const byTime = b.listedAt - a.listedAt;
  return byTime !== 0 ? byTime : byTokenIdAsc(a, b);
}
