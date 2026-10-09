# Error Handling & Notifications

> **Single source of truth untuk taksonomi error & kebijakan notifikasi.**
> Kasus error *per fitur* tetap dimiliki dokumen fitur masing-masing
> ([features/marketplace.md](../features/marketplace.md), [features/payments.md](../features/payments.md),
> [features/notifications.md](../features/notifications.md), [features/auth.md](../features/auth.md),
> [features/users.md](../features/users.md)); dokumen ini yang menetapkan **format, kode, dan
> perilaku tampilannya** agar konsisten dan tidak berantakan.
>
> Permintaan eksplisit pemilik proyek: pesan/notifikasi error harus rapi, tidak berantakan.

## 1. Prinsip

1. **Satu error = satu kode + satu pesan user.** Tidak ada pesan ad-hoc yang ditulis di tempat berbeda.
2. **Pesan user ≠ pesan log.** User dapat pesan singkat & actionable; detail teknis hanya di log server.
3. **Tidak ada stack trace / detail internal** di response API atau di UI (SEC-API).
4. **Kode error stabil** — dipakai di FE, test, dan docs; tidak diganti tanpa alasan.
5. **Bahasa pesan user = English via i18n** (kunci i18n, bukan literal di komponen).

## 2. Envelope API (SSOT bentuk)

Semua error API memakai bentuk seragam:

```json
{ "error": { "code": "INVALID_ALIAS", "message": "…", "requestId": "…" } }
```

- `code` — dari registry (§3), SCREAMING_SNAKE_CASE.
- `message` — singkat, untuk ditampilkan; tidak memuat detail internal.
- `requestId` — untuk korelasi dengan log server (bukan untuk user).
- Status HTTP + kode: `400` validasi, `401` auth, `403` izin, `404` tidak ada,
  `409` konflik, `429` rate limit, `5xx` server.

## 3. Registry kode error (namespace)

| Namespace | Dipakai untuk | Kode terdaftar |
|---|---|---|
| `INVALID_*` | Validasi input | `INVALID_ALIAS`, `INVALID_BIO`, `INVALID_AVATAR`, `INVALID_PRICE`, `INVALID_ACCOUNT_ID`, `INVALID_CURSOR`, `INVALID_QUERY`, `INVALID_REASON`, `INVALID_TARGET`, `INVALID_ROYALTY` |
| `AUTH_*` | Autentikasi/sesi | `AUTH_NONCE_UNKNOWN`, `AUTH_NONCE_EXPIRED`, `AUTH_NONCE_USED`, `AUTH_MESSAGE_MISMATCH`, `AUTH_SIGNATURE`, `AUTH_KEY_NOT_FOUND`, `AUTH_SESSION_EXPIRED`, `AUTH_TOKEN_INVALID`, `AUTH_REFRESH_INVALID`, `AUTH_REFRESH_REUSED` |
| `FORBIDDEN_*` | Otorisasi | `FORBIDDEN_ADMIN`, `FORBIDDEN_SCOPE`, `FORBIDDEN_SELF_BUY`, `FORBIDDEN_BUYER` |
| `NOT_FOUND_*` | Sumber tak ada | `NOT_FOUND_ACCOUNT`, `NOT_FOUND_TOKEN`, `NOT_FOUND_COLLECTION`, `NOT_FOUND_REPORT`, `NOT_FOUND_BLOCKLIST`, `NOT_FOUND_BUNDLE` |
| `CONFLICT_*` | Bentrok state | `CONFLICT_SOLD`, `CONFLICT_STALE`, `CONFLICT_ALREADY_LISTED`, `CONFLICT_OFFER_EXISTS`, `CONFLICT_PRICE_CHANGED`, `CONFLICT_REPORT_EXISTS`, `CONFLICT_ALREADY_DECIDED`, `CONFLICT_ALREADY_BLOCKED`, `CONFLICT_ALREADY_TAKEN`, `CONFLICT_PHASE_OVERLAP`, `CONFLICT_PRE_VALIDATE_FAILED`, `CONFLICT_BUNDLE_TOO_MANY`, `CONFLICT_BUNDLE_ITEM_INVALID` |
| `LAUNCHPAD_*` | Kegagalan alur launchpad | `LAUNCHPAD_PHASE_INACTIVE`, `LAUNCHPAD_NOT_ALLOWED`, `LAUNCHPAD_ALLOCATION_EXHAUSTED`, `LAUNCHPAD_MAX_PER_WALLET`, `LAUNCHPAD_PRICE_MISMATCH` |
| `ADMIN_*` | Kendala aksi admin | `ADMIN_STEPUP_REQUIRED`, `ADMIN_STEPUP_INVALID` |
| `RATE_*` | Rate limit | `RATE_LIMITED` |
| `CHAIN_*` | Kegagalan dari chain/kontrak | `CHAIN_REVERT`, `CHAIN_TIMEOUT`, `CHAIN_INSUFFICIENT_DEPOSIT`, `CHAIN_PAUSED` |
| `SERVER_*` | Kesalahan internal | `SERVER_ERROR` |

**Pengecualian (bukan kode user-facing)**: `MISSING_IN_DB` / `MISSING_ON_CHAIN` di
[indexer-security.md](../security/indexer-security.md) & [ADR-015](../decisions/ADR-015-indexer-consistency.md)
adalah **status rekonsiliasi internal**, bukan kode error yang dikirim ke client — dikecualikan dari aturan
"kode wajib terdaftar" agar aturan tetap bisa diaudit.

> Registry ini bertambah saat implementasi; setiap kode baru **wajib** ditambahkan di sini
> sebelum dipakai (SSOT). Kode `AUTH_*` lengkap didefinisikan bersama tabel error per-endpoint di
> [api/authentication.md](../api/authentication.md) §8; `INVALID_*`/`FORBIDDEN_*`/`CONFLICT_*`/`NOT_FOUND_*`/`LAUNCHPAD_*`/`ADMIN_*`
> dipakai di [api/endpoints.md](../api/endpoints.md), [features/marketplace.md](../features/marketplace.md),
> [features/users.md](../features/users.md), dan [02-product-requirements.md](../02-product-requirements.md) (launchpad).
> Semua kode di tabel ini **sudah terdaftar** — tidak ada kode ➕ yang menggantung.
> Rincian per domain (launchpad/report/admin/profil) + status HTTP per kode: §10–§11.

## 4. Pemetaan panic kontrak → kode user

Kontrak melempar panic/assert dengan teks; wallet menampilkannya mentah. FE **wajib**
menormalkan ke kode user (bukan menampilkan teks kontrak apa adanya):

| Kondisi on-chain | Kode user | Pesan user (i18n key) |
|---|---|---|
| Sale sudah dihapus / tidak ada | `CONFLICT_SOLD` | "Sudah terjual — dana kamu kembali" (refund otomatis) |
| Deposit < harga | `CHAIN_INSUFFICIENT_DEPOSIT` | "Jumlah kurang dari harga" |
| Buyer = seller | `FORBIDDEN_SELF_BUY` | "Tidak bisa membeli milik sendiri" |
| Token tidak sesuai listing (stale) | `CONFLICT_STALE` | "Listing tidak lagi valid" |
| Private listing, buyer bukan target | `FORBIDDEN_BUYER` | "Listing ini khusus pembeli tertentu" |
| Offer aktif ganda | `CONFLICT_OFFER_EXISTS` | "Kamu sudah punya offer aktif di token ini" |
| Royalti > 10% / payout invalid | `INVALID_ROYALTY` | "Royalti tidak valid" |
| Bundle gagal pre-validasi (item invalid/stale) | `CONFLICT_PRE_VALIDATE_FAILED` | "Bundle tidak bisa dibeli — salah satu item tidak valid" |
| Bundle > 10 token | `CONFLICT_BUNDLE_TOO_MANY` | "Maksimum 10 token per bundle" |
| Item bundle sudah tergabung bundle aktif lain | `CONFLICT_BUNDLE_ITEM_INVALID` | "Token sudah tergabung dalam bundle aktif" |
| Launchpad: phase overlap saat konfigurasi | `CONFLICT_PHASE_OVERLAP` | "Phases cannot overlap" |
| Harga berubah antar preview & sign | `CONFLICT_PRICE_CHANGED` | "Harga berubah — periksa ulang" |
| Kontrak di-pause | `CHAIN_PAUSED` | "Marketplace sedang dijeda sementara" |
| Launchpad: fase tidak aktif / belum mulai | `LAUNCHPAD_PHASE_INACTIVE` | "Mint belum dibuka" |
| Launchpad: minter di luar allowlist | `LAUNCHPAD_NOT_ALLOWED` | "Kamu belum masuk allowlist fase ini" |
| Launchpad: alokasi fase habis | `LAUNCHPAD_ALLOCATION_EXHAUSTED` | "Alokasi fase ini habis" |
| Launchpad: melebihi max per wallet | `LAUNCHPAD_MAX_PER_WALLET` | "Sudah mencapai batas mint per wallet" |
| Launchpad: deposit ≠ harga fase | `LAUNCHPAD_PRICE_MISMATCH` | "Jumlah tidak sesuai harga fase" |

- Sumber daftar kasus: tabel Error Cases di [features/marketplace.md](../features/marketplace.md)
  dan [features/payments.md](../features/payments.md).
- Pemetaan dilakukan dengan **mencocokkan tipe error**, bukan substring pesan mentah
  (pesan kontrak bisa berubah). Bila tak dikenal → `CHAIN_REVERT` generik.

## 5. Siklus hidup transaksi (FE)

```text
idle ──submit──► submitted ──masuk blok──► included ──final──► success
                    │                          │
                    └──────────► failed ◄──────┘  (revert / timeout / drop)
```

- **submitted** — tx dikirim; tombol `pending-tx` (disabled + spinner).
- **included** — masuk blok, belum final; UI boleh tampilkan "menunggu konfirmasi".
- **final** — finality tercapai → invalidate query, tampilkan hasil.
- **failed** — revert/timeout/drop → tampilkan kode user (§4), **bukan** teks mentah.
- Aturan koneksi putus saat signing: **cek status tx via hash dulu**; reset state hanya
  bila tx tidak ditemukan/gagal (lihat [frontend-architecture.md](../architecture/frontend-architecture.md)).

## 6. Kebijakan notifikasi (agar tidak berantakan)

- **Satu aksi → satu notifikasi utama.** Hasil tx (sukses/gagal) = 1 toast, tidak diulang.
- **Severity**: `info` (hasil normal), `warning` (perlu perhatian, mis. harga berubah),
  `error` (aksi gagal). Hanya `error` yang boleh menampilkan tombol retry.
- **Dedup**: notifikasi berbasis event id **`(receipt_id, event_index)`** — tidak dobel bila polling mengulang; bukan `tx_hash` (satu tx bisa memuat banyak event)
  (lihat [features/notifications.md](../features/notifications.md)).
- **Grouping**: beberapa event sejenis dalam waktu singkat (mis. beberapa offer) diringkas
  jadi satu entri "3 offer baru", bukan 3 toast bertumpuk.
- **Rate limit tampilan**: jangan tampilkan lebih dari satu toast per aksi; antre sisanya
  ke panel notifikasi.
- **Tidak ada** notifikasi teknis (stack trace, kode internal) ke user — itu log server.
- **Alert Telegram** hanya untuk admin/ops (5xx, pause, owner-tx), bukan user biasa
  (lihat [deployment/monitoring.md](../deployment/monitoring.md)).

## 7. Logging

- Server: log detail (stack, requestId, konteks) — **server-side saja**.
- 5xx → alert Telegram otomatis (SEC-API).
- Jangan pernah log secret, signature, seed, atau token sesi.

## 8. Konsistensi implementasi (satu modul error terpusat)

- **Kode & pesan error hanya didefinisikan di satu modul terpusat.** Semua kode lain
  meng-**impor** dari situ — **dilarang** menulis kode/pesan error ad-hoc di komponen, route,
  atau util lain.
- Modul ini adalah **implementasi** dari katalog §3/§4; **SSOT isinya tetap dokumen ini**.
  Tambah kode baru → update dokumen ini dulu (sesuai [DOCUMENTATION-MAP.md](../DOCUMENTATION-MAP.md) T17), baru modul kode.
- Path modul **sudah dikunci saat scaffold (TASK-001)**: `frontend/lib/errors/` untuk sisi FE
  ([frontend-architecture.md](../architecture/frontend-architecture.md) §1), modul error server untuk API.
  Modul FE belum dibuat — masuk bersama pemakaian pertamanya (TASK-008, TASK-033).
- **Pengecualian yang disengaja (TASK-007):** `frontend/lib/near/wallet-errors.ts` mengklasifikasi error
  **wallet** menjadi kode lokal (`rejected`/`wallet_unavailable`/`connect_failed`) — kode ini **bukan**
  kode registry §3 (features/auth.md §Error Cases menyebutnya "lokal, tanpa kode API"), jadi tidak ada
  kode registry yang di-duplikasi. Pesan user tetap lewat i18n (`auth.error.*`). Kalau kelak error wallet
  perlu punya kode registry, klasifikasi ini yang dipindah ke `lib/errors/`, bukan ditulis ulang.
- Alasan: mencegah pesan tidak konsisten antar layar, memudahkan audit (satu tempat), dan
  memastikan pemetaan panic kontrak (§4) hanya ditulis sekali.
- Selaras dengan perilaku **Surgical Changes** di [AGENTS.md](../../AGENTS.md): ikuti pola yang
  ada, jangan menciptakan pola baru.

## 9. Katalog pesan i18n (lengkap)

Kode error dan pesan user dipisah: **kode** stabil (registry §3), **pesan** = nilai i18n. Struktur file dimiliki [frontend-architecture.md](../architecture/frontend-architecture.md) §9; di sini yang ditetapkan adalah **bentuk katalog**-nya.

> **Koreksi ronde 17.** Versi sebelumnya hanya memuat **10 kunci** untuk ~56 kode — artinya error yang
> paling sering dialami user (harga berubah, stok habis, tidak masuk allowlist, deposit kurang) jatuh ke
> fallback generik *"Something went wrong"* dan user akan retry selamanya untuk kondisi yang tidak bisa
> di-retry. Katalog di bawah memuat **semua kode terdaftar**.

```json
// i18n/en/errors.json
{
  "errors": {
    "INVALID_ALIAS": "Display name must be 32 characters or fewer",
    "INVALID_BIO": "Bio must be 280 characters or fewer",
    "INVALID_AVATAR": "That avatar URL isn't allowed",
    "INVALID_PRICE": "Price is below the minimum (0.01 Ⓝ)",
    "INVALID_ACCOUNT_ID": "That account name isn't valid",
    "INVALID_CURSOR": "This page has expired — start over",
    "INVALID_QUERY": "Check your search and try again",
    "INVALID_REASON": "Please choose a reason",
    "INVALID_TARGET": "That item can't be reported",
    "INVALID_ROYALTY": "Royalty must be 10% or less",

    "AUTH_NONCE_UNKNOWN": "Sign-in request expired — please try again",
    "AUTH_NONCE_EXPIRED": "Sign-in took too long — please try again",
    "AUTH_NONCE_USED": "This sign-in was already used — please try again",
    "AUTH_MESSAGE_MISMATCH": "Signature doesn't match this site — please try again",
    "AUTH_SIGNATURE": "Signature couldn't be verified — please try again",
    "AUTH_KEY_NOT_FOUND": "No active key found for this account",
    "AUTH_SESSION_EXPIRED": "Your session expired — please sign in again",
    "AUTH_TOKEN_INVALID": "Session is invalid — please sign in again",
    "AUTH_REFRESH_INVALID": "Session expired — please sign in again",
    "AUTH_REFRESH_REUSED": "Session ended for security — please sign in again",

    "FORBIDDEN_ADMIN": "You don't have admin access",
    "FORBIDDEN_SCOPE": "You don't have permission for this action",
    "FORBIDDEN_SELF_BUY": "You can't buy your own item",
    "FORBIDDEN_BUYER": "This listing is reserved for a specific buyer",

    "NOT_FOUND_ACCOUNT": "Account not found",
    "NOT_FOUND_TOKEN": "This item no longer exists",
    "NOT_FOUND_COLLECTION": "Collection not found",
    "NOT_FOUND_REPORT": "Report not found",
    "NOT_FOUND_BLOCKLIST": "That entry isn't in the blocklist",
    "NOT_FOUND_BUNDLE": "This bundle no longer exists",

    "CONFLICT_SOLD": "Already sold — your funds were returned",
    "CONFLICT_STALE": "This listing is no longer valid",
    "CONFLICT_ALREADY_LISTED": "This item is already listed",
    "CONFLICT_OFFER_EXISTS": "You already have an active offer on this item",
    "CONFLICT_PRICE_CHANGED": "The price changed — review and confirm again",
    "CONFLICT_REPORT_EXISTS": "You already reported this today",
    "CONFLICT_ALREADY_DECIDED": "This report was already decided",
    "CONFLICT_ALREADY_BLOCKED": "That item is already blocked",
    "CONFLICT_ALREADY_TAKEN": "That name is already taken — choose another",
    "CONFLICT_PHASE_OVERLAP": "Mint phases can't overlap",
    "CONFLICT_PRE_VALIDATE_FAILED": "This bundle can't be bought — one item is no longer valid",
    "CONFLICT_BUNDLE_TOO_MANY": "Maximum 10 items per bundle",
    "CONFLICT_BUNDLE_ITEM_INVALID": "This item is already in an active bundle",

    "LAUNCHPAD_PHASE_INACTIVE": "Mint isn't open right now",
    "LAUNCHPAD_NOT_ALLOWED": "You're not on this phase's allowlist",
    "LAUNCHPAD_ALLOCATION_EXHAUSTED": "This phase is sold out",
    "LAUNCHPAD_MAX_PER_WALLET": "You've reached the mint limit per wallet",
    "LAUNCHPAD_PRICE_MISMATCH": "The amount doesn't match this phase's price",

    "ADMIN_STEPUP_REQUIRED": "Signature confirmation required for this action",
    "ADMIN_STEPUP_INVALID": "Confirmation signature is invalid — try again",

    "RATE_LIMITED": "Too many requests — try again shortly",

    "CHAIN_REVERT": "Transaction failed — nothing was charged beyond network fees",
    "CHAIN_TIMEOUT": "Network is slow — check your transaction before retrying",
    "CHAIN_INSUFFICIENT_DEPOSIT": "Not enough NEAR to cover the price and network fees",
    "CHAIN_PAUSED": "Marketplace is temporarily paused",

    "SERVER_ERROR": "Something went wrong — please try again"
  }
}
```

| Aturan katalog | Nilai |
|---|---|
| Kunci | `errors.<ERROR_CODE>` — kode **verbatim** dari registry §3 (SCREAMING_SNAKE_CASE) |
| Bahasa | English (MVP); siap locale tambahan tanpa mengubah kode |
| Placeholder | Bernama, mis. `"offer.expiresIn": "Expires in {days} days"` |
| Panjang pesan | ≤ 120 karakter; actionable; tanpa detail internal |
| Fallback | Kode tanpa kunci → `SERVER_ERROR` (jangan tampilkan kode mentah ke user) |
| Kode baru | Wajib menambah kunci `errors.<CODE>` **bersamaan** dengan update registry §3 (satu PR) |
| Kelengkapan | **Semua kode registry §3 wajib punya kunci** — diperiksa di CI (jumlah kunci ≥ jumlah kode) |

- Pemetaan panic kontrak (§4) menghasilkan **kode**; komponen menerjemahkan kode → `t('errors.' + code)`.
- **Catatan penting**: `CHAIN_REVERT` sengaja **tidak** menjanjikan refund ("nothing was charged beyond
  network fees") — karena tidak semua kegagalan mengembalikan dana (mis. gagal listing tidak ada dana
  untuk dikembalikan). Versi lama menulis "your funds were returned" yang menyesatkan untuk aksi non-beli.
- Pesan teknis (nama method kontrak, nomor baris) **tidak** masuk katalog user — itu log server (§7).
- Kunci `errors.*` bertipe sehingga kode baru tanpa kunci = error build (frontend-architecture §9).

## 10. Status HTTP per kode (tabel lengkap)

Setiap kode registry §3 punya **satu** status HTTP kanonik. Tabel ini SSOT pemetaan kode → status; [endpoints.md](../api/endpoints.md) § Tabel semantik status HTTP adalah padanan per-endpoint.

| Kode | HTTP | Konteks |
|---|---|---|
| `INVALID_ALIAS` | 400 | PATCH profil |
| `INVALID_BIO` | 400 | PATCH profil |
| `INVALID_AVATAR` | 400 | PATCH profil |
| `INVALID_PRICE` | 400 | Validasi harga (FE/klien) |
| `INVALID_ACCOUNT_ID` | 400 | `GET /auth/nonce` |
| `INVALID_CURSOR` | 400 | Pagination |
| `INVALID_QUERY` | 400 | `GET /search` (★ fase 2) |
| `INVALID_REASON` | 400 | `POST /reports` (enum reason) |
| `INVALID_TARGET` | 400 | `POST /reports`, blocklist |
| `AUTH_NONCE_UNKNOWN` | 401 | verify |
| `AUTH_NONCE_EXPIRED` | 401 | verify |
| `AUTH_NONCE_USED` | 401 | verify (replay) |
| `AUTH_MESSAGE_MISMATCH` | 401 | verify |
| `AUTH_SIGNATURE` | 401 | verify |
| `AUTH_KEY_NOT_FOUND` | 401 | verify |
| `AUTH_SESSION_EXPIRED` | 401 | Semua endpoint Bearer |
| `AUTH_TOKEN_INVALID` | 401 | Semua endpoint Bearer |
| `AUTH_REFRESH_INVALID` | 401 | refresh |
| `AUTH_REFRESH_REUSED` | 401 | refresh |
| `FORBIDDEN_ADMIN` | 403 | Endpoint admin |
| `FORBIDDEN_SCOPE` | 403 | Scope token kurang |
| `FORBIDDEN_SELF_BUY` | 403 | `buy`/`buy_bundle` (FE mapping) |
| `FORBIDDEN_BUYER` | 403 | Private listing |
| `NOT_FOUND_ACCOUNT` | 404 | Profil |
| `NOT_FOUND_TOKEN` | 404 | Report/discovery |
| `NOT_FOUND_COLLECTION` | 404 | Report/admin |
| `NOT_FOUND_REPORT` | 404 | PATCH admin report |
| `NOT_FOUND_BLOCKLIST` | 404 | DELETE blocklist |
| `NOT_FOUND_BUNDLE` | 404 | Discovery bundle |
| `CONFLICT_SOLD` | 409 | Buy (on-chain mapping) |
| `CONFLICT_STALE` | 409 | Listing stale |
| `CONFLICT_ALREADY_LISTED` | 409 | Listing |
| `CONFLICT_OFFER_EXISTS` | 409 | Offer |
| `CONFLICT_PRICE_CHANGED` | 409 | Re-verify |
| `CONFLICT_REPORT_EXISTS` | 409 | `POST /reports` |
| `CONFLICT_ALREADY_DECIDED` | 409 | Admin decision |
| `CONFLICT_ALREADY_BLOCKED` | 409 | Blocklist |
| `LAUNCHPAD_PHASE_INACTIVE` | 409 | Mint |
| `LAUNCHPAD_NOT_ALLOWED` | 403 | Mint |
| `LAUNCHPAD_ALLOCATION_EXHAUSTED` | 409 | Mint |
| `LAUNCHPAD_MAX_PER_WALLET` | 409 | Mint |
| `LAUNCHPAD_PRICE_MISMATCH` | 400 | Mint |
| `ADMIN_STEPUP_REQUIRED` | 403 | Aksi admin destruktif |
| `ADMIN_STEPUP_INVALID` | 401 | Step-up signature |
| `RATE_LIMITED` | 429 | Semua (＋ `Retry-After`) |
| `CHAIN_REVERT` | 409 | Mapping panic |
| `CHAIN_TIMEOUT` | 504 | RPC timeout |
| `CHAIN_INSUFFICIENT_DEPOSIT` | 400 | Mapping panic |
| `CHAIN_PAUSED` | 503 | Kontrak paused |
| `SERVER_ERROR` | 500 | Internal |

- Kode `CHAIN_*` biasanya muncul di **FE** (mapping panic wallet), bukan di respons REST; status HTTP di atas berlaku bila API memproyeksikannya.
- `504` untuk `CHAIN_TIMEOUT` (gateway timeout), bukan `500` — memudahkan retry klien.
- Satu kode **tidak boleh** dipetakan ke dua status berbeda; bila butuh status berbeda → buat kode baru.

## 11. Kode per domain (launchpad / report / admin / profil)

**Launchpad** (on-chain → FE mapping; SEC-CONTRACT-009, INV-017/018/029):

| Kode | Pemicu | Pesan user |
|---|---|---|
| `LAUNCHPAD_PHASE_INACTIVE` | Belum mulai / sudah berakhir / fase overlap (INV-029) | "Mint belum dibuka" |
| `LAUNCHPAD_NOT_ALLOWED` | Phase allowlist & minter tidak terdaftar | "Kamu belum masuk allowlist fase ini" |
| `LAUNCHPAD_ALLOCATION_EXHAUSTED` | Alokasi fase habis | "Alokasi fase ini habis" |
| `LAUNCHPAD_MAX_PER_WALLET` | Melebihi `max_per_wallet` | "Sudah mencapai batas mint per wallet" |
| `LAUNCHPAD_PRICE_MISMATCH` | `attached_deposit ≠ phase.price` (INV-018) | "Jumlah tidak sesuai harga fase" |

**Report** (API):

| Kode | Pemicu |
|---|---|
| `INVALID_REASON` | Enum reason di luar daftar (nilai enum final ⏳ open-by-design) |
| `INVALID_TARGET` | Target report tidak valid (format/tipe) |
| `NOT_FOUND_COLLECTION` / `NOT_FOUND_TOKEN` | Target tidak ada |
| `CONFLICT_REPORT_EXISTS` | Sudah report target sama hari ini (unik DB) |
| `RATE_LIMITED` | > 5 report/akun/hari |

**Admin** (API):

| Kode | Pemicu |
|---|---|
| `FORBIDDEN_ADMIN` | Tidak ada di allowlist DB (bukan sekadar scope) |
| `ADMIN_STEPUP_REQUIRED` | Aksi destruktif tanpa step-up signature |
| `ADMIN_STEPUP_INVALID` | Step-up signature tidak valid/expired |
| `NOT_FOUND_REPORT` / `NOT_FOUND_BLOCKLIST` | Resource admin tidak ada |
| `CONFLICT_ALREADY_DECIDED` | Report sudah diputus admin lain |
| `CONFLICT_ALREADY_BLOCKED` | Target sudah di blocklist |

**Profil** (API):

| Kode | Pemicu |
|---|---|
| `INVALID_ALIAS` | Alias > 32 char / karakter kontrol |
| `INVALID_BIO` | Bio > 280 char / karakter kontrol |
| `INVALID_AVATAR` | URL di luar gateway allowlist / skema terlarang |
| `NOT_FOUND_ACCOUNT` | Profil/akun tidak ada |
| `FORBIDDEN_SCOPE` | Token bukan scope `profile` |

- Aturan: setiap domain menambah kode **hanya** setelah baris masuk §3 + §10 + kunci i18n §9.

## 12. `requestId` (format & generasi)

- **Format**: `req_` + ULID/UUIDv7 **26–36 char** (mis. `req_01HZX8Q2K3M4N5P6Q7R8S9T0V1`). Dipilih ULID/UUIDv7 agar **urut waktu** (memudahkan pencarian log) dan unik.
- **Generasi**: dibuat server **per request masuk** (middleware paling luar), sebelum routing/auth — sehingga request yang gagal auth pun punya `requestId`.
- **Propagasi**: selalu dikembalikan di body error (`error.requestId`) **dan** header `X-Request-Id`. Klien boleh mengirim `X-Request-Id`; bila ada dan valid → dipakai server (korelasi klien), bila tidak → server generate.
- **Log**: setiap baris log terkait request memuat `requestId` (structured logging). **Dilarang** memuat `requestId` di pesan user (itu internal, §1).
- **Bukan** rahasia dan **bukan** otorisasi — hanya korelasi. Jangan menaruh data user di dalamnya.
- Retensi log yang memuat `requestId`: sesuai [monitoring.md](../deployment/monitoring.md) (min. 30 hari untuk 5xx).

## 13. Retry & backoff (parameter)

Retry hanya untuk kegagalan **transien**; dilarang retry aksi non-idempoten tanpa `Idempotency-Key` ([api-overview.md](../api/api-overview.md)).

| Konteks | Maks percobaan | Backoff | Catatan |
|---|---|---|---|
| RPC read (view call) | 3 | eksponensial: 200 ms → 600 ms → 1.8 s (+ jitter ±20%) | Lalu failover provider berikutnya (FASTNEAR → official → dRPC) |
| RPC write (kirim tx) | 2 | 500 ms → 1.5 s | **Cek status tx via hash dulu** sebelum kirim ulang (hindari duplikat) |
| NearBlocks (polling) | 3 | 1 s → 3 s → 9 s | Setelah gagal → badge berhenti update (UI tetap hidup) |
| API klien (FE) | 2 | 300 ms → 900 ms | Hanya untuk `GET`; `POST/PATCH` retry hanya dengan `Idempotency-Key` |
| Deploy SSH/`docker pull` | 3 | 10 s → 20 s → 30 s | Hanya langkah jaringan (ci-cd.md §8) |

- **Tidak retry** pada: `4xx` selain `408`/`429`, kegagalan validasi, revert kontrak (`CHAIN_REVERT`), dan aksi destruktif admin tanpa step-up baru.
- `429` → hormati header `Retry-After` (detik); jangan retry sebelum waktu itu.
- Jitter wajib untuk mencegah thundering herd saat provider pulih.
- Batas total waktu retry satu aksi user ≤ 15 detik; setelah itu tampilkan error dan biarkan user memutuskan.

## 14. Toast (durasi & posisi)

Spesifikasi agar notifikasi tidak berantakan (§6) — nilai final dapat disetel saat sesi branding (TASK-008b), struktur berikut yang mengikat:

| Severity | Durasi tampil | Posisi | Aksi | Auto-dismiss |
|---|---|---|---|---|
| `info` | 4 detik | kanan-atas (desktop) / atas-tengah (mobile) | — | ya |
| `warning` | 6 detik | idem | tombol "Review" (opsional) | ya |
| `error` | **persistent** (sampai ditutup / aksi) | idem | tombol "Retry" (bila ada) + "Dismiss" | tidak |
| hasil tx `success` | 5 detik | idem | link ke tx/aset | ya |
| hasil tx `failed` | **persistent** | idem | "Retry" (bila aman) + "Dismiss" | tidak |

- **Maksimum 1 toast tampil** per aksi (§6); antrean sisanya ke panel notifikasi.
- Toast `error` persistent **tidak** menumpuk: beberapa error berurutan → yang lama dipindahkan ke panel, yang terbaru tampil.
- Posisi konsisten di seluruh aplikasi (satu komponen `Toast` dari design system, frontend-architecture §10).
- Toast **tidak** memuat `requestId`, stack trace, atau detail internal (§1/§7); boleh ada tombol "Copy details" yang menyalin `requestId` untuk laporan (tanpa menampilkannya sebagai teks utama).
- Aksesibilitas: `role="status"` untuk info/success, `role="alert"` untuk error; fokus tidak dirampas otomatis.

## 15. Versioning & deprecation kode error

- **Kode error stabil**: sekali dipublikasikan, kode **tidak diganti**. Perubahan makna = kode baru, kode lama ditandai deprecated.
- Kode deprecated: tetap dikembalikan minimal **1 MINOR** sebelum dihentikan (selaras [versioning-and-release.md](./versioning-and-release.md) §7), dicatat di CHANGELOG bagian `Deprecated`, dan kunci i18n-nya tetap ada sampai benar-benar dihapus.
- Menghapus kode = perubahan **MAJOR** artefak API (breaking untuk klien yang menangani kode itu) — kecuali kode tidak pernah dipakai di respons publik.
- Kode baru: tambahkan ke §3 + §10 + §9 (i18n) + [api/security-architecture] table bila menyangkut endpoint, dalam **satu PR**.
- Versi katalog kode mengikuti versi artefak yang mengembalikannya (API `/v1` → web); tidak ada versi terpisah untuk registry.
- Catatan migrasi kode lama → baru ditulis di CHANGELOG agar klien tahu penggantinya.

## 16. Status

- Envelope + namespace + siklus tx + kebijakan notifikasi — **DECIDED (ronde 14)**.
- Katalog i18n, status HTTP per kode, kode per domain, `requestId`, retry/backoff, toast, versioning kode (§9–§15) — **DECIDED (ronde 15)**.
- Registry kode dilengkapi saat implementasi API (Fase 2); pemetaan panic kontrak diuji di E2E.
- Modul error terpusat (§8) — **path dikunci TASK-001** (`frontend/lib/errors/`); modulnya sendiri dibuat bersama pemakaian pertamanya (TASK-008, TASK-033). TASK-007 menambah klasifikasi error **wallet** yang berdiri sendiri (`lib/near/wallet-errors.ts`) — lihat pengecualian di §8.
- Nilai tampilan toast (durasi/posisi) — **PROPOSED** (final saat branding TASK-008b); struktur §14 mengikat.
