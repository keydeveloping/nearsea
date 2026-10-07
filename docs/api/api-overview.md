# API Overview

> API = layanan off-chain ringan di VPS untuk **data app-mutable** (profil/report/badge/admin) dan **auth**.
> Interaksi nilai/fund TIDAK lewat API — lewat kontrak + wallet.
> Keputusan cakupan: **[ADR-004](../decisions/ADR-004-mvp-data-layer.md)** — MVP **tanpa indexer**; discovery dibaca FE langsung dari RPC + NearBlocks.

## Cakupan API per fase (SSOT cakupan)

| Fase | Yang dilayani API | Sumber data |
|---|---|---|
| **MVP** | `auth` (nonce/verify), `accounts/:id/profile` (publik + PATCH), `reports`, `admin/*` (reports/verified/blocklist) | PostgreSQL (app-mutable) |
| **Fase 2** (indexer Neardata) | + `collections` / `tokens` / `listings` / `offers` / `bundles` / `launchpad` (discovery, search, floor/volume/rarity), `accounts/:id/activity` | proyeksi indexer → PostgreSQL |

> **Penting (koreksi kontradiksi):** pada **MVP**, endpoint discovery **belum ada** — FE membaca state chain langsung lewat **view call RPC** dan riwayat lewat **NearBlocks API**. Tabel chain-derived di [database-schema.md](../database/database-schema.md) ditandai ★ (terisi penuh hanya di fase 2). Endpoint discovery di [endpoints.md](./endpoints.md) ditandai **★ fase 2**.

## Prinsip

- API **tidak pernah** menjadi otoritas atas dana/ownership — itu domain chain (ADR-010).
- FE **selalu** re-verify state kritis (harga, ownership, status listing) via RPC view call sebelum transaksi (SEC-ORDER-003).
- API tidak menyimpan kunci, tidak menandatangani transaksi, tidak menyentuh dana.
- Data dari NearBlocks/pihak ketiga = proyeksi yang bisa basi → tidak pernah memicu transaksi.

## Format

- **REST JSON**. Pagination **cursor-based** (`?cursor=…&limit=…`, response berisi `nextCursor`).
- Error envelope konsisten: `{ "error": { "code": "INVALID_…", "message": "…", "requestId": "…" } }` — bentuk & katalog kode dimiliki [development/error-handling.md](../development/error-handling.md) §2–§3 (dokumen ini tidak menyalin registry).
- Semua nilai chain dalam **string yoctoNEAR** (bukan `number`).

## Autentikasi

- Read: publik (untuk endpoint MVP yang ada).
- Write (report/profil/admin): **signature wallet — NEP-413 utama + custom fallback** (challenge nonce sekali pakai, TTL 5 menit) → session token pendek. Template kanonik dimiliki [security/wallet-authentication.md](../security/wallet-authentication.md) §3. Detail: [authentication.md](./authentication.md).

## Versi API (`/v1`) & kebijakan deprecation

- **Semua endpoint berada di bawah prefix versi mayor `/api/v1`.** Di [endpoints.md](./endpoints.md) dan dokumen lain, penulisan ringkas `/api/…` = `/api/v1/…` (shorthand). Base URL lengkap per environment ada di bagian berikutnya.
- Versi mayor ada di **path** (bukan header/query). Perubahan **breaking** (hapus field, ubah tipe, ubah semantik, hapus endpoint) → rilis versi mayor baru (`/api/v2`).
- Perubahan **non-breaking** tidak menaikkan versi: penambahan field opsional, endpoint baru, nilai enum baru yang aditif. Klien **wajib mengabaikan field yang tidak dikenal** (forward-compatible).
- **Deprecation**: endpoint/field yang akan dihapus ditandai `DEPRECATED` minimal **1 MINOR sebelum** dihapus (aturan dimiliki [development/versioning-and-release.md](../development/versioning-and-release.md) §7), dicatat di [CHANGELOG.md](../../CHANGELOG.md) bagian `Deprecated`, dan diberi jalur pengganti. Respons endpoint deprecated menyertakan header `Deprecation` + `Sunset` (format tanggal HTTP).
- **Jendela dukungan versi lama**: minimal 1 MINOR / durasi kalender final → ⏳ open-by-design (ditetapkan saat rilis breaking pertama).
- Versi mayor API (`v1`) **terpisah** dari versi artefak web (`web-vX.Y.Z`) dan dari `version` event NEP-297 (`1.0.0`) — lihat [versioning-and-release.md](../development/versioning-and-release.md) §4.

## Base URL per environment

Host memakai **placeholder** (`nearsea.example`) — domain produksi belum ditetapkan (open-by-design); nilai final diisi saat deploy. Environment & variabel: [deployment/environments.md](../deployment/environments.md).

| Env | Base URL (placeholder) | Sumber chain |
|---|---|---|
| local | `http://localhost:3000/api/v1` | localnet/sandbox |
| dev | `https://api.dev.nearsea.example/api/v1` | RPC testnet (akun `dev.*`) |
| testnet | `https://api.testnet.nearsea.example/api/v1` | RPC testnet |
| mainnet | `https://api.nearsea.example/api/v1` | RPC mainnet |

> Domain `recipient` pada NEP-413 dibaca dari konfigurasi environment, bukan hardcode ([wallet-authentication.md](../security/wallet-authentication.md) §3). Hostname API final ⏳ open-by-design.

## Content-Type, encoding & tipe nilai

| Aturan | Nilai |
|---|---|
| Request `Content-Type` | `application/json; charset=utf-8` (endpoint dengan body) |
| Response `Content-Type` | `application/json; charset=utf-8` |
| Encoding | UTF-8 saja; byte non-UTF-8 → 400 |
| Body size cap | 16 KB per request ([api-security-architecture.md](../security/api-security-architecture.md) §2) |
| Content-Type tak didukung | 415 (kode dari registry `INVALID_*`) |
| Nilai chain | **string yoctoNEAR** (u128) — dilarang `number` (mis. `"1000000000000000000000000"`) |
| Angka non-chain | JSON `number` (mis. `limit`, `phase_index`) |
| Timestamp | ISO-8601 UTC dengan `Z`, mis. `2026-10-02T12:05:00Z` |
| Field `null` | Boleh eksplisit untuk nilai opsional (mis. `allowed_buyer: null`) |
| Penamaan field | Mengikuti contoh kanonik di [endpoints.md](./endpoints.md): field domain `snake_case`, envelope paginasi `nextCursor` |

## Pagination (cursor)

- Query: `?cursor=<opaque>&limit=<int>`.
- `limit` default **20**, maksimum **100** (hard cap; nilai di atas cap di-clamp ke 100, bukan error). Nilai final dapat disetel server; klien tidak boleh mengasumsikan.
- `cursor` bersifat **opaque**: server-side encode dari ordered key (`sort_key` + tie-breaker `id`), base64url. **Klien dilarang men-decode/mengonstruksi cursor.** Cursor invalid/rusak → 400 (kode `INVALID_*`).
- Respons selalu memuat `nextCursor`; `null` = tidak ada halaman berikutnya:
  ```json
  { "items": [ ], "nextCursor": "eyJzIjoibmV3ZXN0IiwiaWQiOiIxMjMifQ" }
  ```
- Cursor **stateless** (tidak disimpan server) dan tidak boleh dipakai lintas `sort`/filter berbeda; kombinasi parameter berbeda → mulai tanpa cursor.
- Urutan tie-breaker wajib deterministik agar tidak ada item terlewat/duplikat saat paginasi.

## Header autentikasi

- `Authorization: Bearer <access_token>` (JWT). Tanpa cookie → CSRF tidak relevan ([wallet-authentication.md](../security/wallet-authentication.md) §4).
- Scope wajib per endpoint; token dengan scope kurang → 403 `FORBIDDEN_SCOPE`.
- Aksi admin destruktif menambah **step-up signature** (per aksi) — lihat [authentication.md](./authentication.md).
- Header `Authorization` malformed/kosong → 401 (kode `AUTH_*`).
- Request ID: klien **boleh** mengirim `X-Request-Id`; server selalu mengembalikan `requestId` di body error (dan header `X-Request-Id`) untuk korelasi log.

## Idempotency-Key

- Header opsional `Idempotency-Key: <UUID v4>` untuk request tidak idempoten (`POST`/`PATCH`). Berguna bila klien retry karena timeout.
- Server menyimpan pemetaan `(account_id, method, path, key) → response` selama **24 jam** (retensi ⏳ open-by-design); replay dengan key sama mengembalikan respons tersimpan + header `Idempotency-Replayed: true`.
- Key sama dengan body berbeda → 409 `CONFLICT_*`.
- **Idempotensi primer `POST /api/reports`** tetap **unik per (account, target, hari kalender)** di DB ([api-security-architecture.md](../security/api-security-architecture.md) §1) — header ini hanya lapisan tambahan, bukan pengganti constraint DB.
- Request tanpa `Idempotency-Key` yang di-retry **tidak** dijamin idempoten.

## Caching & ETag

| Kategori | `Cache-Control` | ETag |
|---|---|---|
| `GET` publik (profil, discovery fase 2) | `public, max-age=30, stale-while-revalidate=60` | Ya — ETag kuat dari hash body; `If-None-Match` cocok → **304** tanpa body |
| `GET` terautentikasi (mis. `accounts/me/profile`) | `private, no-store` | Tidak |
| Admin / report queue | `no-store` | Tidak |
| `POST`/`PATCH`/`DELETE` | `no-store` | Tidak |

- Nilai `max-age` final ⏳ open-by-design (ditetapkan saat implementasi per endpoint).
- ETag harus berubah bila body berubah; `304` **tidak** mengirim body.

## CORS

- **Allowlist origin eksplisit** per environment (domain FE tiap env) — tidak ada wildcard `*` ([api-security-architecture.md](../security/api-security-architecture.md) §4).
- `Access-Control-Allow-Methods`: `GET, POST, PATCH, DELETE, OPTIONS`.
- `Access-Control-Allow-Headers`: `Authorization, Content-Type, Idempotency-Key, X-Request-Id, If-None-Match` (daftar header kanonik ada di [api-security-architecture.md](../security/api-security-architecture.md)).
- `Access-Control-Allow-Credentials`: **false** (auth via header, bukan cookie).
- Preflight `OPTIONS`: `Access-Control-Max-Age: 600` (⏳ final saat implementasi).
- Origin di luar allowlist → tidak ada header CORS (browser memblokir); server tetap memproses non-CORS request sesuai auth.

## Rencana OpenAPI

- **Rencana**: menyediakan spesifikasi **OpenAPI 3.1** sebagai artefak mesin (`docs/api/openapi.yaml`) yang di-generate/di-validasi terhadap [endpoints.md](./endpoints.md). File spec belum ada → ⏳ open-by-design (dibuat saat implementasi; lihat backlog).
- [endpoints.md](./endpoints.md) tetap **SSOT manusia**; OpenAPI = turunan mesin, dipakai untuk validasi request/response, contract test, dan (opsional) generate klien.
- Bila OpenAPI dan endpoints.md berbeda → endpoints.md yang benar sampai spec diperbaiki.

## Changelog API

> Perubahan kontrak API (bukan kontrak on-chain). Format Keep a Changelog; versi artefak & tag di [versioning-and-release.md](../development/versioning-and-release.md), catatan rilis menyeluruh di [CHANGELOG.md](../../CHANGELOG.md).

| Tanggal | Versi | Perubahan | Breaking? |
|---|---|---|---|
| 2026-10-02 | v1 (pra-rilis) | Definisi awal `/api/v1` (auth, profil, reports, admin/*); discovery ditandai ★ fase 2 | — |

- Entri baru ditambahkan **saat rilis**; perubahan breaking wajib menaikkan versi mayor path.

## Status

- Prefix `/v1`, base URL per env, pagination cursor, idempotency, CORS, ETag, rencana OpenAPI — **DECIDED (dokumen ini)**; nilai operasional yang belum final ditandai `⏳ open-by-design` (hostname, sunset window, cache TTL, retensi idempotency, file OpenAPI).
- Registry kode error tetap dimiliki [error-handling.md](../development/error-handling.md) — kode baru ditambahkan di sana sebelum dipakai.
