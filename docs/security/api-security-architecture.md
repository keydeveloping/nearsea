# API Security Architecture

> Model keamanan per kategori endpoint (Report API di VPS). Prinsip: **API tidak dipercayai dari frontend** — semua otorisasi di server; API tidak pernah menyentuh dana (FACT: settlement hanya kontrak).

## 1. Matriks per kategori endpoint

| Kategori | AuthN | AuthZ | Input validation | Rate limit | Idempotency | Audit |
|---|---|---|---|---|---|---|
| `GET /api/auth/nonce` | — | — | account_id format | 10/IP/menit + 30/akun/jam | nonce unik | percobaan gagal login |
| `POST /api/auth/verify` | signature — **NEP-413 utama + custom fallback** (wallet-authentication.md) | — | strict schema | 10/IP/menit | konsumsi nonce atomic | semua percobaan (gagal & sukses) |
| `GET` discovery (collections/tokens/listings/offers) | publik | — | query params whitelist + batas | 60/IP/menit | cursor pagination | — |
| `GET /accounts/:id/activity` | publik | — | account_id format | 30/IP/menit | — | — |
| `POST /api/reports` | session scope `report` | account terverifikasi | enum reason, panjang, target exists | 5/akun/hari | idempotency = (account, target, hari kalender — konsisten dengan UNIQUE DB) | log |
| `PATCH /api/profile` | session scope `profile` | hanya `sub` sendiri | alias ≤32, bio ≤280, avatar URL allowlist | 10/akun/jam | last-write | log |
| `POST\|PATCH /api/admin/*` | session scope `admin` + step-up signature per aksi | allowlist DB | strict schema | 30/menit per akun admin | decision id | `admin_audit` wajib |

## 2. Pertahanan eksplisit (OWASP API top-10, adaptasi)

| Ancaman | Pertahanan |
|---|---|
| **BOLA** (akses data orang lain) | Semua resource scoped: session `sub` = pemilik; admin ops lewat allowlist — TIDAK ada endpoint "ambil by id" yang membaca milik orang lain; data chain memang publik (FACT) sehingga BOLA surface kecil |
| Broken function authorization | Method-level guard middleware + unit test authz per endpoint (SEC-API-002) |
| Privilege escalation | Scope token (`report|profile|admin`) dipisah; step-up untuk destructive; allowlist cek di server bukan di token claim saja |
| Injection (SQL) | ORM/parameterized only (Prisma); tidak ada raw string concat |
| SSRF | API TIDAK mem-fetch URL eksternal di MVP (FACT — tidak ada fitur server-side fetch); avatar URL hanya disimpan + dirender FE via gateway allowlist; fase 2 fetcher → metadata-security.md |
| Resource exhaustion | Rate limit + pagination cursor + batas response size + body size limit (mis. 16KB) |
| Excessive data exposure | Response schema eksplisit; tidak ada field internal; tidak ada PII (wallet-only) |
| Race conditions | Nonce consume atomic (DB), idempotency keys; tidak ada alur "check-then-act" untuk hal berbayar (bayar = kontrak) |

## 3. Error handling & output

- Error envelope konsisten `{error:{code,message}}`; pesan generik ke client (detil di log).
- Stack trace TIDAK pernah ke response; log server-side + alert Telegram untuk 5xx.

## 4. Sesi & header

- JWT TTL 15 menit + refresh 12 jam rotasi (SEC-AUTH-006); revoke = denylist server.
- Security headers (frontend-security.md juga berlaku ke API): HSTS, X-Content-Type-Options, X-Frame-Options DENY, CORS allowlist domain.

## 5. Yang TIDAK dilakukan API (by design)

- Tidak memegang kunci/secret yang berlaku on-chain.
- Tidak memvalidasi/memproses pembayaran (bypass API langsung kontrak).
- Tidak jadi sumber kebenaran kepemilikan (proyeksi saja).

## 6. Inventaris endpoint lengkap (MVP + ★ fase 2)

> Daftar otoritatif path & cakupan dimiliki [endpoints.md](../api/endpoints.md) + [api-overview.md](../api/api-overview.md); tabel ini = ringkasan keamanan (auth + rate) untuk audit. Penulisan ringkas `/api/…` = `/api/v1/…`. Semua endpoint tunduk **body size cap 16 KB** + batas ukuran respons.

### 6a. MVP

| Method | Path | AuthN | AuthZ | Rate limit | Idempotency |
|---|---|---|---|---|---|
| `GET` | `/api/v1/auth/nonce` | none | — | 10/IP/menit + 30/akun/jam | nonce unik |
| `POST` | `/api/v1/auth/verify` | signature (NEP-413 utama + fallback) | — | 10/IP/menit; 5 gagal/menit → lockout | konsumsi nonce atomik |
| `POST` | `/api/v1/auth/refresh` | refresh token | — | ⏳ (usulan 20/IP/menit) | rotasi refresh |
| `POST` | `/api/v1/auth/logout` | Bearer | scope apa pun | ⏳ | idempoten |
| `GET` | `/api/v1/accounts/:id/profile` | none | — | 60/IP/menit | ETag/304 |
| `PATCH` | `/api/v1/accounts/me/profile` | Bearer | scope `profile`; hanya `sub` sendiri | 10/akun/jam | last-write |
| `POST` | `/api/v1/reports` | Bearer | scope `report` | 5/akun/hari | unik `(reporter, target, hari)` |
| `GET` | `/api/v1/admin/reports` | Bearer | scope `admin` + allowlist DB | 30/menit/akun admin | — |
| `PATCH` | `/api/v1/admin/reports/:id` | Bearer + step-up | scope `admin` + allowlist + step-up | 30/menit/akun admin | decision id |
| `PATCH` | `/api/v1/admin/collections/:id/verified` | Bearer + step-up | scope `admin` + allowlist + step-up | 30/menit/akun admin | last-write |
| `GET` | `/api/v1/admin/blocklist` | Bearer | scope `admin` + allowlist | 30/menit/akun admin | — |
| `POST` | `/api/v1/admin/blocklist` | Bearer + step-up | scope `admin` + allowlist + step-up | 30/menit/akun admin | unik `(target_type, target_id)` |
| `DELETE` | `/api/v1/admin/blocklist/:id` | Bearer + step-up | scope `admin` + allowlist + step-up | 30/menit/akun admin | idempoten |
| `GET` | `/api/health` | none | — | — | — (health check operasional; bukan bagian versi `/api/v1`) |

### 6b. ★ FASE 2 (butuh indexer — belum ada di MVP)

| Method | Path | AuthN | Rate limit | Catatan |
|---|---|---|---|---|
| `GET` | `/api/v1/collections` | none | 60/IP/menit | publik, cursor pagination |
| `GET` | `/api/v1/collections/:id` | none | 60/IP/menit | publik |
| `GET` | `/api/v1/tokens` | none | 60/IP/menit | publik |
| `GET` | `/api/v1/tokens/:contract/:tokenId` | none | 60/IP/menit | publik |
| `GET` | `/api/v1/listings` | none | 60/IP/menit | publik |
| `GET` | `/api/v1/offers` | none | 60/IP/menit | `account` wajib |
| `GET` | `/api/v1/bundles` | none | 60/IP/menit | publik |
| `GET` | `/api/v1/bundles/:bundleId` | none | 60/IP/menit | publik |
| `GET` | `/api/v1/launchpad/:collection` | none | 60/IP/menit | publik |
| `GET` | `/api/v1/accounts/:id/activity` | none | 30/IP/menit | proxy/cache NearBlocks |
| `GET` | `/api/v1/search` | none | 60/IP/menit | `q` min 2 char |

- **Tidak ada** endpoint "ambil resource milik orang lain by id" (BOLA surface kecil — data chain publik, FACT).
- Endpoint admin = allowlist DB **di server**, bukan sekadar klaim scope di token (defense-in-depth).
- Endpoint baru wajib masuk inventaris ini + [endpoints.md](../api/endpoints.md) + rate limit eksplisit (SEC-API-001).

## 7. Pointer skema request/response

> Skema field-level **tidak diduplikasi** di sini (single source of truth). Dokumen ini hanya menunjuk.

| Endpoint | Skema lengkap |
|---|---|
| Auth (nonce/verify/refresh/logout) | [endpoints.md](../api/endpoints.md) § Endpoint MVP — Auth; [authentication.md](../api/authentication.md) |
| Profil | [endpoints.md](../api/endpoints.md) § Endpoint MVP — Profil |
| Reports + admin | [endpoints.md](../api/endpoints.md) § Endpoint MVP — Reports / Admin |
| Discovery ★ fase 2 | [endpoints.md](../api/endpoints.md) § Endpoint ★ FASE 2 |
| Envelope error | [error-handling.md](../development/error-handling.md) §2 |
| Konvensi lintas-endpoint (pagination, caching, idempotency, versi) | [api-overview.md](../api/api-overview.md) |

- Aturan nilai: semua uang = **string yoctoNEAR**; timestamp ISO-8601 UTC; `limit` default 20 maks 100; `cursor` opaque.
- Respons daftar selalu `{ items[], nextCursor }`; error selalu `{ error: { code, message, requestId } }`.

## 8. Matriks test authz per endpoint (SEC-API-002)

> Setiap endpoint **wajib** punya test otorisasi: happy path + jalur ditolak (401/403). Ini menutup **SEC-API-002** (broken authorization / BOLA). Test = integration test API; kolom "TC" merujuk [test-cases.md](../testing/test-cases.md) bila sudah ada.

| Endpoint | Happy | Tanpa auth | Scope salah | Bukan pemilik/allowlist | TC |
|---|---|---|---|---|---|
| `GET /auth/nonce` | 200 | n/a | n/a | n/a | TC-027 |
| `POST /auth/verify` | 200 | 401 | n/a | 401 (key bukan milik akun) | TC-028 |
| `POST /auth/refresh` | 200 | 401 | 403 (scope admin) | 401 (reuse/expired) | TC-029 |
| `POST /auth/logout` | 204 | 401 | n/a | n/a | TC-030 |
| `GET /accounts/:id/profile` | 200 | n/a | n/a | 404 (tidak ada) | TC-031 |
| `PATCH /accounts/me/profile` | 200 | 401 | 403 `FORBIDDEN_SCOPE` | — (selalu `sub` sendiri) | TC-032 |
| `POST /reports` | 201 | 401 | 403 `FORBIDDEN_SCOPE` | 404 target | TC-033 |
| `GET /admin/reports` | 200 | 401 | — | 403 `FORBIDDEN_ADMIN` | TC-034 |
| `PATCH /admin/reports/:id` | 200 | 401 | — | 403 (di luar allowlist / tanpa step-up) | TC-035, TC-026 |
| `PATCH /admin/collections/:id/verified` | 200 | 401 | — | 403 | TC-036 |
| `GET /admin/blocklist` | 200 | 401 | — | 403 | TC-037 |
| `POST /admin/blocklist` | 201 | 401 | — | 403 / 409 | TC-038 |
| `DELETE /admin/blocklist/:id` | 204 | 401 | — | 403 / 404 | TC-039 |

- **Aturan**: endpoint baru tanpa baris di matriks ini = tidak boleh merge (SEC-API-002).
- Test menegakkan **server-side** (bukan mengandalkan FE menyembunyikan tombol); scope di token **tidak cukup** untuk admin (allowlist DB dicek ulang).
- BOLA: karena tidak ada endpoint baca milik orang lain, test fokus pada **function-level authorization** + scope.

## 9. Algoritma rate limit (token bucket)

> **DIPUTUSKAN (dokumen ini)**: rate limit memakai **token bucket** per kunci. Alasan: mengizinkan burst pendek namun membatasi laju rata-rata; sederhana & deterministik; dapat disimpan di Redis/in-memory (MVP satu instance).

```text
ALGORITMA: Token Bucket (single bucket per rate-key)

Parameter per aturan rate limit:
  capacity   = burst maksimum (token)
  refill_rate= token/detik  (rate / 60)
  cost       = 1 token per request (kecuali dinyatakan lain)

State per rate-key: { tokens: float, last_refill: timestamp }

function allow(key, now):
  st = load(key)                       # atomic (Redis INCR/Lua atau lock in-memory)
  elapsed = now − st.last_refill
  st.tokens = min(capacity, st.tokens + elapsed * refill_rate)
  st.last_refill = now
  if st.tokens >= cost:
    st.tokens -= cost
    store(key, st)
    return ALLOW
  else:
    retry_after = ceil((cost − st.tokens) / refill_rate)
    store(key, st)
    return DENY(retry_after)
```

**Rate-key** (urutan spesifisitas): `account_id` (bila terautentikasi) → `ip` → kombinasi. Endpoint auth memakai **dua bucket** (IP + akun) — penyerang harus kalah di keduanya.

| Aturan (contoh) | capacity (burst) | refill_rate | Setara |
|---|---|---|---|
| `10/IP/menit` (auth) | 10 | 10/60 ≈ 0.167/s | 10/menit rata-rata, burst ≤10 |
| `30/akun/jam` (nonce) | 5 | 30/3600 ≈ 0.0083/s | burst kecil, jendela panjang |
| `60/IP/menit` (discovery ★) | 20 | 1/s | burst 20, lalu 60/menit |
| `5/akun/hari` (reports) | 5 | 5/86400 ≈ 5.8e-5/s | hampir hard cap |
| `30/menit` (admin) | 10 | 0.5/s | burst 10 |

- **Lockout**: `POST /auth/verify` → 5 gagal/menit memicu lockout sementara (bucket gagal terpisah, `capacity=5`, refill `5/60`); sukses tidak menambah bucket gagal.
- **Respons 429** selalu memuat header `Retry-After` (detik) + `error.code` dari namespace `RATE_*`; tidak membocorkan sisa token internal.
- **Fail-open vs fail-closed**: bila penyimpanan rate-limit down → endpoint auth **fail-closed** (tolak 503), endpoint read boleh **fail-open** (rate limit bukan kontrol dana). Kebijakan final ⏳ saat implementasi.
- Distribusi multi-instance (fase scaling): bucket di store bersama (Redis) — lihat [scaling.md](../architecture/scaling.md).

## 10. Katalog kode error (pointer)

> Registry kode error **tidak diduplikasi** di sini. Sumber tunggal: [error-handling.md](../development/error-handling.md) §3 (namespace `INVALID_*`/`AUTH_*`/`FORBIDDEN_*`/`NOT_FOUND_*`/`CONFLICT_*`/`RATE_*`/`CHAIN_*`/`SERVER_*`).

- Envelope error: `{ "error": { "code", "message", "requestId" } }` ([error-handling.md](../development/error-handling.md) §2).
- Pemetaan panic kontrak → kode user: [error-handling.md](../development/error-handling.md) §4 + [smart-contract-security-architecture.md](./smart-contract-security-architecture.md) §15.
- **Aturan**: kode baru wajib ditambahkan ke registry **sebelum** dipakai (SSOT); kode tidak pernah diganti tanpa alasan (stabil untuk FE/test/docs).
- Status HTTP ↔ namespace: 400 `INVALID_*`, 401 `AUTH_*`, 403 `FORBIDDEN_*`, 404 `NOT_FOUND_*`, 409 `CONFLICT_*`, 429 `RATE_*`, 5xx `SERVER_*` (tabel lengkap: [endpoints.md](../api/endpoints.md) § Tabel semantik status HTTP).

## 11. CORS spesifik

> API dipanggil browser dari origin web kita sendiri; **bukan** API publik lintas-origin. Karena itu CORS = **allowlist ketat**, bukan `*`.

```text
Access-Control-Allow-Origin      : <origin web final>          (echo origin bila ada di allowlist; JANGAN '*')
Access-Control-Allow-Methods     : GET, POST, PATCH, DELETE, OPTIONS
Access-Control-Allow-Headers     : Authorization, Content-Type, Idempotency-Key, X-Request-Id, If-None-Match
Access-Control-Expose-Headers    : ETag, Retry-After, Idempotency-Replayed
Access-Control-Allow-Credentials : false                        (auth via Bearer header, bukan cookie)
Access-Control-Max-Age           : 600                          (cache preflight 10 menit)
Vary                             : Origin
```

- **Allowlist origin**: origin web produksi + `http://localhost:<port>` untuk dev (port diizinkan eksplisit, bukan `localhost:*` wildcard). Origin di luar allowlist → **tidak** mengembalikan header CORS (browser memblokir); preflight ditolak.
- **Tanpa credentials** (`Allow-Credentials: false`) karena sesi memakai `Authorization: Bearer` (JWT) — tidak ada cookie lintas-origin; ini menutup CSRF lintas-origin (tidak ada cookie otomatis).
- Preflight `OPTIONS` tidak butuh auth, tetapi tetap tunduk rate limit ringan + `Vary: Origin` agar cache tidak salah origin.
- `Idempotency-Key` & `If-None-Match` masuk `Allow-Headers`; `ETag`/`Retry-After`/`Idempotency-Replayed` di-*expose* agar FE bisa membacanya.
- Nilai origin final mengikuti domain deploy (`<domain>` — open-by-design); dokumentasi CSP & domain: [frontend-security.md](./frontend-security.md) §2.
