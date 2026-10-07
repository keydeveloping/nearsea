# API Endpoints

> Kontrak API detail per endpoint (draft inti MVP — final disempurnakan saat implementasi). Frontend dan API bisa kerja paralel dari dokumen ini.

## Format template per endpoint

```md
### METHOD /path

Auth: required|optional|none

Request: { ... }
Response: { ... }
Errors: 400 INVALID_… / 401 / 404 / 429 …
```

## Endpoints

### Auth
```text
GET  /api/auth/nonce?account_id=…      challenge (NEP-413 / fallback)
POST /api/auth/verify                  verifikasi signature → session token
```

### Read (discovery) — ★ FASE 2 (butuh indexer; MVP: FE baca langsung dari RPC + NearBlocks)

> Koreksi kontradiksi: endpoint di bawah **belum ada di MVP**. Cakupan API per fase: [api-overview.md](./api-overview.md). Sumber proyeksi: [ADR-004](../decisions/ADR-004-mvp-data-layer.md), [ADR-015](../decisions/ADR-015-indexer-consistency.md).

```text
★ GET /api/collections                    daftar koleksi (?cursor&limit&verified=)
★ GET /api/collections/:id                detail + stats dasar
★ GET /api/tokens?collection=&sort=&cursor=&limit=   grid NFT (sort: newest/price_asc/price_desc)
★ GET /api/tokens/:contract/:tokenId      detail + listing + offers (+ fields: allowedBuyer|null, bundleId|null, isStale)
★ GET /api/listings?status=active&sort=&cursor=&limit=
★ GET /api/offers?account=…               offers masuk/keluar (market views + cache)
★ GET /api/bundles?status=active&cursor=  daftar bundle aktif (discovery)
★ GET /api/launchpad/:collection          phase aktif & berikutnya, status allowlist, progres mint
★ GET /api/bundles/:bundleId              detail bundle + item list
★ GET /api/accounts/:id/activity          riwayat (NearBlocks proxy/cache)
```

### Activity & profil (MVP — kecuali activity ★)

```text
★ GET  /api/accounts/:id/activity         riwayat (NearBlocks proxy/cache) — fase 2
GET  /api/accounts/:id/profile          profil publik (alias/bio/avatar)
PATCH /api/accounts/me/profile          update profil sendiri
  Auth: session scope `profile`
  Body: { alias?: string ≤32, bio?: string ≤280, avatarUrl?: string (gateway allowlist) }
  Errors: 400 INVALID_ALIAS / INVALID_BIO / INVALID_AVATAR ; 401 ; 429
```

### Moderasi (semua admin = session `admin` + STEP-UP SIGNATURE per aksi destruktif — SEC-ADMIN-003)
```text
POST  /api/reports                      kirim report (Auth: session scope `report`; idempotency per hari)
GET   /api/admin/reports                antrean (Auth: admin allowlist)
PATCH /api/admin/reports/:id           keputusan: {action: "hide_target"|"ignore", reason} — hide = sembunyikan TOKEN/KOLEKSI yang dilaporkan (bukan reportnya)
PATCH /api/admin/collections/:id/verified   set badge verified
GET   /api/admin/blocklist             daftar blocklist display-layer
POST  /api/admin/blocklist             tambah entri: { target_type, target_id, reason }
DELETE /api/admin/blocklist/:id        hapus entri
```

### Nilai uang

- Semua nilai chain dalam **string yoctoNEAR**. Format Ⓝ hanya di layer tampilan.

---

# Spesifikasi detail per endpoint

> Semua path di bawah memakai prefix versi `/api/v1` (penulisan ringkas `/api/…` di atas = `/api/v1/…`). Konvensi lintas-endpoint (pagination, error envelope, idempotency, CORS, caching): [api-overview.md](./api-overview.md). Auth & JWT: [authentication.md](./authentication.md).

## Konvensi umum

- **Query param**: tipe string/integer/enum; `limit` integer (default 20, maks 100); `cursor` opaque; param tak dikenal **diabaikan** (bukan error) kecuali dinyatakan lain.
- **Respons daftar** selalu berbentuk `{ "items": [...], "nextCursor": "<opaque>|null" }`.
- **Error**: envelope `{ "error": { "code", "message", "requestId" } }` — katalog [error-handling.md](../development/error-handling.md) §3.
- **Nilai yoctoNEAR** selalu string; **timestamp** ISO-8601 UTC.
- **Auth**: header `Authorization: Bearer <jwt>`; scope per endpoint ada di tabel di bawah.

## Tabel semantik status HTTP

| Status | Makna | Kapan dipakai |
|---|---|---|
| `200 OK` | Sukses | `GET`, `PATCH` yang mengembalikan resource |
| `201 Created` | Resource baru dibuat | `POST /reports`, `POST /admin/blocklist` |
| `204 No Content` | Sukses tanpa body | `POST /auth/logout`, `DELETE /admin/blocklist/:id` |
| `304 Not Modified` | Cache valid | `GET` dengan `If-None-Match` cocok (ETag) |
| `400 Bad Request` | Validasi input gagal | field/format/param invalid (`INVALID_*`) |
| `401 Unauthorized` | Auth gagal/tidak ada | signature/session/nonce (`AUTH_*`) |
| `403 Forbidden` | Terautentikasi tapi tidak berizin | scope/allowlist (`FORBIDDEN_*`) |
| `404 Not Found` | Resource tidak ada | (`NOT_FOUND_*`) |
| `409 Conflict` | Bentrok state / idempotency | (`CONFLICT_*`) |
| `415 Unsupported Media Type` | `Content-Type` salah | body bukan JSON |
| `429 Too Many Requests` | Rate limit / lockout | (`RATE_*`) + header `Retry-After` |
| `5xx` | Kesalahan server | (`SERVER_*`); detail hanya di log |

## Enum

**`sort` (discovery, ★ fase 2)**

| Nilai | Arti | Default |
|---|---|---|
| `newest` | Terbaru (listed/minted) | ✓ |
| `price_asc` | Harga naik | |
| `price_desc` | Harga turun | |

**`status` listing**

| Nilai | Arti |
|---|---|
| `active` | Listing aktif |
| `sold` | Terjual |
| `cancelled` | Dibatalkan seller |
| `stale` | Ownership mismatch → disembunyikan |

**`status` offer**

| Nilai | Arti |
|---|---|
| `active` | Offer aktif (escrow terkunci) |
| `accepted` | Diterima seller |
| `cancelled` | Dibatalkan buyer |
| `expired` | Lewat `expires_at` (refund) |
| `superseded` | Auto-cancel saat offer lain di-accept |

**`status` bundle**

| Nilai | Arti |
|---|---|
| `active` | Bundle aktif |
| `sold` | Semua token berpindah |
| `cancelled` | Dibatalkan seller |
| `pre_validate_failed` | Pre-validasi gagal → abort bersih + refund |
| `partial` | Kegagalan residual mid-loop → kompensasi G7 |

**`action` moderasi (admin)**

| Nilai | Arti |
|---|---|
| `hide_target` | Sembunyikan TOKEN/KOLEKSI yang dilaporkan (bukan report-nya) |
| `ignore` | Tidak ada tindakan |

**`target_type` (blocklist)**

| Nilai | Arti |
|---|---|
| `token` | Token (`contract_account_id` + `token_id`) |
| `collection` | Koleksi (`contract_account_id`) |

> Nilai enum `reason` report **belum final** → ⏳ open-by-design (dikunci saat implementasi; wajib enum tertutup, divalidasi server).

## Auth scope per endpoint

| Endpoint | Auth | Scope |
|---|---|---|
| `GET /api/v1/auth/nonce` | none | — |
| `POST /api/v1/auth/verify` | none (signature) | — |
| `POST /api/v1/auth/refresh` | refresh token | (dari sesi) |
| `POST /api/v1/auth/logout` | Bearer | scope token apa pun |
| `GET /api/v1/accounts/:id/profile` | none | — |
| `PATCH /api/v1/accounts/me/profile` | Bearer | `profile` |
| `POST /api/v1/reports` | Bearer | `report` |
| `GET /api/v1/admin/reports` | Bearer | `admin` + allowlist |
| `PATCH /api/v1/admin/reports/:id` | Bearer | `admin` + allowlist + step-up |
| `PATCH /api/v1/admin/collections/:id/verified` | Bearer | `admin` + allowlist + step-up |
| `GET /api/v1/admin/blocklist` | Bearer | `admin` + allowlist |
| `POST /api/v1/admin/blocklist` | Bearer | `admin` + allowlist + step-up |
| `DELETE /api/v1/admin/blocklist/:id` | Bearer | `admin` + allowlist + step-up |
| Discovery (★ fase 2) & `accounts/:id/activity` | none | — |

## Rate limit per endpoint

| Endpoint | Limit | Sumber |
|---|---|---|
| `GET /auth/nonce` | 10/IP/menit + 30/akun/jam | api-security-architecture §1 |
| `POST /auth/verify` | 10/IP/menit; **5 gagal/menit → lockout sementara** | api-security-architecture §1 |
| `POST /auth/refresh` | ⏳ open-by-design (usulan 20/IP/menit) | — |
| `GET` discovery (★ fase 2) | 60/IP/menit | api-security-architecture §1 |
| `GET /accounts/:id/activity` | 30/IP/menit | api-security-architecture §1 |
| `GET /accounts/:id/profile` | 60/IP/menit | api-security-architecture §1 |
| `POST /reports` | 5/akun/hari | api-security-architecture §1 |
| `PATCH /accounts/me/profile` | 10/akun/jam | api-security-architecture §1 |
| `POST/PATCH/DELETE /admin/*` | 30/menit per akun admin | api-security-architecture §1 |

- Semua endpoint juga tunduk **body size cap 16 KB** dan batas ukuran respons.
- Respons 429 selalu memuat header `Retry-After` (detik) + `error.code` dari namespace `RATE_*`.

## Mekanisme Idempotency-Key

- Header opsional `Idempotency-Key: <UUID v4>`; server menyimpan `(account_id, method, path, key) → response` 24 jam (retensi ⏳).
- Replay → respons tersimpan + `Idempotency-Replayed: true`; key sama body beda → 409 `CONFLICT_*`.
- `POST /reports` **wajib** idempoten lewat constraint DB unik per `(account, target, hari kalender)` — header hanya pelengkap ([api-security-architecture.md](../security/api-security-architecture.md) §1).
- Detail lintas-endpoint: [api-overview.md](./api-overview.md).

---

## Endpoint MVP — Auth

### `GET /api/v1/auth/nonce`

Auth: none · Rate: 10/IP/menit + 30/akun/jam

**Query param**

| Param | Tipe | Wajib | Batas |
|---|---|---|---|
| `account_id` | string | ya | format akun NEAR valid (named/sub/implicit/`0x`) |

**Response 200**

```json
{
  "nonce": "3f9a1c7d5b2e8f04a6c1d9e7b3f5028c4a7d1e6f9b0c3a5d8e2f4b7c1a9d6e30",
  "domain": "nearsea.example",
  "network": "testnet",
  "expires_at": "2026-10-02T12:05:00Z"
}
```

**Errors**: `400 INVALID_ACCOUNT_ID` · `429 RATE_LIMITED`

### `POST /api/v1/auth/verify`

Auth: none (signature) · Rate: 10/IP/menit; 5 gagal/menit → lockout

**Request body**

| Field | Tipe | Wajib | Catatan |
|---|---|---|---|
| `account_id` | string | ya | harus sama dengan nonce |
| `nonce` | string | ya | 32 byte hex |
| `message` | string | ya | template kanonik §3 wallet-authentication |
| `recipient` | string | ya | = domain server |
| `public_key` | string | ya | `ed25519:…` / `secp256k1:…` / ml-dsa |
| `signature` | string | ya | signature atas payload NEP-413 |

**Request**

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

**Response 200**

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

**Errors**: `400 INVALID_*` · `401 AUTH_NONCE_*` / `AUTH_SIGNATURE` / `AUTH_MESSAGE_MISMATCH` / `AUTH_KEY_NOT_FOUND` · `429 RATE_LIMITED`

### `POST /api/v1/auth/refresh`

Auth: refresh token · Rate: ⏳ open-by-design

**Request**: `{ "refresh_token": "rt_01HZX8..." }` → **Response 200**: pasangan access+refresh baru (rotasi). **Errors**: `401 AUTH_REFRESH_INVALID` / `AUTH_REFRESH_REUSED` · `403` untuk scope `admin` (tanpa silent refresh).

### `POST /api/v1/auth/logout`

Auth: Bearer · Rate: ⏳

**Request**: `{ "refresh_token": "rt_01HZX9..." }` → **Response 204**. **Errors**: `401 AUTH_SESSION_EXPIRED`.

---

## Endpoint MVP — Profil

### `GET /api/v1/accounts/:id/profile`

Auth: none · Rate: 60/IP/menit

**Response 200**

```json
{
  "account_id": "alice.testnet",
  "alias": "Alice",
  "bio": "NEAR NFT collector",
  "avatar_url": "https://ipfs.io/ipfs/bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
  "updated_at": "2026-10-02T12:00:00Z"
}
```

**Errors**: `404 NOT_FOUND_ACCOUNT` · `429 RATE_LIMITED`

### `PATCH /api/v1/accounts/me/profile`

Auth: Bearer scope `profile` · Rate: 10/akun/jam · Idempotency: last-write

**Request body** (semua opsional; hanya field terkirim yang diubah)

| Field | Tipe | Batas | Validasi |
|---|---|---|---|
| `alias` | string | ≤32 char | `400 INVALID_ALIAS` |
| `bio` | string | ≤280 char | `400 INVALID_BIO` |
| `avatar_url` | string | URL | gateway allowlist → `400 INVALID_AVATAR` |

```json
{ "alias": "Alice", "bio": "NEAR NFT collector" }
```

**Response 200**: objek profil lengkap (seperti `GET`).

**Errors**: `400 INVALID_ALIAS` / `INVALID_BIO` / `INVALID_AVATAR` · `401 AUTH_*` · `403 FORBIDDEN_SCOPE` · `429 RATE_LIMITED`

---

## Endpoint MVP — Reports

### `POST /api/v1/reports`

Auth: Bearer scope `report` · Rate: 5/akun/hari · Idempotency: unik `(account, target, hari)`

**Request body**

| Field | Tipe | Wajib | Catatan |
|---|---|---|---|
| `contract_account_id` | string | ya | target koleksi |
| `token_id` | string\|null | tidak | bila melaporkan token tertentu |
| `reason` | enum | ya | ⏳ nilai enum belum final |

```json
{
  "contract_account_id": "nft.example.testnet",
  "token_id": "42",
  "reason": "spam"
}
```

**Response 201**

```json
{
  "id": "rep_01HZX8Q2...",
  "status": "open",
  "created_at": "2026-10-02T12:00:00Z"
}
```

**Errors**: `400 INVALID_*` (reason/panjang/target) · `401` · `403 FORBIDDEN_SCOPE` · `404 NOT_FOUND_COLLECTION` / `NOT_FOUND_TOKEN` · `409 CONFLICT_REPORT_EXISTS` (sudah report target sama hari ini) · `429 RATE_LIMITED`

---

## Endpoint MVP — Admin

> Semua endpoint admin: Bearer scope `admin` + **allowlist DB** + audit `admin_audit`. Aksi destruktif menambah **step-up signature** (SEC-ADMIN-003). Bentuk transport step-up (header vs body) → ⏳ open-by-design; yang tetap: signature atas template aksi kanonik §3 wallet-authentication, diverifikasi ulang terhadap kunci aktif.

### `GET /api/v1/admin/reports`

Auth: admin · Rate: 30/menit

**Query param**: `status` (enum report status) · `cursor` · `limit`

**Response 200**

```json
{
  "items": [
    {
      "id": "rep_01HZX8Q2...",
      "reporter": "bob.testnet",
      "contract_account_id": "nft.example.testnet",
      "token_id": "42",
      "reason": "spam",
      "status": "open",
      "created_at": "2026-10-02T12:00:00Z"
    }
  ],
  "nextCursor": null
}
```

**Errors**: `401` · `403 FORBIDDEN_ADMIN` · `429`

### `PATCH /api/v1/admin/reports/:id`

Auth: admin + **step-up** · Rate: 30/menit

**Request body**

| Field | Tipe | Wajib | Catatan |
|---|---|---|---|
| `action` | enum | ya | `hide_target` \| `ignore` |
| `reason` | string | ya | bebas-text, wajib untuk audit |

```json
{
  "action": "report_decide",
  "decision": "hide_target",
  "reason": "confirmed scam collection",
  "step_up": {
    "message": "NearSea admin action\nDomain: nearsea.example\nNetwork: testnet\nAccount: admin.testnet\nAction: report_decide\nTarget: nft.example.testnet\nNonce: 9a1f...\nExpires: 2026-10-02T12:10:00Z",
    "public_key": "ed25519:7Kq3...",
    "signature": "ed25519:4mB9..."
  }
}
```

**Response 200**

```json
{
  "id": "rep_01HZX8Q2...",
  "status": "resolved",
  "action": "hide_target",
  "decided_by": "admin.testnet",
  "decided_at": "2026-10-02T12:01:00Z"
}
```

**Errors**: `400 INVALID_*` · `401 AUTH_*` · `403 FORBIDDEN_ADMIN` · `404 NOT_FOUND_REPORT` · `409 CONFLICT_ALREADY_DECIDED` · `429`

### `PATCH /api/v1/admin/collections/:id/verified`

Auth: admin + **step-up** · Rate: 30/menit

**Request**: `{ "verified": true, "reason": "verified partner", "step_up": { … } }` (Action kanonik: `verified_set`, Target: contract account id).
**Response 200**: `{ "contract_account_id": "nft.example.testnet", "verified": true, "decided_by": "admin.testnet", "decided_at": "…" }`.
**Errors**: `400` · `401` · `403` · `404 NOT_FOUND_COLLECTION` · `429`.

### `GET /api/v1/admin/blocklist`

Auth: admin · Rate: 30/menit

**Query param**: `target_type` (enum) · `cursor` · `limit`

```json
{
  "items": [
    {
      "id": "blk_01HZX8Q2...",
      "target_type": "collection",
      "target_id": "nft.example.testnet",
      "reason": "phishing",
      "decided_by": "admin.testnet",
      "decided_at": "2026-10-02T12:02:00Z"
    }
  ],
  "nextCursor": null
}
```

### `POST /api/v1/admin/blocklist`

Auth: admin + **step-up** · Rate: 30/menit

**Request**: `{ "target_type": "collection", "target_id": "nft.example.testnet", "reason": "phishing", "step_up": { … } }` (Action kanonik: `blocklist`).
**Response 201**: objek entri blocklist. **Errors**: `400 INVALID_*` · `403` · `404` · `409 CONFLICT_ALREADY_BLOCKED` · `429`.

### `DELETE /api/v1/admin/blocklist/:id`

Auth: admin + **step-up** · Rate: 30/menit · **Response 204**. **Errors**: `401` · `403` · `404 NOT_FOUND_BLOCKLIST` · `429`.

---

## Endpoint ★ FASE 2 — Discovery

> **Belum ada di MVP.** Cakupan & sumber: [api-overview.md](./api-overview.md), [ADR-004](../decisions/ADR-004-mvp-data-layer.md), [ADR-015](../decisions/ADR-015-indexer-consistency.md). Skema di bawah kontrak target; proyeksi indexer (Neardata) → PostgreSQL. Semua read publik, rate 60/IP/menit, pagination cursor, sort default `newest`.

### `★ GET /api/v1/collections`

**Query**: `verified` (bool) · `sort` (`newest`\|`volume_desc` ⏳) · `cursor` · `limit`

```json
{
  "items": [
    {
      "contract_account_id": "nft.example.testnet",
      "name": "Example Collection",
      "symbol": "EXMPL",
      "owner": "creator.testnet",
      "verified": true,
      "floor_price_yocto": "1000000000000000000000000",
      "volume_yocto": "50000000000000000000000000"
    }
  ],
  "nextCursor": null
}
```

**Errors**: `400 INVALID_CURSOR` · `429`

### `★ GET /api/v1/collections/:id`

**Response 200**: detail koleksi + stats dasar (supply, owners, floor, volume) + `base_uri`. **Errors**: `404 NOT_FOUND_COLLECTION` · `429`.

### `★ GET /api/v1/tokens`

**Query**: `collection` (contract id) · `sort` (`newest`\|`price_asc`\|`price_desc`) · `cursor` · `limit`

```json
{
  "items": [
    {
      "contract_account_id": "nft.example.testnet",
      "token_id": "42",
      "owner_id": "alice.testnet",
      "metadata": { "title": "Example #42", "media": "ipfs://bafy..." },
      "listing": { "price_yocto": "1000000000000000000000000", "status": "active" }
    }
  ],
  "nextCursor": null
}
```

### `★ GET /api/v1/tokens/:contract/:tokenId`

**Response 200**: detail + listing + offers (+ field `allowed_buyer`|null, `bundle_id`|null, `isStale`).

```json
{
  "contract_account_id": "nft.example.testnet",
  "token_id": "42",
  "owner_id": "alice.testnet",
  "metadata": { "title": "Example #42", "media": "ipfs://bafy..." },
  "listing": {
    "price_yocto": "1000000000000000000000000",
    "seller": "alice.testnet",
    "allowed_buyer": null,
    "bundle_id": null,
    "isStale": false,
    "status": "active"
  },
  "offers": [
    { "buyer": "bob.testnet", "amount_yocto": "900000000000000000000000", "expires_at": "2026-10-09T12:00:00Z" }
  ]
}
```

**Errors**: `404 NOT_FOUND_TOKEN` · `429`.

### `★ GET /api/v1/listings`

**Query**: `status` (`active` default) · `collection` · `sort` · `cursor` · `limit`

```json
{
  "items": [
    {
      "contract_account_id": "nft.example.testnet",
      "token_id": "42",
      "seller": "alice.testnet",
      "price_yocto": "1000000000000000000000000",
      "status": "active",
      "listed_at": "2026-10-01T12:00:00Z"
    }
  ],
  "nextCursor": null
}
```

### `★ GET /api/v1/offers`

**Query**: `account` (wajib) · `direction` (`incoming`\|`outgoing` ⏳) · `status` · `cursor` · `limit`

### `★ GET /api/v1/bundles`

**Query**: `status` (`active` default) · `cursor` · `limit`

```json
{
  "items": [
    { "bundle_id": "1", "seller": "alice.testnet", "price_yocto": "2000000000000000000000000", "status": "active", "item_count": 3 }
  ],
  "nextCursor": null
}
```

### `★ GET /api/v1/bundles/:bundleId`

**Response 200**: detail bundle + `items[]` (`contract_account_id`, `token_id`). **Errors**: `404 NOT_FOUND_BUNDLE`.

### `★ GET /api/v1/launchpad/:collection`

**Response 200**

```json
{
  "collection": "nft.example.testnet",
  "active_phase": {
    "phase_index": 1,
    "name": "Public",
    "price_yocto": "5000000000000000000000000",
    "allocation": 1000,
    "minted": 120,
    "max_per_wallet": 3,
    "starts_at": "2026-10-02T00:00:00Z",
    "ends_at": "2026-10-03T00:00:00Z",
    "allowlist_required": false
  },
  "next_phase": null
}
```

### `★ GET /api/v1/accounts/:id/activity`

Auth: none · Rate: 30/IP/menit · Sumber: NearBlocks proxy/cache (MVP: FE langsung; fase 2: endpoint ini).

**Response 200**

```json
{
  "items": [
    { "type": "nft_transfer", "contract_account_id": "nft.example.testnet", "token_id": "42", "tx_hash": "...", "block_height": 123456789, "timestamp": "2026-10-02T11:00:00Z" }
  ],
  "nextCursor": null
}
```

---

## Search — ★ FASE 2

### `★ GET /api/v1/search`

**Query param**

| Param | Tipe | Wajib | Default | Catatan |
|---|---|---|---|---|
| `q` | string | ya | — | min 2 char; full-text nama koleksi/token |
| `type` | enum | tidak | `all` | `all` \| `collection` \| `token` |
| `collection` | string | tidak | — | batasi ke satu koleksi |
| `cursor` | string | tidak | — | opaque |
| `limit` | integer | tidak | 20 | maks 100 |

**Response 200**

```json
{
  "items": [
    { "type": "collection", "contract_account_id": "nft.example.testnet", "name": "Example Collection" },
    { "type": "token", "contract_account_id": "nft.example.testnet", "token_id": "42", "title": "Example #42" }
  ],
  "nextCursor": null
}
```

**Errors**: `400 INVALID_QUERY` · `429`.

---

## Versioning

- Semua endpoint di dokumen ini = versi **`/v1`**; penulisan `/api/…` adalah shorthand `/api/v1/…`.
- Perubahan breaking → `/v2` (kebijakan lengkap: [api-overview.md](./api-overview.md) § Versi API).
- Endpoint/field deprecated ditandai `DEPRECATED` + header `Deprecation`/`Sunset`, minimal 1 MINOR sebelum dihapus.
