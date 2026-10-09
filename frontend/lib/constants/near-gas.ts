/**
 * Anggaran gas untuk call lintas-kontrak (docs/features/marketplace.md §Anggaran Gas).
 *
 * Nilai = **batas atas yang dipasang**, bukan pemakaian terukur: gas yang tidak terpakai
 * dikembalikan. Dipasang lega supaya rantai callback (dual verification → callback →
 * `nft_transfer_payout` → resolve) tidak kehabisan budget di tengah.
 */

/** 1 Tgas = 10^12 gas. */
const TGA = 1_000_000_000_000n;

function gas(tgas: number): string {
  return (BigInt(tgas) * TGA).toString();
}

/** Tx-1 listing: `nft_approve` di kontrak koleksi (~10–15 Tgas terukur). */
export const GAS_APPROVE = gas(30);

/** Tx-2 listing: `list_nft_for_sale` + 2 view + callback `process_listing`. */
export const GAS_LIST = gas(40);

/** `buy`: proses purchase + `nft_transfer_payout` + resolve (~135 Tgas terukur). */
export const GAS_BUY = gas(200);

/** Deposit 1 yocto untuk method NEP-178 yang memakai `assert_nonzero_deposit`. */
export const ONE_YOCTO = "1";
