/**
 * Media metadata = **konten tidak tepercaya** (docs/security/frontend-security.md §1):
 * hanya boleh dirender lewat `<img>` — tidak pernah `innerHTML`, tidak pernah inline SVG.
 *
 * Hanya `https:` yang diterima: `data:`/`blob:`/`javascript:` dari metadata pihak ketiga tidak
 * punya alasan untuk dirender, dan menerimanya melebarkan permukaan serangan halaman.
 * Allowlist host final menyusul bersama CSP (`metadata-security.md`); sampai itu ada, skema
 * adalah batas yang bisa ditegakkan di sini.
 */
export function safeMediaUrl(raw: string | null): string | null {
  if (raw === null) return null;

  try {
    const url = new URL(raw);
    return url.protocol === "https:" ? url.toString() : null;
  } catch {
    return null;
  }
}
