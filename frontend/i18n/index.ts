import common from "./en/common.json";

const messages = { common } as const;

type Namespace = keyof typeof messages;
type MessageKeyOf<N extends Namespace> = `${N}.${keyof (typeof messages)[N] & string}`;

export type MessageKey = { [N in Namespace]: MessageKeyOf<N> }[Namespace];

/**
 * Copy UI selalu lewat modul ini (tidak ada string hardcode di komponen).
 * Kunci bertipe: kunci yang salah = error build (code-standards.md §10).
 */
export function t(key: MessageKey): string {
  const separator = key.indexOf(".");
  const namespace = key.slice(0, separator) as Namespace;
  const name = key.slice(separator + 1);
  const value: string | undefined = messages[namespace][name as never];

  if (value === undefined) {
    throw new Error(`Missing i18n message: ${key}`);
  }

  return value;
}

/** Substitusi placeholder bernama: `format(t("common.networkBanner"), { network: "testnet" })`. */
export function format(template: string, values: Record<string, string>): string {
  return template.replace(/\{(\w+)\}/g, (match, name: string) => values[name] ?? match);
}
