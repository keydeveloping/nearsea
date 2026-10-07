# Feature — Notifications

## Objective

User tahu kejadian penting: offer diterima, token terjual, offer di-accept, offer expire, mint berhasil. (Bid/auction = fase 2.)

## Sumber data (DIPUTUSKAN ronde 6: in-app minimal SEJAK MVP)

- **MVP**: polling — NearBlocks API (riwayat tx milik user) + view call market contract (offers aktif untuk saya) → daftar notifikasi in-app. Interval: **30–60 detik saat tab aktif** (final, bukan draft).
- **Mekanisme per tipe**: offer diterima/di-accept/terjual = diff riwayat tx NearBlocks; offer expire = cek expiry lokal dari view `get_offers` (tidak ada tx on-chain untuk expire); mint berhasil = diff riwayat mint.
- **Fase 2 (custom indexer)**: ganti polling dengan event dari indexer Neardata — real-time, tanpa beban API pihak ketiga.

## Kanal (kandidat)

> **v1 (ronde 6)**: in-app saja — bell + unread badge + dropdown/halaman notifikasi.
> Email / Telegram / push browser = fase lanjutan (butuh PII & service eksternal).

## UI

- Bell di header + unread badge; dropdown daftar + halaman `/notifications` (opsional).
- Tipe notifikasi: offer diterima, offer di-accept, token terjual, offer expire, mint berhasil, **offer di-cancel otomatis + refund** (saat offer lain di-accept pada token yang sama).

## Model data (MVP — tanpa server)

- Notifikasi dibangun client-side: TanStack Query polling (30–60 dtk, tab aktif) → diff activity → item notifikasi.
- Read-state disimpan di **localStorage** per akun (tanpa DB di MVP).
- Fase 2 (indexer): tabel `notifications` di Postgres, unread permanen lintas device.

## Skema Payload Notifikasi (JSON)

> Ini **model internal client** (bukan skema event on-chain — skema event dimiliki [webhooks.md](../api/webhooks.md)). Nilai chain tetap string yoctoNEAR.

```json
{
  "id": "9f8E3a1b...:2",
  "event_id": { "receipt_id": "9f8E3a1b...", "event_index": 2 },
  "type": "offer_received",
  "account_id": "alice.testnet",
  "created_at": "2026-10-02T12:00:00Z",
  "block_height": 123456789,
  "read": false,
  "payload": {
    "contract_account_id": "nft.example.testnet",
    "token_id": "42",
    "counterparty": "bob.testnet",
    "amount_yocto": "900000000000000000000000",
    "expires_at": "2026-10-09T12:00:00Z"
  },
  "destination": "/token/nft.example.testnet/42"
}
```

| Field | Tipe | Wajib | Deskripsi |
|---|---|---|---|
| `id` | string | ya | `${receipt_id}:${event_index}` — kunci idempotensi & kunci read-state. |
| `event_id.receipt_id` | string | ya | Receipt NEAR sumber event. |
| `event_id.event_index` | number | ya | Indeks event dalam receipt (satu tx bisa memuat banyak event). |
| `type` | enum | ya | Lihat tabel tipe di bawah. |
| `account_id` | string | ya | Pemilik kotak notifikasi (viewer). |
| `created_at` | string | ya | ISO-8601 UTC (dari blok/timestamp tx). |
| `block_height` | number | ya | Untuk ordering deterministik. |
| `read` | boolean | ya | Diturunkan dari read-set (bukan disimpan per item). |
| `payload` | object | ya | Field spesifik per `type` (lihat tabel). |
| `destination` | string | ya | Route internal untuk click-through. |

**Tipe → payload + destinasi:**

| `type` | `payload` tambahan | `destination` |
|---|---|---|
| `offer_received` | `contract_account_id`, `token_id`, `counterparty` (buyer), `amount_yocto`, `expires_at` | `/token/:contract/:tokenId` (panel offers) |
| `offer_accepted` | `contract_account_id`, `token_id`, `counterparty` (seller), `amount_yocto` | `/token/:contract/:tokenId` |
| `offer_cancelled` | `contract_account_id`, `token_id`, `counterparty`, `amount_yocto` | `/token/:contract/:tokenId` |
| `offer_expired` | `contract_account_id`, `token_id`, `amount_yocto` | `/token/:contract/:tokenId` |
| `token_sold` | `contract_account_id`, `token_id`, `counterparty` (buyer), `price_yocto` | `/token/:contract/:tokenId` |
| `mint_success` | `contract_account_id`, `token_ids[]`, `phase_index` | `/profile/:account?tab=owned` |
| `listing_stale` | `contract_account_id`, `token_id` | `/token/:contract/:tokenId` |

## Algoritma Unread Count

```text
function unreadCount(items[], readSet):
  count = 0
  for item in items:
    if item.id not in readSet:
      count += 1
  return count

# readSet = Set(id) dari localStorage (lihat skema kunci)
# badge menampilkan min(count, 99) → "99+" bila lebih
```

- `read` per item **diturunkan** (`id ∈ readSet`) — tidak disimpan boolean per item (hemat storage, idempoten).
- Menandai dibaca: tambah `id` ke `readSet` (union), tulis kembali ke localStorage.
- "Tandai semua dibaca": `readSet ∪= { item.id }` untuk semua item yang sedang tampil.
- Pruning: `readSet` dibatasi **500 id terbaru** (atau TTL 30 hari) agar tidak tumbuh tanpa batas; id lama yang hilang tidak akan muncul lagi karena item juga dipangkas.

## Skema Kunci localStorage + Versioning

```text
nearsea:notif:v1:<network>:<account_id>:read        → JSON array of id (readSet)
nearsea:notif:v1:<network>:<account_id>:lastCursor  → cursor pagination terakhir (opaque)
nearsea:notif:v1:<network>:<account_id>:seenAt      → ISO-8601, kapan terakhir buka panel
```

- `v1` = **versi skema**; saat skema berubah breaking → naikkan ke `v2` dan **abaikan** kunci `v1` (tanpa migrasi paksa; read-state boleh reset).
- `<network>` (testnet/mainnet) disertakan agar state testnet tidak bocor ke mainnet (satu browser bisa dua network).
- `<account_id>` mengisolasi per akun — ganti akun → baca read-set akun lain.
- Semua operasi tulis **union** (merge), bukan overwrite, untuk mencegah kehilangan read-state antar tab.
- Nilai invalid/parse gagal → perlakukan sebagai kosong + tulis ulang (jangan crash).
- Selaras frontend-architecture.md §11 (read-state di localStorage per akun; **session JWT tidak** di localStorage).

## Cross-Tab Sync

- Dengarkan event `storage` (`window.addEventListener('storage', …)`) → bila kunci `:read` berubah, merge ke store + recompute unread.
- Store: Zustand `notifications` (wallet-ui/modal store terpisah) — sumber tunggal untuk badge & dropdown.
- Tulis selalu **union** → dua tab tidak saling menimpa (last-writer-wins pada union = aman).
- `BroadcastChannel` sebagai fallback real-time antar tab = ⏳ open-by-design (opsional; `storage` event cukup untuk MVP).
- Satu tab aktif melakukan polling; tab lain mengandalkan `storage` event (hindari polling ganda).

## Ordering & Dedup Window

- **Urutan**: `block_height` **desc**, lalu urutan receipt dalam blok, lalu `event_index` **desc** (terbaru di atas) — deterministik, selaras [webhooks.md](../api/webhooks.md) § Event ordering.
- **Dedup**: set id `(receipt_id, event_index)` — **bukan** `tx_hash` (satu tx bisa memuat banyak event).
- **Jendela dedup**: id disimpan di `readSet` + set `seenIds` (union) dan dipangkas ke **500 id terbaru / 30 hari**; item lebih tua dibuang bersama id-nya.
- **Polling overlap**: diff dihitung terhadap `seenIds`; event yang sama dari polling ulang tidak menghasilkan item baru (idempoten, at-least-once tolerant).
- **Window fetch**: ambil maksimum **100 event terbaru** per poll (di atas itu, sisanya ditunda ke poll berikutnya via cursor).

## NearBlocks — Endpoint & Parameter yang Dipakai

> Sumber riwayat tx akun (MVP tanpa indexer; ADR-004). Base: `https://api.nearblocks.io` ([system-architecture.md](../architecture/system-architecture.md) § Port; CSP `connect-src` di [frontend-security.md](../security/frontend-security.md) §2). API key opsional.

| Keperluan | Parameter | Catatan |
|---|---|---|
| Riwayat tx akun (outgoing/incoming) | `account = <account_id>`, filter `type = nft`, `page`/`per_page` (atau cursor) | Untuk diff `market_offer`, `market_offer_accept`, `market_sale`, mint |
| Filter event market | cocokkan `EVENT_JSON:` `standard: "x-nearsea-market"` pada log receipt | Ekstrak `(receipt_id, event_index)` |
| Rate limit | backoff saat 429; retry eksponensial | Badge berhenti update, UI tetap hidup |

- **Path spesifik endpoint NearBlocks ⏳ open-by-design** — dikunci saat implementasi mengikuti dokumentasi NearBlocks yang berlaku; yang dikunci di sini adalah **param** (account + filter NFT) dan **pola** (diff by event id).
- Polling interval **30–60 detik saat tab aktif** (final); tab tidak aktif → polling berhenti (hemat kuota).
- Offer expire **tidak** dari NearBlocks (tidak ada tx) — dari cek expiry lokal atas `get_offers` (view).

## Pagination (Daftar Notifikasi)

- Halaman `/notifications` + dropdown memakai pola `{items[], nextCursor}` (cursor opaque) — konsisten dengan [endpoints.md](../api/endpoints.md).
- Page size: **20** item per muat; "load more" memakai cursor tersimpan (`:lastCursor`).
- Badge unread dihitung dari item yang **sudah di-fetch** (MVP, maks 100/poll); fase 2 (indexer) menghitung unread server-side.
- Item read tetap ditampilkan (tidak hilang) — hanya tidak menambah badge.

## Bell Dropdown — State

| State | Kondisi | Tampilan |
|---|---|---|
| `idle-empty` | Tidak ada item | Bell tanpa badge + empty state "No notifications yet" |
| `idle-read` | Ada item, semua terbaca | Bell tanpa badge |
| `unread` | Ada item belum dibaca | Bell + badge `n` (cap `99+`) |
| `loading` | Fetch pertama / refresh | Skeleton list |
| `stale` | NearBlocks gagal/rate-limit | Badge berhenti update + indikator "gagal menyegarkan" (list lama tetap tampil) |
| `error` | Parse/gagal total | Empty state + tombol retry |

- Klik item → tandai dibaca + navigasi ke `destination` (tabel tipe).
- Buka panel → **tidak** otomatis menandai semua dibaca (harus aksi eksplisit) kecuali "mark all read".

## Nuansa "Offer Received" vs "Offer Cancelled"

> Empat peristiwa offer berbeda dan **tidak boleh** disatukan — aktor & maknanya beda.

| Notifikasi | Aktor | Event sumber | Dilihat oleh | Makna |
|---|---|---|---|---|
| `offer_received` | Buyer `make_offer` | `market_offer` | **Seller** | "Ada yang menawar tokenmu" (tawaran masuk, escrow terkunci) |
| `offer_cancelled` | Buyer `cancel_offer` | `market_offer_cancel` | Seller | "Penawar membatalkan tawarannya" — **bukan** ditolak seller |
| `offer_cancelled` (auto) | Sistem (offer lain di-accept) | `market_offer_cancel` / superseded | Buyer (yang ter-supersede) | "Tawaranmu batal otomatis karena token terjual ke penawar lain" |
| `offer_expired` | Sistem (lazy) | `market_offer_expire` | Buyer | "Tawaranmu kedaluwarsa — dana kembali" |
| `offer_accepted` | Seller `accept_offer` | `market_offer_accept` | **Buyer** | "Tawaranmu diterima" |

- `offer_received` (tawaran **masuk**) ≠ `offer_accepted` (tawaran **diterima**) — dua tipe berbeda, jangan dicampur.
- `offer_cancelled` oleh buyer ≠ `offer_expired` (kedaluwarsa) ≠ `offer_accepted`; refund terjadi di ketiganya tapi pesan user berbeda.
- Auto-cancel (superseded) adalah `offer_cancelled` dengan `payload.reason = "superseded"` (⏳ field opsional) — jika tidak dibedakan, minimal `counterparty`/konteks menjelaskan penyebab.

## Error Cases

| Kasus | Perlakuan |
|---|---|
| NearBlocks gagal/rate-limit | badge berhenti update, retry backoff, UI tetap berfungsi |
| Notifikasi duplikat | idempotency via event id **(receipt_id, event_index)** (set terlihat di localStorage) — bukan `tx_hash` (satu tx bisa memuat banyak event) |

## Acceptance Criteria

- Given seller punya offer baru, When ada yang make_offer, Then bell menampilkan 1 unread dalam ≤ 60 detik.
- Given notifikasi dari event id (receipt_id, event_index) identik sudah ada, When polling berikutnya, Then tidak terjadi duplikat (idempotency).
- Given offer milik user lewat expire, When FE mengecek expiry lokal, Then notifikasi "offer expired" muncul tanpa menunggu tx on-chain.
