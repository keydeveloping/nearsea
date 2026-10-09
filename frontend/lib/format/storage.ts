/**
 * Aritmetika storage NEP-145 — user membayar storage-nya sendiri, kontrak tidak pernah
 * menanggung (AGENTS.md §Blockchain Rules, docs/features/marketplace.md §Storage Deposit).
 *
 * Semua nilai = string yoctoNEAR; aritmetika BigInt, tanpa float.
 */

import { asYoctoNear, type YoctoNear } from "./money";

/**
 * Kekurangan saldo storage yang harus dilampirkan supaya listing diterima.
 *
 * Kontrak menagih entry `Sale` dari saldo storage seller; deposit yang **tidak** kurang
 * (mis. seller sudah punya saldo cukup) dilaporkan `0`, bukan angka negatif — kontrak
 * memakai `saturating` semantics yang sama.
 */
export function storageShortfall(minYocto: YoctoNear, availableYocto: YoctoNear): YoctoNear {
  const min = BigInt(minYocto);
  const available = BigInt(availableYocto);
  return asYoctoNear((available >= min ? 0n : min - available).toString());
}
