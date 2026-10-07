import common from "./en/common.json";

const messages = { common } as const;

type Namespace = keyof typeof messages;

export type MessageKey = {
  [N in Namespace]: `${N & string}.${keyof (typeof messages)[N] & string}`;
}[Namespace];

function lookup(key: MessageKey): string | undefined {
  const separator = key.indexOf(".");
  const namespace = key.slice(0, separator);
  const name = key.slice(separator + 1);
  // Satu-satunya cast: TypeScript tidak bisa memecah template literal `a.b`
  // menjadi dua literal terpisah. `MessageKey` tetap yang menjaga pemanggil.
  const namespaceMessages: Record<string, string> = messages[namespace as Namespace];
  return namespaceMessages[name];
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
