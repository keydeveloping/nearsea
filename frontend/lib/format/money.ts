/**
 * Konversi nilai Ⓝ. Aritmetika **selalu BigInt** — dilarang float pada yoctoNEAR
 * (AGENTS.md §Blockchain Rules, frontend-architecture.md §Konversi & format).
 */

export type YoctoNear = string & { readonly __brand: "YoctoNear" };

const YOCTO_PER_NEAR = 10n ** 24n;
const YOCTO_DECIMAL_DIGITS = 24;

export function asYoctoNear(value: string): YoctoNear {
  if (!/^\d+$/.test(value)) {
    throw new Error(`Invalid yoctoNEAR amount: "${value}"`);
  }
  return value as YoctoNear;
}

/**
 * Format tampilan Ⓝ dari string yoctoNEAR. Pecahan **dipotong** (bukan dibulatkan)
 * supaya tidak pernah melebih-lebihkan saldo.
 */
export function formatNear(value: YoctoNear, maxFractionDigits = 4): string {
  const yocto = BigInt(value);
  const whole = yocto / YOCTO_PER_NEAR;
  const fraction = yocto % YOCTO_PER_NEAR;

  if (fraction === 0n || maxFractionDigits <= 0) return whole.toString();

  const digits = fraction
    .toString()
    .padStart(YOCTO_DECIMAL_DIGITS, "0")
    .slice(0, maxFractionDigits)
    .replace(/0+$/, "");

  return digits.length === 0 ? whole.toString() : `${whole}.${digits}`;
}
