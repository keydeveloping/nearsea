# Frontend Security

> Browser = permukaan yang dilihat user dan tempat semua konten untrusted di-render. Prinsip: FE tidak pernah jadi sumber kebenaran; FE **memastikan** apa yang user lihat = apa yang akan di-sign.

## 1. XSS & konten untrusted

| Sumber untrusted | Aturan |
|---|---|
| Metadata `title/description/extra` | Render sebagai TEKS React (escaped by default); **dilarang** `dangerouslySetInnerHTML`; markdown/HTML dari user tidak pernah di-render sebagai HTML |
| `media` URL | Hanya `<img>`/next-image; gateway allowlist + CSP (metadata-security.md) |
| SVG | Hanya via `<img>`; tidak pernah inline; tidak pernah `<object>/<embed>` |
| Nama koleksi/akun | Teks murni; tanpa auto-linking yang bisa membuka `javascript:` URL |

## 2. CSP & headers (target)

```text
Content-Security-Policy:
  default-src 'self';
  img-src 'self' https://ipfs.dweb.link https://ipfs.io <gateway-allowlist-final>;   <!-- placeholder: final saat TASK-008; kebijakan data: URL = OPEN QUESTION (metadata-security) -->
  script-src 'self' (+ nonce untuk inline framework);
  style-src 'self' 'nonce-<per-request>' (+ nonce untuk inline style Next.js — hindari 'unsafe-inline');
  connect-src 'self' https://rpc.testnet.near.org https://test.rpc.fastnear.com https://free.rpc.fastnear.com https://api.nearblocks.io https://api.<domain>;   <!-- placeholder: <domain> final saat deploy (open-by-design, infrastructure.md); nilai contoh "nearsea.example" -->
  frame-ancestors 'none'; object-src 'none'; base-uri 'self'; form-action 'self'
Strict-Transport-Security: max-age=63072000; includeSubDomains
X-Content-Type-Options: nosniff
X-Frame-Options: DENY (anti-clickjacking)
Referrer-Policy: strict-origin-when-cross-origin
```
> CATATAN: CSP tidak mendukung wildcard hostname tengah (mis. `https://rpc*` TIDAK VALID) — gunakan host eksplisit; wildcard hanya `*.` di awal label.

- Third-party scripts: default TIDAK ADA (tanpa analytics pihak ketiga di MVP — kalau nanti ada: self-host atau strict allowlist + SRI).

## 3. Wallet & transaksi

- **Preview = reality**: argumen tx yang ditampilkan di modal (harga, ownership, receiver, fee, royalti) diambil dari view-call on-chain TERKINI sebelum sign — bukan dari cache (SEC-ORDER-003). (SEC-FE-002)
- Address display: full address + copy button; verified badge hanya dari API server (bukan client heuristic).
- Network guard: tampilkan network (testnet) permanen; transaksi lintas network ditolak di FE + wallet akan gagal sendiri.
- FE tidak pernah: minta seed phrase, menyimpan private key, menandatangani apa pun.

## 4. Phishing resistance

- Domain resmi ditampilkan di footer; logo/nama mirror diklarifikasi di onboarding.
- Warning interstitial untuk link eksternal dari metadata (metadata-security.md).
- Tidak ada deep-link yang langsung mengeksekusi tx tanpa modal konfirmasi.

## 5. Supply chain FE

- Lockfile (yarn.lock) di-commit; Dependabot/audit di CI (cicd-security.md).
- Tanpa CDN runtime script injection; semua bundel dari build kita.

## 6. State & data

- localStorage hanya: wallet connection state, notification read-state — TIDAK ada token jangka panjang (session JWT pendek in-memory + refresh di memory).
- Semua fetch via HTTPS ke allowlist `connect-src`.

## 7. CSP final (string konkret) + nonce

> Target: **CSP ketat** tanpa `'unsafe-inline'` untuk script/style. Host eksplisit (CSP tidak mendukung wildcard hostname tengah — mis. `https://rpc*` TIDAK VALID; wildcard hanya `*.` di awal label). Nilai gateway di bawah **PROPOSED konkret** (selaras [metadata-security.md](./metadata-security.md) §9); `<domain>` tetap placeholder (final saat deploy).

```text
Content-Security-Policy:
  default-src 'self';
  script-src 'self' 'nonce-<per-request>' 'strict-dynamic';
  style-src 'self' 'nonce-<per-request>';
  img-src 'self' https://<project-gateway> https://ipfs.dweb.link https://ipfs.io data:;   <!-- data: = ⏳ OPEN QUESTION, default TOLAK: hapus token data: bila tetap tolak -->
  connect-src 'self' https://rpc.testnet.near.org https://test.rpc.fastnear.com https://free.rpc.fastnear.com https://api.nearblocks.io https://api.<domain>;
  font-src 'self';
  frame-ancestors 'none';
  base-uri 'self';
  form-action 'self';
  object-src 'none';
  worker-src 'self';
  manifest-src 'self';
  upgrade-insecure-requests;
  report-uri /api/v1/csp-report; report-to csp-endpoint
```

| Direktif | Nilai | Alasan |
|---|---|---|
| `script-src` | `'self' 'nonce-…' 'strict-dynamic'` | nonce per-request untuk inline framework; `strict-dynamic` mewarisi kepercayaan ke script yang di-load nonce-holder; **tanpa** `'unsafe-inline'`/`'unsafe-eval'` |
| `style-src` | `'self' 'nonce-…'` | hindari `'unsafe-inline'` (inline style Next.js diberi nonce) |
| `img-src` | `'self'` + gateway allowlist | media hanya dari gateway (§2 metadata-security); `data:` **default tolak** (OPEN QUESTION) |
| `connect-src` | `'self'` + RPC failover + NearBlocks + `api.<domain>` | fetch XHR/fetch/WebSocket; **tidak ada** host arbitrer |
| `frame-ancestors` | `'none'` | anti-clickjacking (melengkapi X-Frame-Options DENY) |
| `object-src` | `'none'` | tidak ada plugin/embed |
| `base-uri` | `'self'` | cegah injeksi `<base>` |
| `form-action` | `'self'` | cegah form hijack |

- **Wildcard**: hanya boleh `https://*.contoh-domain.tld` (label awal). Dilarang `https://rpc*`, `https://*.near.org*`, dsb. Karena itu RPC host ditulis **eksplisit** (bukan `*.fastnear.com`).
- Bila `data:` untuk `img-src` akhirnya **ditolak** (default), hapus token `data:` dari `img-src` (policy minimum).
- Header tambahan (sudah ada di §2): HSTS `max-age=63072000; includeSubDomains`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`.

**Implementasi nonce (detail)**:

```text
1. Edge middleware (Next.js middleware) membangkitkan nonce acak per-request:
   nonce = base64(crypto.randomBytes(16))          # ≥128-bit; sekali pakai per response
2. Nonce di-inject ke header CSP response (script-src/style-src 'nonce-<n>').
3. Next.js meneruskan nonce ke <Script nonce={n}> / inline style yang diizinkan.
4. nonce TIDAK boleh di-cache: response dengan nonce = `Cache-Control: no-store`
   (atau nonce di-generate per response, bukan per build).
5. Nonce ≠ CSRF token; nonce hanya mengizinkan eksekusi script tertentu.
```

- **`'strict-dynamic'`**: script yang punya nonce boleh memuat script lain (bundel Next.js) tanpa perlu didaftarkan; host allowlist script jadi tidak perlu — mengurangi permukaan.
- **Larangan**: jangan fallback ke `'unsafe-inline'`; jangan menyimpan nonce ke localStorage/sessionStorage; jangan pakai nonce statis dari build.
- **Report-only dulu**: uji `Content-Security-Policy-Report-Only` di staging sebelum enforce (lihat §10).

## 8. SRI (Subresource Integrity) — generasi

> Prinsip §5: **tanpa CDN runtime script injection**; semua bundel dari build kita. SRI = pertahanan berlapis bila kelak ada aset dari origin lain.

- **Self-hosted (default)**: bundel di-serve dari origin sendiri (`'self'`) → SRI opsional, tetapi **wajib** untuk aset yang di-load dari host eksternal (bila ada).
- **Generasi**: hash dihitung saat build dari byte final artefak:

```bash
# sha384 (direkomendasikan SRI) → base64
openssl dgst -sha384 -binary dist/app.js | openssl base64 -A
# hasil: sha384-<base64>
```

- **Tag**: `<script src="…" integrity="sha384-…" crossorigin="anonymous">`; aset lintas-origin **wajib** `crossorigin="anonymous"` agar SRI bisa diverifikasi.
- **CI**: job build menghitung hash + memverifikasi tag `integrity` cocok dengan artefak; mismatch → build gagal (SEC-CICD-001).
- **Algoritma**: gunakan **sha384** (standar SRI); sha256 boleh bila perlu. Hash dihitung dari byte **final** (setelah minify), bukan sumber.
- **Third-party**: default tidak ada; bila kelak ditambah → wajib SRI + `crossorigin` + allowlist (kebijakan §9).

## 9. Kebijakan allowlist dependensi

> Supply chain = permukaan utama FE. Kebijakan: **minimal dependensi, terkunci, ter-audit**.

| Aturan | Detail |
|---|---|
| Lockfile | `yarn.lock` (atau setara) **di-commit**; CI pakai `--frozen-lockfile` (build reproducible) |
| Versi | pin exact (tanpa `^`/`~` untuk dependensi kritis); update lewat PR terkontrol |
| Postinstall scripts | tolak/matikan script `postinstall`/`preinstall` dependensi yang tidak perlu (mengurangi eksekusi arbitrer saat install) |
| Audit | Dependabot/`npm audit`/`yarn audit` di CI; kerentanan high/critical → build gagal (SEC-CICD-001) |
| Review dependensi baru | PR yang menambah dependensi wajib alasan + review (ukuran, maintainer, lisensi, riwayat) |
| Duplikasi | hindari paket kecil yang bisa diganti util internal |
| Transitive | audit transitif; lockfile membekukan pohon |
| Lisensi | hanya lisensi permisif yang disetujui; catat di review |

- **Larangan**: CDN runtime script injection; memuat script dari host yang tidak dikontrol; dependensi tanpa lockfile; menonaktifkan audit CI.
- **Allowlist eksplisit** (untuk aset/script eksternal, bukan dependensi npm): kosong di MVP. Bila ada (mis. RPC/font), daftar host di CSP `connect-src`/`font-src` + review.
- Dependensi kritis (wallet selector, near-api-js, framework) diperlakukan sebagai "kode tepercaya" tetapi tetap di-pin + di-audit.

## 10. Desain endpoint CSP-report

> Menerima laporan pelanggaran CSP dari browser. **PROPOSED** (bukan v1 wajib); nilai operasional ⏳ open-by-design.

```text
Endpoint : POST /api/v1/csp-report        (atau path non-/api; final saat implementasi)
Auth     : none (browser mengirim otomatis) — TIDAK ada data sensitif
Content  : application/csp-report | application/reports+json
Body cap : 16 KB (selaras body size cap API)
Rate     : per-IP token bucket (usulan 30/IP/menit) — endpoint publik → tahan abuse
Retensi  : simpan agregat (bukan raw penuh) 30 hari (usulan); raw sampling bila perlu
PII      : JANGAN log header auth/cookie/URL berisi token; strip query sensitif
```

- **Payload** (`report-to` format baru): `{ type:"csp-violation", body:{ documentURL, referrer, blockedURL, effectiveDirective, originalPolicy, disposition, statusCode } }`; format lama `report-uri` = `{ "csp-report": { … } }`. Terima keduanya.
- **Penanganan**: parse ketat → tolak bila bukan JSON/CSP report (415/400); agregasi per `(effectiveDirective, blockedURL host)`; alert bila ada pelanggaran `script-src` yang mencurigakan (sinyal XSS).
- **Isolasi**: endpoint terpisah dari API bisnis (tidak menyentuh DB app); rate limit + body cap.
- **Observability**: metrik `csp_violations_total{directive,host}`; alert untuk lonjakan (indikasi serangan/injeksi) → Telegram ([monitoring.md](../deployment/monitoring.md)).
- **Enforce vs report-only**: jalankan `Report-Only` di staging; setelah bersih → enforce + tetap kirim report.

## 11. Peta test (FE)

| Kontrol | Test | Layer | SEC |
|---|---|---|---|
| CSP header terpasang dengan direktif benar | header scan (curl/CI) — assert `script-src` tanpa `unsafe-inline`, `frame-ancestors 'none'` | integration | SEC-FE-001 |
| nonce per-request unik & script jalan | unit (middleware) + E2E render | unit + E2E | SEC-FE-001 |
| XSS metadata (title/desc) di-escape | unit + E2E dengan payload `<script>` | unit + E2E | SEC-META-001 |
| SVG hanya `<img>`, tanpa inline/object | unit (snapshot) + review | unit | SEC-META-001 |
| Link eksternal `rel` + interstitial | unit + E2E | unit + E2E | SEC-META-001 |
| Preview = on-chain sebelum sign | E2E (TC-022, TC-040) | E2E | SEC-ORDER-003 / SEC-FE-002 |
| SRI cocok artefak | CI build check | CI | SEC-CICD-001 |
| Dependensi terkunci + audit lolos | CI (`--frozen-lockfile` + audit) | CI | SEC-CICD-001 |
| CSP-report endpoint: parse + rate limit | unit + integration | unit | SEC-FE-001 |
| Tanpa third-party script | review + header scan | review | SEC-FE-001 |

- **Aturan**: perubahan CSP/nonce/allowlist wajib disertai update test ini + sinkron ke [metadata-security.md](./metadata-security.md) (gateway) dan [api-security-architecture.md](./api-security-architecture.md) (CORS) — satu sumber kebijakan host.

## Status: PROPOSED — jadi acceptance criteria TASK-008 (FE) + review saat TASK-008b (branding).
