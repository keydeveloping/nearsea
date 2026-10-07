# Webhooks & Events

> Dua jenis "event" dalam sistem ini — jangan tertukar.

## 1. On-chain events (sumber kebenaran)

Event NEP-297 yang di-emit kontrak, format `EVENT_JSON:` (satu baris log per event):

```text
EVENT_JSON:{"standard":"nep171","version":"1.0.0","event":"nft_mint","data":[{"owner_id":"alice.near","token_ids":["1"]}]}
EVENT_JSON:{"standard":"nep171","version":"1.0.0","event":"nft_transfer","data":[{"old_owner_id":"alice.near","new_owner_id":"bob.near","token_ids":["1"]}]}
```

Custom market events (standard `x-nearsea-market`):

```text
market_list · market_delist · market_update_price · market_sale (buyer, price, payout)
market_offer · market_offer_accept · market_offer_cancel · market_offer_expire
market_stale_detected · market_bundle_partial · launchpad_phase_start · launchpad_mint (phase, price)
```

Semua pakai format `EVENT_JSON:` NEP-297 dengan `standard: "x-nearsea-market", version: "1.0.0"`.
Event identity untuk dedup/indexing = **(receipt_id, event_index)** — lihat [../security/indexer-security.md](../security/indexer-security.md).

## 2. Webhooks keluar

> **TIDAK ada di v1** (ronde 11-12): notifikasi user cukup in-app; alert admin via bot Telegram.
> Webhook keluar (Discord/email/dst.) ditunda sampai ada kebutuhan nyata.

## Pengaman (berlaku SAAT webhook keluar diaktifkan — pasca-v1)

- Webhook wajib HMAC secret + idempotency key **(receipt_id, event_index)** — bukan tx_hash (satu tx = banyak event).

---

# Skema payload event (field-level)

> Format on-chain: satu baris log `EVENT_JSON:` NEP-297 per event, `standard: "x-nearsea-market"`, `version: "1.0.0"`. Nilai chain = **string yoctoNEAR**. Nama field di bawah = **kontrak target**; field tambahan boleh ditambah (aditif) tanpa menaikkan versi, klien wajib mengabaikan field tak dikenal.

## Envelope event

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_sale",
  "data": [ { } ]
}
```

- `event` = nama event; `data` = array (umumnya 1 elemen per event).
- Identity/dedup = `(receipt_id, event_index)` — [indexer-security.md](../security/indexer-security.md) §1.

## Daftar lengkap event + parameter

> **SSOT katalog event = tabel ini.** Kolom **Pemitik** = kontrak yang mem-emit (market contract /
> NFT collection contract / factory). Event yang sama tidak boleh di-emit oleh dua kontrak.

| Event | Pemitik | `data[]` fields |
|---|---|---|
| `market_list` | market | `nft_contract_id`, `token_id`, `seller`, `price_yocto`, `approval_id`, `allowed_buyer` (string\|null) |
| `market_delist` | market | `nft_contract_id`, `token_id`, `seller` |
| `market_update_price` | market | `nft_contract_id`, `token_id`, `seller`, `old_price_yocto`, `new_price_yocto` |
| `market_sale` | market | `nft_contract_id`, `token_id`, `buyer`, `seller`, `price_yocto`, `payout` (object map `{"account": "amount"}`) |
| `market_offer` | market | `nft_contract_id`, `token_id`, `buyer`, `amount_yocto`, `expires_at` (u64 ns) |
| `market_offer_accept` | market | `nft_contract_id`, `token_id`, `buyer`, `seller`, `amount_yocto`, `payout` (object map) |
| `market_offer_cancel` | market | `nft_contract_id`, `token_id`, `buyer`, `amount_yocto` |
| `market_offer_expire` | market | `nft_contract_id`, `token_id`, `buyer`, `amount_yocto` |
| `market_stale_detected` | market | `nft_contract_id`, `token_id`, `seller`, `detected_owner`, `reason` (`ownership_mismatch` \| `approval_revoked`) |
| `market_purchase_recovered` | market | `nft_contract_id`, `token_id`, `buyer`, `refund_yocto`, `sale_restored` (bool) |
| `market_bundle_create` | market | `bundle_id`, `seller`, `price_yocto`, `items[]` (`{nft_contract_id, token_id}`) |
| `market_bundle_cancel` | market | `bundle_id`, `seller` |
| `market_bundle_partial` | market | `bundle_id`, `buyer`, `seller`, `transferred[]`, `failed[]`, `refund_yocto` |
| `market_pause` | market | `caller` (owner/guardian) |
| `market_unpause` | market | `caller` (owner-DAO) |
| `fee_update` | market | `old_fee_bps`, `new_fee_bps`, `caller` |
| `treasury_update` | market | `old_treasury`, `new_treasury`, `caller` |
| `treasury_withdraw` | market | `amount_yocto`, `treasury`, `caller` |
| `launchpad_phase_start` | NFT collection | `collection`, `phase_index`, `name`, `price_yocto`, `allocation`, `max_per_wallet`, `allowlist_required` |
| `launchpad_mint` | NFT collection | `collection`, `phase_index`, `account_id`, `token_ids[]`, `price_yocto` |
| `factory_collection_created` | factory | `collection` (sub-akun baru), `creator`, `royalty_bps`, `name` |

> Nama kanonik method kontrak (`make_offer`, `buy`, `accept_offer`, …) dimiliki [features/marketplace.md](../features/marketplace.md) § Contract API; reference implementasi per kontrak (init args, layout, konstanta) ada di [../contracts/](../contracts/market.md). Nama event di atas memakai bentuk aksi, bukan nama method. `launchpad_*` di-emit dari **NFT collection contract** (bukan market); `factory_collection_created` dari factory.

## Field-level schema per `market_*`

### `market_list`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_list",
  "data": [
    {
      "nft_contract_id": "nft.example.testnet",
      "token_id": "42",
      "seller": "alice.testnet",
      "price_yocto": "1000000000000000000000000",
      "approval_id": 7,
      "allowed_buyer": null
    }
  ]
}
```

### `market_delist`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_delist",
  "data": [
    { "nft_contract_id": "nft.example.testnet", "token_id": "42", "seller": "alice.testnet" }
  ]
}
```

### `market_update_price`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_update_price",
  "data": [
    {
      "nft_contract_id": "nft.example.testnet",
      "token_id": "42",
      "seller": "alice.testnet",
      "old_price_yocto": "1000000000000000000000000",
      "new_price_yocto": "1500000000000000000000000"
    }
  ]
}
```

### `market_sale`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_sale",
  "data": [
    {
      "nft_contract_id": "nft.example.testnet",
      "token_id": "42",
      "buyer": "bob.testnet",
      "seller": "alice.testnet",
      "price_yocto": "1000000000000000000000000",
      "payout": {
        "alice.testnet": "960000000000000000000000",
        "creator.testnet": "20000000000000000000000"
      }
    }
  ]
}
```

> Fee 2% dipotong on-chain (ADR-005); `payout` = **object map penerima final** (bentuk serialisasi NEP-199 `Payout`: `HashMap<AccountId, U128>` → JSON object; maks 10 receiver, Σ payout + fee ≤ harga — aturan di [order-protocol-security.md](../security/order-protocol-security.md) §5). Contoh di atas: harga 1 Ⓝ − fee 0.02 − royalti 0.02 = seller 0.96.

### `market_offer`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_offer",
  "data": [
    {
      "nft_contract_id": "nft.example.testnet",
      "token_id": "42",
      "buyer": "bob.testnet",
      "amount_yocto": "900000000000000000000000",
      "expires_at": 1799016000000000000
    }
  ]
}
```

### `market_offer_accept`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_offer_accept",
  "data": [
    {
      "nft_contract_id": "nft.example.testnet",
      "token_id": "42",
      "buyer": "bob.testnet",
      "seller": "alice.testnet",
      "amount_yocto": "900000000000000000000000",
      "payout": {
        "alice.testnet": "864000000000000000000000",
        "creator.testnet": "18000000000000000000000"
      }
    }
  ]
}
```

### `market_offer_cancel`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_offer_cancel",
  "data": [
    { "nft_contract_id": "nft.example.testnet", "token_id": "42", "buyer": "bob.testnet", "amount_yocto": "900000000000000000000000" }
  ]
}
```

### `market_offer_expire`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_offer_expire",
  "data": [
    { "nft_contract_id": "nft.example.testnet", "token_id": "42", "buyer": "bob.testnet", "amount_yocto": "900000000000000000000000" }
  ]
}
```

> `market_offer_expire` di-emit saat refund lazy dieksekusi (accept/cancel/withdraw menyentuh offer) — **bukan** saat `expires_at` terlewat, karena MVP tidak punya cron.

### `market_stale_detected`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_stale_detected",
  "data": [
    {
      "nft_contract_id": "nft.example.testnet",
      "token_id": "42",
      "seller": "alice.testnet",
      "detected_owner": "carol.testnet"
    }
  ]
}
```

### `market_bundle_partial`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "market_bundle_partial",
  "data": [
    {
      "bundle_id": "1",
      "buyer": "bob.testnet",
      "seller": "alice.testnet",
      "transferred": [ { "nft_contract_id": "nft.example.testnet", "token_id": "42" } ],
      "failed": [ { "nft_contract_id": "nft.example.testnet", "token_id": "43" } ],
      "refund_yocto": "500000000000000000000000"
    }
  ]
}
```

> `market_bundle_partial` = residual mid-loop failure (sangat jarang) — NEAR tidak punya rollback lintas-receipt; kompensasi G7 ([order-protocol-security.md](../security/order-protocol-security.md) §6).

### `launchpad_phase_start`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "launchpad_phase_start",
  "data": [
    {
      "collection": "nft.example.testnet",
      "phase_index": 1,
      "name": "Public",
      "price_yocto": "5000000000000000000000000",
      "allocation": 1000,
      "max_per_wallet": 3,
      "allowlist_required": false
    }
  ]
}
```

### `launchpad_mint`

```json
{
  "standard": "x-nearsea-market",
  "version": "1.0.0",
  "event": "launchpad_mint",
  "data": [
    {
      "collection": "nft.example.testnet",
      "phase_index": 1,
      "account_id": "alice.testnet",
      "token_ids": ["1", "2"],
      "price_yocto": "10000000000000000000000000"
    }
  ]
}
```

### Event bundle, pause, fee/treasury & factory (ditambahkan ronde 16)

Contoh payload event yang sebelumnya open-by-design — nama final ditetapkan ronde 16 (ADR-001).
`expires_at` di semua event = **u64 nanodetik** (selaras `env::block_timestamp()`).

```json
{
  "standard": "x-nearsea-market", "version": "1.0.0", "event": "market_bundle_create",
  "data": [ { "bundle_id": 7, "seller": "alice.testnet",
              "price_yocto": "3000000000000000000000000",
              "items": [ { "nft_contract_id": "nft.example.testnet", "token_id": "42" },
                         { "nft_contract_id": "nft.example.testnet", "token_id": "43" } ] } ]
}
```

```json
{ "standard": "x-nearsea-market", "version": "1.0.0", "event": "market_bundle_cancel",
  "data": [ { "bundle_id": 7, "seller": "alice.testnet" } ] }
```

```json
{ "standard": "x-nearsea-market", "version": "1.0.0", "event": "market_pause",
  "data": [ { "caller": "nearsea.testnet" } ] }
```

```json
{ "standard": "x-nearsea-market", "version": "1.0.0", "event": "treasury_withdraw",
  "data": [ { "amount_yocto": "500000000000000000000000",
              "treasury": "treasury.nearsea.testnet", "caller": "nearsea.testnet" } ] }
```

```json
{ "standard": "x-nearsea-market", "version": "1.0.0", "event": "factory_collection_created",
  "data": [ { "collection": "punks.nearsea.testnet", "creator": "artist.testnet",
              "royalty_bps": 500, "name": "Punks" } ] }
```

> `fee_update` / `treasury_update` / `market_unpause` memakai pola yang sama (`old_*`, `new_*`, `caller`).
> `caller` = akun yang memicu (owner MVP / guardian `pause_callers` untuk pause di mainnet — ADR-013).

## Event ordering & jaminan emisi

| Jaminan | Nilai |
|---|---|
| Urutan | Deterministik: `block_height` naik → urutan receipt dalam blok → `event_index` naik ([indexer-security.md](../security/indexer-security.md) §1) |
| Finality | Ingest data uang **hanya blok `final`** (tanpa reorg di belakang final — FACT) |
| Dedup | Idempotent upsert by `(receipt_id, event_index)` |
| Duplikat | Mungkin terjadi (retry ingestion) → konsumen **wajib** idempoten |
| At-least-once | Ya (bukan exactly-once) — konsumen harus toleran duplikat |
| Ordering lintas receipt | Tidak dijamin lintas shard/receipt berbeda selain urutan blok di atas |
| Delivery webhook | ⏳ belum ada di v1 — spec di bawah berlaku saat diaktifkan |

---

# Webhook keluar (pasca-v1) — spesifikasi delivery

> **Status: TIDAK ada di v1** (ronde 11-12). Bagian ini adalah spec **saat diaktifkan** agar desain tidak tambal-sulam. Semua nilai operasional yang belum diputuskan ditandai `⏳ open-by-design`.

## Registrasi endpoint & rotasi secret

- Registrasi webhook = aksi **admin** (scope `admin` + allowlist + audit `admin_audit`); tabel pendukung ⏳ open-by-design.
- URL wajib **HTTPS**; URL non-HTTPS ditolak saat registrasi.
- Per-endpoint: `secret` (HMAC), `event_filter[]` (subset nama event), `active` (bool).
- **Rotasi secret**: dukung **dua secret aktif** (primary + secondary) selama masa transisi; pengirim menandatangani dengan primary, penerima menerima keduanya; setelah transisi secondary dihapus. ⏳ Durasi transisi open-by-design.
- Secret disimpan terenkripsi; tidak pernah dikembalikan penuh setelah dibuat (hanya sekali tampil).

## Header & HMAC canonicalization

| Header | Isi |
|---|---|
| `X-NearSea-Event` | Nama event, mis. `market_sale` |
| `X-NearSea-Delivery` | ID unik pengiriman (UUID) — untuk dedup/observability |
| `X-NearSea-Timestamp` | Unix detik saat signing (bukan saat emit event) |
| `X-NearSea-Signature` | `sha256=<hex>` dari signing string di bawah |
| `Content-Type` | `application/json; charset=utf-8` |

**Signing string** (persis, `\n` = newline):

```text
<timestamp>.<raw_request_body>
```

- `signature = HMAC-SHA256(secret, signing_string)` → hex lowercase, prefiks `sha256=`.
- Body **raw** yang di-sign adalah byte yang dikirim (tanpa re-serialisasi). Penerima **wajib** memverifikasi atas byte mentah, lalu parse JSON.
- **Timestamp tolerance**: tolak bila `|now − timestamp| > 300 detik` (⏳ final saat implementasi) untuk mencegah replay.
- Verifikasi signature **constant-time**; bandingkan juga saat transisi dua-secret (terima bila cocok dengan salah satu).

Contoh header:

```http
POST /webhooks/nearsea HTTP/1.1
Host: merchant.example
Content-Type: application/json; charset=utf-8
X-NearSea-Event: market_sale
X-NearSea-Delivery: 01HZX8Q2-9f3a-4b7c-8d1e-2a5b6c7d8e9f
X-NearSea-Timestamp: 1759406400
X-NearSea-Signature: sha256=3b1f...e9a2
```

## Contoh payload keluar

```json
{
  "id": "01HZX8Q2-9f3a-4b7c-8d1e-2a5b6c7d8e9f",
  "type": "market_sale",
  "version": "1.0.0",
  "created_at": "2026-10-02T12:00:00Z",
  "network": "testnet",
  "data": {
    "event_id": { "receipt_id": "9f8E...", "event_index": 2 },
    "block_height": 123456789,
    "nft_contract_id": "nft.example.testnet",
    "token_id": "42",
    "buyer": "bob.testnet",
    "seller": "alice.testnet",
    "price_yocto": "1000000000000000000000000",
    "payout": {
      "alice.testnet": "960000000000000000000000",
      "creator.testnet": "20000000000000000000000"
    }
  }
}
```

- `id` = sama dengan header `X-NearSea-Delivery`.
- `data.event_id` = kunci idempotensi konsumen `(receipt_id, event_index)`.
- `version` = versi **payload webhook** (lihat kebijakan versi di bawah), terpisah dari versi kontrak & versi event NEP-297.

## Delivery, retry & backoff

| Aspek | Nilai |
|---|---|
| Metode | `POST` JSON |
| Timeout per percobaan | 10 detik ⏳ |
| Sukses | HTTP `2xx` |
| Retry | `4xx` selain 408/429 → **tidak** di-retry (config salah); `408`/`429`/`5xx`/timeout/koneksi gagal → retry |
| Backoff | eksponensial + jitter, mis. 1s, 5s, 30s, 2m, 10m, 1h, 6h (⏳ final) |
| Maks percobaan | ⏳ open-by-design (usulan 8) |
| At-least-once | Ya; konsumen **wajib** dedup via `id`/`(receipt_id, event_index)` |
| Urutan | **Tidak dijamin** antar delivery; konsumen memakai `block_height` untuk urutan |

## Dead Letter Queue & observability

- Setelah percobaan habis → delivery masuk **DLQ**; admin dapat inspeksi & **replay manual** (aksi admin + audit).
- Retensi DLQ ⏳ open-by-design (usulan 7 hari).
- Observability: metrik sukses/gagal per endpoint, latensi, kedalaman DLQ; alert Telegram untuk kegagalan beruntun ([deployment/monitoring.md](../deployment/monitoring.md)).
- Auto-disable endpoint setelah N kegagalan beruntun (usulan), notifikasi admin; re-enable manual.

## Kebijakan versi payload

- `version` payload webhook mengikuti SemVer.
- Perubahan **aditif** (field baru opsional, event baru) = MINOR — tidak memutus konsumen.
- Perubahan **breaking** (hapus/ubah tipe/ubah semantik field) = MAJOR; endpoint lama tetap didukung selama jendela deprecation (⏳).
- Konsumen **wajib** mengabaikan field yang tidak dikenal.
- Versi payload webhook, versi event NEP-297, dan versi artefak web (`web-vX.Y.Z`) adalah **tiga hal terpisah** ([versioning-and-release.md](../development/versioning-and-release.md) §4).

## Status

- Skema payload event (field-level) + urutan emisi + HMAC canonicalization + retry/DLQ + kebijakan versi — **DECIDED (dokumen ini)** sebagai kontrak target.
- **Webhook keluar belum aktif di v1** — seluruh bagian delivery bersifat spec pasca-v1.
- Nilai operasional bertanda `⏳ open-by-design`: timeout, jadwal backoff, maks percobaan, retensi DLQ, durasi rotasi secret, jendela deprecation.
