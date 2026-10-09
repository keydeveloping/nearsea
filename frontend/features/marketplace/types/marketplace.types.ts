/**
 * Tipe data marketplace seperti yang ter-serialisasi dari kontrak (docs/contracts/market.md §5).
 *
 * Nilai Ⓝ **selalu** string yoctoNEAR (`U128` → string di JSON) — dilarang `number`
 * (AGENTS.md §Blockchain Rules). Nama field di sini camelCase; pemetaan dari snake_case
 * kontrak dilakukan sekali di `api/market-api.ts`.
 */

import type { YoctoNear } from "@/lib/format/money";

/** Satu entry `Sale` — baris listing aktif. */
export interface Sale {
  nftContractId: string;
  tokenId: string;
  /** Seller yang memasang listing (bukan pemilik saat ini bila listing sudah stale). */
  ownerId: string;
  /** `null` = listing dibuat tanpa id approval konkret (lihat catatan di `api/market-api.ts`). */
  approvalId: number | null;
  priceYocto: YoctoNear;
  /** Non-null = listing privat: tidak boleh tampil di grid publik. */
  allowedBuyer: string | null;
  /** u64 nanodetik dari `env::block_timestamp()`. */
  listedAt: number;
}

/** Token dari `nft_token` (NEP-171, metadata ter-flatten ke objek yang sama). */
export interface NftToken {
  tokenId: string;
  ownerId: string;
  /** Peta `account_id → approval_id` (NEP-178). */
  approvedAccountIds: Record<string, number>;
  title: string | null;
  description: string | null;
  media: string | null;
}

/** `royalty_config()` koleksi — royalti bersifat **kontrak-level**, bukan per token. */
export interface RoyaltyConfig {
  receiverId: string;
  bps: number;
}

/** `storage_balance_bounds()` (NEP-145). */
export interface StorageBounds {
  minYocto: YoctoNear;
  maxYocto: YoctoNear | null;
}

/** Kemampuan chain yang dibutuhkan fitur marketplace — **interface sempit**, bukan `WalletApi`.
 *
 * Fitur tidak boleh saling impor (frontend-architecture.md §1), jadi wallet disuntikkan dari
 * `app/` (yang boleh impor `features/`) dan marketplace hanya bergantung pada bentuk ini.
 * `WalletApi` memenuhinya secara struktural, jadi tidak ada perekat tambahan di `app/`.
 */
export interface MarketChain {
  accountId: string | null;
  transactionsDisabled: boolean;
  viewFunction: (params: {
    contractId: string;
    method: string;
    args?: Record<string, unknown>;
  }) => Promise<unknown>;
  callFunction: (params: {
    contractId: string;
    method: string;
    args?: Record<string, unknown>;
    gas?: string;
    deposit?: string;
  }) => Promise<unknown>;
}
