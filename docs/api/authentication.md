# Authentication (API)

> Otentikasi untuk endpoint yang menulis (report/profil/admin). Transaksi chain tidak butuh ini — wallet sign langsung ke kontrak.
> **Keputusan (ADR-011, riset 2026-10-01): format utama = NEP-413** (status Final, dukungan wallet luas); custom challenge = fallback untuk wallet tanpa NEP-413.

## Model — DIPUTUSKAN: NEP-413 utama + custom fallback

> **Template pesan kanonik dimiliki [security/wallet-authentication.md](../security/wallet-authentication.md) §3** — dokumen ini tidak mendefinisikan ulang. Ringkasnya: urutan field login `Domain → Network → Account → Nonce → Expires → Scope`, dengan `Nonce` **wajib di dalam message**.

### Jalur utama: NEP-413

1. Frontend minta `GET /api/auth/nonce?account_id=…` → server simpan {nonce 32B, account_id, expires_at = now+5m, used=false} di tabel `auth_nonce` → balas {nonce, domain, network, expires_at}.
2. Frontend minta wallet `signMessage` (NEP-413) dengan payload:
   - `message`: teks kanonik (lihat template kanonik di wallet-authentication.md §3; network WAJIB di dalam message karena NEP-413 tidak punya field network; untuk step-up admin, Scope → `admin` + baris `Action:` + `Target:`),
   - `nonce`: 32-byte dari server, `recipient`: domain app (placeholder `nearsea.example`; nilai final saat deploy), `callbackUrl`: opsional.
3. Frontend kirim `POST /api/auth/verify {account_id, nonce, message, recipient, signature}` → **server validasi memcmp penuh** `message` terhadap template kanonik + `recipient` terhadap domain (tidak percaya string FE), lalu verifikasi signature:
   - scheme-agnostic: cek public key milik `account_id` & aktif via RPC `view_access_key_list` (mendukung ed25519/secp256k1/ml-dsa-65-hash) — SEC-AUTH-004.
4. Nonce dikonsumsi **atomically** (`UPDATE … WHERE used=false`) — sekali pakai, TTL 5 menit (SEC-AUTH-001). **Urutan: validasi dulu, konsumsi belakangan** — percobaan gagal tidak mengonsumsi nonce.
5. Server terbitkan **session token** (JWT pendek, TTL 15 menit, refresh ≤ 12 jam dengan rotasi — SEC-AUTH-003/006) berisi `{sub, scope}`. **Khusus scope `admin`: tanpa silent refresh** — admin wajib re-login + step-up (lihat [admin-security.md](../security/admin-security.md)).

### Fallback: custom challenge (wallet tanpa NEP-413)

Sama seperti di atas, tapi signature dibuat lewat sign-message wallet atas **template kanonik yang sama** (memuat Domain + Network + Account + Nonce + Expires + Scope, dengan Action/Target untuk step-up) — server membandingkan template penuh (SEC-AUTH-005).

## Step-up authorization (admin)

Aksi destruktif (hide/verified/blocklist) memakai template terpisah dengan **action name eksplisit** (kanonik di [wallet-authentication.md](../security/wallet-authentication.md) §3): `Domain → Network → Account → Action → Target → Nonce → Expires` — signature kedua diminta saat aksi (SEC-ADMIN-003), bukan memakai ulang token login.

## Rules

- Signature gagal validasi → 401; percobaan gagal TIDAK mengonsumsi nonce — dihitung rate-limit per IP+account (5 gagal/menit → lockout sementara).
- Tidak ada password/session cookie untuk hal yang bisa diverifikasi signature (token via header Authorization → CSRF tidak relevan).
- Private key TIDAK PERNAH menyentuh backend.
- Rate limit: nonce 10/IP/menit + 30/akun/jam; verify 10/IP/menit (api-security-architecture.md).

## Contoh request/response (worked)

> String `message` pada contoh di bawah mengikuti **template kanonik** [wallet-authentication.md](../security/wallet-authentication.md) §3 (`Domain → Network → Account → Nonce → Expires → Scope`). Dokumen ini **tidak** mendefinisikan ulang template — urutan/format field dimiliki §3.

### 1. Minta nonce

```http
GET /api/v1/auth/nonce?account_id=alice.testnet HTTP/1.1
Host: api.testnet.nearsea.example
```

```json
{
  "nonce": "3f9a1c7d5b2e8f04a6c1d9e7b3f5028c4a7d1e6f9b0c3a5d8e2f4b7c1a9d6e30",
  "domain": "nearsea.example",
  "network": "testnet",
  "expires_at": "2026-10-02T12:05:00Z"
}
```

- `nonce` = 32 byte (64 hex char), sekali pakai, TTL 5 menit.
- `domain` = placeholder kanonik; nilai final saat deploy ([wallet-authentication.md](../security/wallet-authentication.md) §3).

### 2. Verifikasi signature → session

```http
POST /api/v1/auth/verify HTTP/1.1
Host: api.testnet.nearsea.example
Content-Type: application/json; charset=utf-8
```

```json
{
  "account_id": "alice.testnet",
  "nonce": "3f9a1c7d5b2e8f04a6c1d9e7b3f5028c4a7d1e6f9b0c3a5d8e2f4b7c1a9d6e30",
  "message": "NearSea login\nDomain: nearsea.example\nNetwork: testnet\nAccount: alice.testnet\nNonce: 3f9a1c7d5b2e8f04a6c1d9e7b3f5028c4a7d1e6f9b0c3a5d8e2f4b7c1a9d6e30\nExpires: 2026-10-02T12:05:00Z\nScope: profile",
  "recipient": "nearsea.example",
  "public_key": "ed25519:7Kq3...",
  "signature": "ed25519:4mB9..."
}
```

`message` + `recipient` **wajib dikirim** agar server bisa membandingkan penuh (tidak percaya string FE) — lihat [wallet-authentication.md](../security/wallet-authentication.md) §3 langkah 4.

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900,
  "refresh_token": "rt_01HZX8...",
  "refresh_expires_in": 43200,
  "scope": "profile",
  "account_id": "alice.testnet"
}
```

### 3. Refresh (rotasi)

```http
POST /api/v1/auth/refresh HTTP/1.1
Content-Type: application/json; charset=utf-8
```

```json
{ "refresh_token": "rt_01HZX8..." }
```

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900,
  "refresh_token": "rt_01HZX9...",
  "refresh_expires_in": 43200,
  "scope": "profile",
  "account_id": "alice.testnet"
}
```

### 4. Logout / revokasi

```http
POST /api/v1/auth/logout HTTP/1.1
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json; charset=utf-8
```

```json
{ "refresh_token": "rt_01HZX9..." }
```

```http
HTTP/1.1 204 No Content
```

## Algoritma verifikasi signature (pseudocode)

> Urutan **wajib**: validasi dulu, konsumsi nonce belakangan (SEC-AUTH-001). Byte-level serialisasi signature mengikuti NEP-413 (termasuk prefix domain NEP-413); nilai persisnya dikunci oleh **test vector** di bagian akhir.

```text
function verify_login(req):
  n = db.auth_nonce.find(req.nonce)
  if n == null:                         reject 401 AUTH_NONCE_UNKNOWN
  if n.used:                            reject 401 AUTH_NONCE_USED
  if n.expires_at <= now_utc:           reject 401 AUTH_NONCE_EXPIRED
  if n.account_id != req.account_id:    reject 401 AUTH_MESSAGE_MISMATCH

  # 1) bandingkan message dengan template kanonik (constant-time)
  expected = canonical_login_message(
      domain=cfg.domain, network=cfg.network, account=n.account_id,
      nonce=n.nonce, expires=n.expires_at, scope=req.scope)
  if not constant_time_equal(expected, req.message): reject 401 AUTH_MESSAGE_MISMATCH
  if req.recipient != cfg.domain:                    reject 401 AUTH_MESSAGE_MISMATCH

  # 2) pastikan public key milik account & aktif (skema-agnostik — SEC-AUTH-004)
  keys = rpc.view_access_key_list(req.account_id)
  if req.public_key not in keys or not keys[req.public_key].active:
      reject 401 AUTH_KEY_NOT_FOUND

  # 3) verifikasi signature atas payload NEP-413 (serialisasi+prefix NEP-413)
  if not verify_signature(scheme=req.public_key.type,
                          payload=nep413_payload(req), sig=req.signature,
                          pk=req.public_key):
      reject 401 AUTH_SIGNATURE

  # 4) konsumsi nonce ATOMIC — hanya jika 1..3 lolos
  changed = db.auth_nonce.update(used=true where nonce=req.nonce and used=false)
  if changed == 0:                      reject 401 AUTH_NONCE_USED   # race

  return issue_session(account_id=req.account_id, scope=req.scope)
```

- `canonical_login_message` = fungsi yang membaca template §3; **bukan** literal tersebar di kode.
- `scope` yang diminta klien harus cocok dengan `Scope:` di dalam message; scope valid: `report | profile | admin`.
- Kegagalan pada langkah mana pun **tidak** mengonsumsi nonce; kegagalan dihitung rate-limit per IP+account (5 gagal/menit → lockout sementara).

## Skema klaim JWT

Access token = JWT (RFC 7519). **Satu scope per token.**

Header:

```json
{ "alg": "HS256", "typ": "JWT" }
```

- Algoritma **dikunci** `HS256` dengan kunci simetris dari env `JWT_SECRET` ([environments.md](../deployment/environments.md)). Verifikasi **menolak** `alg: none` dan alg di luar allowlist (mencegah alg-confusion). Rotasi `JWT_SECRET` ⏳ open-by-design (ditetapkan saat implementasi; rencana: dual-key window).

Payload (claims):

| Claim | Tipe | Wajib | Makna |
|---|---|---|---|
| `iss` | string | ya | Issuer tetap, mis. `nearsea-api` |
| `sub` | string | ya | `account_id` pemilik sesi |
| `scope` | string | ya | `report` \| `profile` \| `admin` |
| `iat` | number | ya | Waktu terbit (Unix detik) |
| `exp` | number | ya | Kedaluwarsa — **TTL 900 detik** (15 menit) |
| `jti` | string | ya | ID sesi = `sessions.token_id` (dipakai denylist/revokasi) |

```json
{
  "iss": "nearsea-api",
  "sub": "alice.testnet",
  "scope": "profile",
  "iat": 1759406400,
  "exp": 1759407300,
  "jti": "sess_01HZX8Q2..."
}
```

## Endpoint refresh & rotasi

- `POST /api/v1/auth/refresh` — body `{ "refresh_token": "<opaque>" }`; **tanpa** Bearer access token.
- Refresh token **opaque** (bukan JWT), tersimpan di tabel `sessions` (`token_id`, `account_id`, `scope`, `expires_at`, `rotated_from`) — [database-schema.md](../database/database-schema.md).
- TTL refresh **≤ 12 jam** (43200 detik); access token baru TTL 15 menit.
- **Rotasi setiap pakai** (SEC-AUTH-006): refresh lama langsung invalid; refresh baru mencatat `rotated_from`.
- **Deteksi reuse**: memakai refresh yang sudah dirotasi → seluruh rantai sesi (family) dicabut + 401 (kode `AUTH_REFRESH_REUSED`).
- Refresh kedaluwarsa → 401 `AUTH_REFRESH_INVALID`.
- **Scope `admin`: TIDAK ada silent refresh** — refresh endpoint menolak token scope admin; admin wajib re-login + step-up ([admin-security.md](../security/admin-security.md) §2).

## Logout & revokasi

- `POST /api/v1/auth/logout` — Bearer access token + body `{ "refresh_token": "<opaque>" }` (opsional bila hanya ingin mencabut access).
- Efek: hapus sesi server-side (baris `sessions`) + masukkan `jti` ke **denylist** sampai `exp` (access token yang sudah terbit tetap ditolak).
- Respons `204 No Content` (idempoten: logout ulang tetap 204).
- Revokasi admin (mis. allowlist berubah) → revoke semua sesi akun tersebut (aksi SUPER_ADMIN + audit).
- JWT tidak bisa "ditarik" sebelum `exp` tanpa denylist — karena itu TTL dijaga pendek (15 menit).

## Matriks scope → permission

| Scope | Endpoint yang diizinkan | Syarat tambahan |
|---|---|---|
| `profile` | `PATCH /api/v1/accounts/me/profile` | hanya `sub` sendiri |
| `report` | `POST /api/v1/reports` | akun harus terverifikasi |
| `admin` | `GET/PATCH/POST/DELETE /api/v1/admin/*` | **allowlist DB** + **step-up signature per aksi destruktif** (SEC-ADMIN-003) |

- Endpoint read publik (profil publik, discovery fase 2) **tidak butuh** scope.
- Token scope `admin` **tidak** otomatis memberi `profile`/`report`; scope terpisah.
- Scope di klaim JWT **saja** tidak cukup untuk admin — server tetap cek allowlist DB ([api-security-architecture.md](../security/api-security-architecture.md) §2).

## Penanganan clock skew

- `expires_at` nonce ditetapkan **server** (UTC) dan dibandingkan dengan jam server; jam klien tidak memengaruhi validitas.
- Verifikasi JWT: toleransi **±60 detik** untuk `iat` (mengakomodasi selisih jam); `exp` tidak ditoleransi melewati batas (token lewat `exp` → 401 `AUTH_SESSION_EXPIRED`).
- `iat` di masa depan > 60 detik → 401 (kode `AUTH_TOKEN_INVALID`).
- Server wajib sinkron NTP (lihat [deployment/monitoring.md](../deployment/monitoring.md)).
- TTL nonce & session dihitung dari jam server, bukan dari timestamp di request.

## Access-key removal/rotation & akun multi-key

- **Verifikasi ulang setiap `verify`**: server memanggil `view_access_key_list(account_id)` (RPC) dan memastikan `public_key` yang dikirim **terdaftar & aktif** — skema-agnostik ed25519/secp256k1/ml-dsa-65-hash (SEC-AUTH-004).
- **Akun multi-key**: kunci mana pun yang aktif boleh dipakai sign; tidak ada binding "satu kunci". `public_key` yang dikirim harus ada di daftar; jika tidak → 401 `AUTH_KEY_NOT_FOUND`.
- **Kunci dihapus/di-rotasi setelah token terbit**: JWT yang sudah terbit tetap valid sampai `exp` (≤15 menit). Aksi admin destruktif **selalu** memverifikasi ulang kunci saat step-up, sehingga kunci yang dicabut tidak bisa menyelesaikan aksi.
- Penghapusan kunci terdeteksi (mis. job/saat verify) → cabut sesi akun terkait (best-effort; mekanisme final ⏳ open-by-design).
- Akun tanpa kunci aktif (mis. `0` key) → semua verify gagal 401 `AUTH_KEY_NOT_FOUND`.
- Perilaku function-call key vs full-access key untuk `signMessage` → ⏳ open-by-design (dikunci saat implementasi + test).

## Daftar kode error (auth)

> Katalog kode **dimiliki** [development/error-handling.md](../development/error-handling.md) §3. Tabel ini menampilkan subset relevan auth — semua kode di bawah **sudah terdaftar** di registry.

| Status | Kode | Kondisi |
|---|---|---|
| 401 | `AUTH_NONCE_UNKNOWN` | nonce tidak ditemukan / bukan milik account |
| 401 | `AUTH_NONCE_EXPIRED` | nonce lewat TTL 5 menit |
| 401 | `AUTH_NONCE_USED` | nonce sudah dikonsumsi (replay / race) |
| 401 | `AUTH_MESSAGE_MISMATCH` | `message`/`recipient`/domain/network/scope tidak cocok template kanonik |
| 401 | `AUTH_SIGNATURE` | signature tidak valid atas `public_key` |
| 401 | `AUTH_KEY_NOT_FOUND` | `public_key` bukan kunci aktif milik `account_id` |
| 401 | `AUTH_SESSION_EXPIRED` | access token lewat `exp` |
| 401 | `AUTH_TOKEN_INVALID` | JWT malformed / alg tidak diizinkan / `iat` skew |
| 401 | `AUTH_REFRESH_INVALID` | refresh token tidak dikenal/kedaluwarsa |
| 401 | `AUTH_REFRESH_REUSED` | refresh token sudah dirotasi (reuse terdeteksi → family dicabut) |
| 403 | `FORBIDDEN_SCOPE` | scope token kurang untuk endpoint |
| 403 | `FORBIDDEN_ADMIN` | bukan allowlist admin |
| 429 | `RATE_LIMITED` | rate limit / lockout sementara (lihat di bawah) |

Contoh respons lockout (verify gagal berulang):

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 60
Content-Type: application/json; charset=utf-8
```

```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Too many failed attempts. Try again later.",
    "requestId": "req_01HZX8..."
  }
}
```

- Lockout: **5 verify gagal/menit** (per IP+account) → ditolak 429 + `Retry-After` sampai jendela pulih.
- Semua percobaan gagal di-log (account, IP hash, alasan) untuk alert Telegram ([wallet-authentication.md](../security/wallet-authentication.md) §5).
- Pesan user final lewat i18n; teks di atas hanya contoh.

## NEP-413 test vectors

> **SSOT vektor = [wallet-authentication.md](../security/wallet-authentication.md) §8 (TV-1..TV-8)** —
> dokumen itu mendefinisikan seluruh vektor, termasuk scope-mismatch (TV-7) dan step-up target (TV-8).
> Bagian ini hanya menjelaskan cakupan minimum yang wajib tercakup oleh vektor tersebut.

**Status: ⏳ nilai diisi saat implementasi** — nilai byte-level (serialisasi payload, prefix NEP-413,
encoding nonce) harus dikunci oleh vektor nyata dari wallet, bukan dikarang. Implementasi wajib
mengisi vektor di wallet-authentication.md §8 dan menjalankannya sebagai test.

**Cakupan wajib vektor** (harus tercakup TV-1..TV-8): NEP-413 valid, nonce mismatch, domain/recipient
mismatch, nonce dipakai ulang, nonce expired, minimal satu skema kunci non-ed25519, scope tidak sesuai,
dan step-up admin (Action/Target). Sumber format NEP-413: [wallet-authentication.md](../security/wallet-authentication.md) §2 (FACT riset 2026-10-01).

## Status

- Contoh request/response, algoritma verifikasi, skema JWT, refresh/logout, matriks scope, clock skew, penanganan kunci — **DECIDED (dokumen ini)** sebagai spesifikasi implementasi.
- Kode error di tabel §8 **sudah terdaftar** di [error-handling.md](../development/error-handling.md) §3 (SSOT).
- NEP-413 test vectors — ⏳ **diisi saat implementasi**.
- Function-call key vs full-access untuk `signMessage` — ⏳ open-by-design.
