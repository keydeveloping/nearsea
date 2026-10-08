import auth from "./en/auth.json";
import common from "./en/common.json";

const messages = { auth, common } as const;

/** Semua jalur kunci bertitik, dihitung dari bentuk pesan (kunci salah = error build). */
type DottedPaths<T, Prefix extends string = ""> = {
  [K in keyof T & string]: T[K] extends Record<string, unknown>
    ? DottedPaths<T[K], `${Prefix}${K}.`>
    : `${Prefix}${K}`;
}[keyof T & string];

export type MessageKey = DottedPaths<typeof messages>;

function lookup(key: MessageKey): string | undefined {
  // Satu-satunya cast: TypeScript tidak bisa mengindeks pohon bertipe dengan
  // segmen kunci yang sudah dipisah runtime. `MessageKey` tetap yang menjaga pemanggil.
  let node: unknown = messages;
  for (const segment of key.split(".")) {
    if (typeof node !== "object" || node === null) return undefined;
    node = (node as Record<string, unknown>)[segment];
  }
  return typeof node === "string" ? node : undefined;
}

/**
 * Copy UI selalu lewat modul ini (tidak ada string hardcode di komponen).
 * Kunci bertipe: kunci yang salah = error build (code-standards.md §10).
 */
export function t(key: MessageKey): string {
  const value = lookup(key);

  if (value === undefined) {
    throw new Error(`Missing i18n message: ${key}`);
  }

  return value;
}

/** Substitusi placeholder bernama: `format(t("common.networkBanner"), { network: "testnet" })`. */
export function format(template: string, values: Record<string, string>): string {
  return template.replace(/\{(\w+)\}/g, (match, name: string) => values[name] ?? match);
}
