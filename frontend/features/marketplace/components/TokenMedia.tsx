"use client";

import { format, t } from "@/i18n";
import { safeMediaUrl } from "@/lib/format/media";

/**
 * Media token dari metadata pihak ketiga. Dirender lewat `<img>` saja — tidak ada `innerHTML`,
 * tidak ada SVG inline (docs/security/frontend-security.md §1). URL yang bukan `https:` ditolak
 * dan digantikan placeholder teks, jadi metadata jahat tidak bisa menyuntikkan apa pun.
 */
export function TokenMedia({
  media,
  name,
  className,
}: {
  media: string | null;
  name: string;
  className?: string;
}) {
  const url = safeMediaUrl(media);

  if (url === null) {
    return (
      <div
        aria-hidden="true"
        className={`flex items-center justify-center bg-zinc-100 font-mono text-zinc-400 dark:bg-zinc-900 ${className ?? ""}`}
      >
        {name}
      </div>
    );
  }

  return (
    // `<img>` disengaja: media adalah konten pihak ketiga dan hanya boleh dirender lewat
    // `<img>` (frontend-security.md §1). `next/image` menuntut allowlist host yang belum final.
    // eslint-disable-next-line @next/next/no-img-element
    <img
      src={url}
      alt={format(t("marketplace.detail.mediaAlt"), { name })}
      className={`bg-zinc-100 object-cover dark:bg-zinc-900 ${className ?? ""}`}
    />
  );
}
