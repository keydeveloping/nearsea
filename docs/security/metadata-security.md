# Metadata Security

> Semua metadata token (`media`, `reference`, `extra`, nama, deskripsi) = **UNTRUSTED** input dari kreator mana pun (platform terbuka). Padanan istilah EVM (ERC-721): `tokenURI/image/animation_url` → NEP-177 `media/extra`.

## 1. Permukaan serangan per fase

| Fase | Siapa memproses metadata | Risiko |
|---|---|---|
| MVP | **Hanya browser** (render `<img src=media>`) — server TIDAK mem-fetch URL apa pun (FACT: tidak ada fitur server-side fetch di MVP → SSRF surface = kosong) | XSS via SVG/HTML, phishing via konten, External link berbahaya |
| Fase 2 (indexer) | Tambah **fetcher server-side** (thumbnail/cache/rekap trait) | + SSRF, decompression bomb, MIME confusion |
| Fase 3 (lazy mint) | Upload pipeline server-side (IPFS pinning) | + file upload abuse |

Field terkait: `media` + `media_hash` (hash konten media) dan `reference` + `reference_hash` (hash JSON metadata tambahan) — **dua pasang berbeda, jangan dicampur**. `external_url` BUKAN field standar NEP-177 (konvensi ekstensi app-level; fallback baca dari `extra`).

## 2. Aturan rendering (MVP, frontend-security.md detail)

- Media HANYA via `<img>` / `<Image>` — **dilarang**: `innerHTML`, `<object>`, `<embed>`, iframe metadata, markdown-to-HTML dari deskripsi.
- SVG: dirender via `<img>` (secure static mode: script dan umumnya resource eksternal diblokir browser — perilaku bisa berbeda antar browser); TIDAK pernah inline `<svg>` dari data user; CSP `img-src` allowlist tetap dipasang sebagai defense-in-depth.
- Gateway IPFS: **allowlist** (`ipfs.dweb.link`, `ipfs.io`, gateway resmi kita — contoh provisional, final = OPEN QUESTION §5) — URL arbitrer ditulis ulang ke gateway allowlist; http:// (non-TLS) ditolak.
- `external_url` & link deskripsi: render sebagai link dengan `rel="noopener noreferrer nofollow"` + interstitial "link eksternal" (phishing mitigation).
- `media_hash` dan `reference_hash`: bila ada, verifikasi client-side hash setelah fetch (integritas konten — dua field berbeda: media_hash = hash media, reference_hash = hash JSON metadata).

## 3. Design fetcher terisolasi (untuk fase 2 — didesain sekarang, dikode nanti)

```text
URL VALIDATION   : protokol allowlist (https, ipfs:// → gateway allowlist); tolak http:, ftp:, data: (kebijakan data: = OPEN QUESTION §5, default tolak)
SSRF GUARDS      : blokir private/loopback/link-local/CGNAT IP (10/8, 172.16/12, 192.168/16, 127/8, 169.254/16, 100.64/10, 0.0.0.0, ::1, fe80::/10, fc00::/7, ::ffff:0:0/96) — praktiknya: tolak semua yang bukan global-unicast;
                   DNS rebinding: resolve → validasi IP → koneksi ke IP yang sama (pin), bukan resolve kedua
LIMITS           : redirect ≤3 (re-validasi tiap hop), timeout 5s, response ≤ 5MB, dekompresi ≤ 25MB (zip-bomb), Content-Type allowlist (image/*, application/json utk metadata)
ISOLATION        : fetcher = process/container terpisah dari API utama (network egress terbatas), tidak punya kredensial DB utama
CACHE            : hasil hash-keyed, TTL; kegagalan = placeholder, bukan error user
```

## 4. Kebijakan konten (linked ke fraud-and-abuse.md)

- Metadata tidak dimoderasi on-chain (FACT: chain immutable); moderasi = display-layer (hide dari discovery) via report system.
- Impersonation check (nama koleksi mirip brand resmi) → proses report + verified badge (ronde 3).

## 5. Status

- MVP rules — DECIDED (diterapkan saat FE dibangun; TASK-008).
- Fetcher design — PROPOSED (fase 2; jadi prerequisite TASK-014 indexer).
- Data-URL policy, daftar gateway final — OPEN QUESTION saat implementasi FE.

## 6. Algoritma hash (eksplisit: sha256)

> Field `media_hash` dan `reference_hash` (NEP-177) = **hash konten**, bukan hash on-chain. Algoritma yang dipakai eksplisit: **SHA-256** (256-bit), direpresentasikan sebagai **base64** (konvensi NEP-177) atau hex — bentuk final ⏳ open-by-design saat FE/fetcher, tetapi **algoritmanya dikunci sha256**.

| Field | Menghash apa | Algoritma | Kapan diverifikasi |
|---|---|---|---|
| `media_hash` | byte mentah **file media** (gambar/video) yang di-fetch dari `media` | **sha256** | client-side (MVP) setelah fetch; fetcher (fase 2) sebelum cache |
| `reference_hash` | byte mentah **file JSON metadata** yang di-fetch dari `reference` | **sha256** | idem |

- **Dua pasang berbeda, jangan dicampur**: `media`+`media_hash` = konten media; `reference`+`reference_hash` = JSON metadata tambahan.
- **Verifikasi**: hitung sha256 dari byte **mentah** respons (sebelum parse/transformasi); bandingkan constant-time dengan nilai di metadata. Mismatch → tampilkan placeholder + tandai "konten tidak terverifikasi" (bukan crash, bukan eksekusi).
- **Hash bukan pengganti CSP**: hash membuktikan integritas konten, bukan bahwa konten aman; rendering tetap `<img>` (tidak pernah inline SVG/HTML).
- Bila hash tidak ada di metadata → tetap render (banyak koleksi pihak ketiga tidak menyertakan hash); ketiadaan hash = "tidak terverifikasi", bukan error.
- **Larangan**: jangan memakai MD5/SHA-1; jangan memverifikasi hash atas hasil re-encode/transformasi (harus byte asli).

## 7. Batas fetcher (eksplisit & terkunci)

> Nilai di §3 dibuat eksplisit di sini agar implementasi fase 2 tidak menafsir ulang. Semua batas **DIPUTUSKAN (dokumen ini)** kecuali ditandai lain; pelanggaran → tolak fetch, bukan retry tanpa batas.

| Kontrol | Nilai | Catatan |
|---|---|---|
| Redirect maksimum | **3 hop** | re-validasi protokol + SSRF guard **setiap hop**; lebih → tolak |
| Timeout koneksi | **5 detik** | per hop; total termasuk redirect |
| Timeout total request | **10 detik** | batas keseluruhan; lebih → abort |
| Ukuran respons (terkompresi) | **≤ 5 MB** | di atas → tolak tanpa membaca penuh (streaming dengan cap) |
| Ukuran hasil dekompresi | **≤ 25 MB** | zip-bomb guard; rasio dekompresi maks 100:1 (PROPOSED) |
| Content-Type allowlist | `image/*`, `application/json` | selain itu tolak (MIME confusion); jangan percaya ekstensi |
| Protokol | `https`, `ipfs://`→gateway | tolak `http:`, `ftp:`, `file:`, `data:` (kebijakan data: = OPEN QUESTION §9, default tolak) |
| IP tujuan | hanya **global-unicast** | tolak private/loopback/link-local/CGNAT (daftar §3); DNS rebinding → resolve lalu pin IP |
| Metode HTTP | hanya `GET`/`HEAD` | tanpa body request |

**Rate limit fetcher** (mencegah fetcher jadi alat DoS keluar / diblokir gateway):

| Bucket | Nilai | Alasan |
|---|---|---|
| Global (seluruh fetcher) | **50 request/detik** | lindungi bandwidth & reputasi IP kita |
| Per-host tujuan | **10 request/detik** | jangan flood satu gateway (diblokir) |
| Per-URL (dedup in-flight) | **1 request** | dedup single-flight; hasil di-cache |
| Cache TTL sukses | **7 hari** (PROPOSED) | konten IPFS immutable → TTL panjang aman |
| Cache TTL gagal | **5 menit** | jangan retry beruntun konten rusak |
| Backoff | eksponensial + jitter saat 429/5xx | hormati `Retry-After` gateway |

- **Circuit breaker** per host: setelah N kegagalan beruntun (PROPOSED: 10) → buka selama 5 menit, lalu half-open.
- Fetcher **tidak punya** kredensial DB utama & tidak memproses input user langsung (proses/container terpisah — §3 ISOLATION).

## 8. Peta test (metadata)

> Metadata security belum punya TC khusus bernomor; bukti = unit test FE/fetcher + E2E. Setiap kontrol §2/§3/§6/§7 wajib punya test.

| Kontrol | Test | Layer | SEC |
|---|---|---|---|
| Media hanya `<img>`, tanpa `innerHTML`/iframe | lint/review komponen + E2E render metadata jahat | unit + E2E | SEC-META-001 |
| SVG tidak pernah inline | unit (snapshot render) + review | unit | SEC-META-001 |
| Gateway allowlist menolak URL arbitrer | unit (URL rewriter) | unit | SEC-META-001 |
| `rel="noopener noreferrer nofollow"` + interstitial link eksternal | unit + E2E | unit + E2E | SEC-META-001 |
| Hash sha256 match/mismatch | unit (fixture byte) | unit | SEC-META-001 |
| SSRF guard (private IP, DNS rebinding, redirect berantai) | unit fetcher (fase 2) | unit | SEC-META-002 |
| Batas dekompresi/timeout/size | unit fetcher | unit | SEC-META-002 |
| Rate limit fetcher + circuit breaker | unit + load (fase 2) | unit | SEC-META-002 |
| E2E jalur emas metadata tampil benar | TC-040, TC-041 | E2E | SEC-META-001 |

- **Aturan**: metadata jahat diuji dengan fixture nyata (SVG ber-script, JSON raksasa, redirect loop) — bukan hanya happy path.

## 9. Allowlist gateway (PROPOSED konkret) & data-URL (OPEN QUESTION)

> Status tetap: **daftar final = OPEN QUESTION** (dikunci saat FE/TASK-008); **kebijakan data: URL = OPEN QUESTION, default TOLAK**. Bagian ini memberi **usulan konkret** agar implementasi tidak menebak — nilai di bawah **PROPOSED**, bukan keputusan produk final.

**Usulan allowlist gateway (urutan prioritas)**:

```text
1. <project-gateway>            gateway milik kita (self-host / pinning service), mis. gateway.nearsea.example
                                → PRIORITAS; semua ipfs:// ditulis ulang ke sini lebih dulu
2. ipfs.dweb.link               gateway publik (dweb.link) — fallback
3. ipfs.io                      gateway publik (Protocol Labs) — fallback
```

- **Tidak masuk allowlist**: `cloudflare-ipfs.com` (deprecated), gateway acak pihak ketiga, subdomain wildcard (`*.ipfs.*`) — CSP tidak mendukung wildcard hostname tengah (lihat frontend-security §2).
- URL `ipfs://<cid>/<path>` **ditulis ulang** ke `https://<gateway>/ipfs/<cid>/<path>`; CID divalidasi bentuknya (CIDv0/CIDv1) sebelum rewrite.
- URL `https://` ke host di luar allowlist untuk field media → **ditolak** (tidak dirender), kecuali host = gateway allowlist. `external_url`/link deskripsi tetap boleh host apa pun karena dirender sebagai **link teks** (bukan dimuat), dengan interstitial + `rel`.
- `http://` (non-TLS) selalu ditolak untuk media (mixed-content + tanpa integritas).

**Kebijakan `data:` URL** (OPEN QUESTION — default **TOLAK**):

| Opsi | Kelebihan | Risiko | Posisi |
|---|---|---|---|
| **Tolak semua `data:`** (default) | permukaan terkecil; tidak ada SVG/HTML inline | sebagian koleksi pihak ketiga pakai data-URI | **default saat ini** |
| Izinkan `data:image/*` (tanpa SVG) + cap ukuran | kompatibilitas | parsing MIME rumit; risiko XSS via `image/svg+xml` | PROPOSED bila diperlukan |
| Izinkan semua `data:` | — | XSS/hTML inline | **ditolak** |

- Bila kelak `data:image/*` diizinkan: **wajib** exclude `image/svg+xml`, cap ukuran (mis. ≤1 MB), dan tetap render via `<img>` (bukan inline) — keputusan final ⏳ open-by-design.
- Perubahan allowlist gateway **wajib** disinkronkan ke CSP `img-src` ([frontend-security.md](./frontend-security.md) §2) dan ke rewriter URL — satu sumber kebijakan.
