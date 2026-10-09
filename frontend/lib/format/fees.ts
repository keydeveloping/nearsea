/**
 * Breakdown pembayaran sale: fee platform + royalti kreator + sisa ke seller.
 *
 * Rumus = SSOT docs/features/payments.md §Fee & Royalti dan docs/contracts/market.md §3a:
 * aritmetika **integer u128 dengan pembulatan ke bawah**, dilarang float (AGENTS.md
 * §Blockchain Rules). Nilai yang ditampilkan di sini adalah **estimasi** — jumlah otoritatif
 * adalah hasil `nft_transfer_payout` saat settlement, dan market memvalidasi ulang payout itu.
 */

import { asYoctoNear, type YoctoNear } from "./money";

/** Basis poin: 10_000 bps = 100%. Sama dengan konstanta kontrak (market `FEE_BPS_DEFAULT`). */
export const BPS_DENOMINATOR = 10_000;

export interface SaleBreakdown {
  price: YoctoNear;
  /** Fee platform, dipotong **dari** harga (bukan ditambahkan) — payments.md §Fee. */
  fee: YoctoNear;
  royalty: YoctoNear;
  /** Residual: yang benar-benar diterima seller. */
  sellerProceeds: YoctoNear;
  feeBps: number;
  royaltyBps: number;
}

/** Bagian `bps` dari `amount`, dibulatkan **ke bawah** supaya tidak pernah melebihkan. */
export function bpsOf(amount: YoctoNear, bps: number): YoctoNear {
  if (!Number.isInteger(bps) || bps < 0 || bps > BPS_DENOMINATOR) {
    throw new Error(`Invalid basis points: ${bps}`);
  }
  return asYoctoNear(((BigInt(amount) * BigInt(bps)) / BigInt(BPS_DENOMINATOR)).toString());
}

/**
 * Basis poin → persen untuk ditampilkan (`200` → `"2"`, `250` → `"2.5"`).
 *
 * Aritmetika integer: tidak ada float yang bisa mengubah angka yang ditampilkan ke user.
 */
export function percentFromBps(bps: number): string {
  if (!Number.isInteger(bps) || bps < 0) throw new Error(`Invalid basis points: ${bps}`);
  const whole = Math.trunc(bps / 100);
  const fraction = (bps % 100).toString().padStart(2, "0").replace(/0+$/, "");
  return fraction.length === 0 ? whole.toString() : `${whole}.${fraction}`;
}

/**
 * `null` bila angkanya tidak bisa dipercaya: bps di luar rentang, royalti tidak diketahui, atau
 * fee + royalti melebihi harga. Kontrak menolak payout seperti itu, dan menampilkan sisa seller
 * seolah royalti nol akan **melebih-lebihkan** penerimaan seller — lebih baik jujur "estimasi
 * tidak tersedia" daripada menampilkan angka palsu.
 */
export function computeSaleBreakdown(
  price: YoctoNear,
  feeBps: number,
  royaltyBps: number | null,
): SaleBreakdown | null {
  if (royaltyBps === null) return null;
  if (!Number.isInteger(feeBps) || !Number.isInteger(royaltyBps)) return null;
  if (feeBps < 0 || royaltyBps < 0) return null;
  if (feeBps > BPS_DENOMINATOR || royaltyBps > BPS_DENOMINATOR) return null;

  const priceValue = BigInt(price);
  const fee = bpsOf(price, feeBps);
  const royalty = bpsOf(price, royaltyBps);
  const claimed = BigInt(fee) + BigInt(royalty);
  if (claimed > priceValue) return null;

  return {
    price,
    fee,
    royalty,
    sellerProceeds: asYoctoNear((priceValue - claimed).toString()),
    feeBps,
    royaltyBps,
  };
}
