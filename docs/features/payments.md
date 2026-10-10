# Feature — Payments (NEAR & FT settlement)

> Pembayaran marketplace. Prinsip: tidak ada backend payment — semua lewat kontrak.

## MVP — DIPUTUSKAN (ronde 1): NEAR saja

- Buyer attach deposit = harga ke `buy` (listing aktif) atau escrow via `make_offer` → `accept_offer`.
- Market → `nft_transfer_payout` → Payout (seller + royalti) → `resolve_purchase` distribusi/refund.
- **Fee platform 2%** dipotong on-chain sebelum distribusi, dikirim ke treasury.
> ⏳ **OPEN-BY-DESIGN**: alamat treasury account ditetapkan saat deploy testnet (sementara = owner market contract).

## Multi-currency (FT, NEP-141) — FASE 2 (bukan MVP, ronde 1)

- Harga listing dinotasikan (ft_contract, amount).
- Buyer `ft_transfer_call` ke market → `ft_on_transfer` (validasi predecessor = FT whitelist) → settle.
- Perlu juga storage deposit FT di market untuk tiap token yang diterima.
- Referensi: RESEARCH.md 11.1 #6; FT tutorial langkah 7.

## Aturan

- Nilai selalu string u128; refund melewati resolve callback; dilarang transfer manual di frontend.

## Algoritma Distribusi Payout (listing & accept offer)

> Otoritas perhitungan = kontrak (on-chain). Fee dipotong **sebelum** distribusi (ADR-005). Semua aritmetika **integer u128**, `floor` (pembagian integer); tidak ada float. Semua hasil = string yoctoNEAR.

```text
function settle(price, fee_bps, nft_payout[]):
  # 1) fee platform — dipotong lebih dulu
  fee      = floor(price * fee_bps / 10_000)        # fee_bps = 200 (2%); cap MAX_FEE_BPS = 500
  max_pay  = price - fee                            # plafon payout (royalti + seller)

  # 2) payout dari NFT contract (NEP-199, UNTRUSTED) — validasi ketat
  assert 1 <= len(nft_payout) <= 10                 # INV-003/021
  assert receiver unik                              # duplikat → digabung dulu (INV-003 DEFAULT: merge)
  assert setiap amount > 0
  Σpay     = sum(nft_payout[i].amount)
  assert Σpay <= max_pay                            # INV-002 (tidak boleh melebihi plafon)

  # 3) seller menerima SELURUH residual — bukan hanya dust (lihat §Koreksi ronde 23)
  seller   = max_pay - Σpay                         # ≥ 0 karena Σpay ≤ max_pay
  distributions = nft_payout + [ (seller, seller), (treasury, fee) ]
  assert sum(distributions.amount) == price         # internal, exact
```

- **Koreksi ronde 23 — residual seller BUKAN dust.** Sebelumnya §ini meng-`assert sisa in {0, 1}`.
  Itu berasal dari model tutorial (RESEARCH.md §10.5) di mana kontrak NFT mengembalikan **seluruh**
  distribusi (seller + royalti) sehingga `max_pay − Σpay` hanya sisa pembulatan. **NearSea tidak begitu**:
  kontrak koleksi mengembalikan **hanya royalti kreator** ([contracts/nft-collection.md](../contracts/nft-collection.md) §4 —
  `Payout::from([(creator_id, amount)])`), dan market yang menambahkan seller ke payout map
  ([contracts/market.md](../contracts/market.md) §3). Akibatnya `max_pay − Σpay` = **proceeds seller**,
  yang wajar bernilai besar (mis. royalti 5% → residual 93% harga). Meng-`assert sisa ≤ 1 yocto` akan
  **menolak setiap penjualan normal** — bertentangan dengan TC-002 dan AC TASK-005.
  Yang divalidasi market tetap: `Σpay ≤ max_pay`, `amount > 0`, `1 ≤ len ≤ 10`.
- **Aturan pembulatan**: setiap rate (fee & royalti) di-`floor` independen; **seluruh residual jatuh ke
  seller** sehingga `fee + Σroyalti + seller == price` persis (aritmetika internal, exact).
- Royalti **per token** di-cap 10% dari harga wajarnya (INV-027); agregat bundle = penjumlahan tanpa cap.
- Validasi gagal di langkah mana pun → tolak + **refund penuh** ke buyer (`resolve_purchase`, INV-001..003).

### Contoh terhitung — price 10 Ⓝ, fee 2%, royalti 5% + 2.5%

`1 Ⓝ = 1000000000000000000000000`; `10 Ⓝ = 10000000000000000000000000`.

| Komponen | Rate | Perhitungan | Nilai (yocto) |
|---|---|---|---|
| Harga | — | — | `10000000000000000000000000` |
| Fee platform | 2% (200 bps) | `floor(10 Ⓝ × 200 / 10000)` | `200000000000000000000000` |
| Plafon payout | — | `price − fee` | `9800000000000000000000000` |
| Royalti A | 5% (500 bps) | `floor(10 Ⓝ × 500 / 10000)` | `500000000000000000000000` |
| Royalti B | 2.5% (250 bps) | `floor(10 Ⓝ × 250 / 10000)` | `250000000000000000000000` |
| Σpayout (dari koleksi) | — | A + B | `750000000000000000000000` |
| Seller (residual) | — | `max_pay − Σpayout` | `9050000000000000000000000` |

- Verifikasi: `fee + royalti A + royalti B + seller = 0.2 + 0.5 + 0.25 + 9.05 = 10 Ⓝ` ✓ (exact)
- `Σpayout` (A + B) = `750000000000000000000000` ≤ `max_pay` ✓; penerima = 2 ≤ 10 ✓.
- Residual seller = 9.05 Ⓝ — **besar dan sah**, bukan pelanggaran INV-002.

### Contoh dust pada sisi royalti (bukan pada residual)

Dust hanya muncul di aritmetika **rate**: `royalti = floor(basis × rate / 10_000)` bisa menghasilkan `0`
bila basis di bawah granularitas rate (mis. `< 20` yocto di 500 bps). Karena `amount == 0` ditolak
(`∀p.amount > 0`, INV-003), token seperti itu **tidak bisa di-settle** → ditolak + refund
([contracts/nft-collection.md](../contracts/nft-collection.md) §4 "Konsekuensi dust"). Praktis tak
terjangkau (harga NFT tidak pernah se-fraksi itu). Tidak ada jalur di mana market menahan dust ≤1 yocto:
seluruh `max_pay − Σpay` masuk ke seller.

## Algoritma Merge Multi-Receiver (Bundle)

> Bundle = **satu harga**; royalti dihitung **per token** lalu **di-merge per receiver** sebelum payout (INV-003/027). Receiver unik hasil merge **> 10 → tolak saat `create_bundle`** (INV-021). Fee 2% dihitung dari harga bundle, **dipotong sekali**.

```text
function aggregate_bundle(bundle_price, items[]):
  merged = {}                                  # receiver_id → amount
  for item in items:                           # 1..=10 token
    p = nft_transfer_payout(item, ...)         # payout per token (royalti; UNTRUSTED)
    for (receiver, amount) in p:
      merged[receiver] += amount               # DIJUMLAHKAN per receiver (tanpa cap agregat, INV-027)
  assert len(merged) <= 10                     # ditolak saat create_bundle bila lebih (INV-021)
  fee     = floor(bundle_price * fee_bps / 10_000)   # sekali untuk seluruh bundle
  max_pay = bundle_price - fee
  assert sum(merged) <= max_pay
  seller  = max_pay - sum(merged)              # residual → seller
  return merged + [seller, fee]
```

### Contoh terhitung — bundle 2 token, harga 10 Ⓝ

**Basis royalti per token — DECIDED (ronde 16, bisa di-override user):**

```text
basis(token) =
  1. harga listing AKTIF token itu di market, bila ada;
  2. bila tidak ada → harga mint fase tempat token di-mint;
  3. bila tidak ada keduanya → sisa harga bundle dibagi rata ke token
     yang belum punya basis (proporsional, dibulatkan floor per token).
royalti_token = rate_royalti_token × basis(token)      # floor ke yocto
```

- Validasi `Σroyalti + fee ≤ harga_bundle` dilakukan saat **`create_bundle`** (bagian INV-025
  pre-validasi) — bila terlampaui → `create_bundle` ditolak (`CONFLICT_BUNDLE_TOO_MANY` bukan;
  gunakan pesan payout-invalid), bukan saat settlement.
- Derivasi struct royalti → payout map (receiver, amount): [../contracts/nft-collection.md](../contracts/nft-collection.md) §Royalti.
- Yang tetap dikunci (tidak berubah): aturan **merge per receiver** di bawah.

Contoh di bawah memakai basis proporsional 6 Ⓝ + 4 Ⓝ (konsisten dengan aturan di atas —
bila kedua token ter-list 6 Ⓝ dan 4 Ⓝ, basis = harga listing masing-masing):

| Sumber | Receiver | Nilai (yocto) |
|---|---|---|
| Royalti token A (5% × 6 Ⓝ) | `creatorX.testnet` | `300000000000000000000000` |
| Royalti token B (2.5% × 4 Ⓝ) | `creatorX.testnet` | `100000000000000000000000` |
| Royalti token B (1% × 4 Ⓝ) | `creatorY.testnet` | `40000000000000000000000` |
| **Merge** `creatorX` | `creatorX.testnet` | `400000000000000000000000` |
| **Merge** `creatorY` | `creatorY.testnet` | `40000000000000000000000` |

| Komponen | Nilai (yocto) |
|---|---|
| Harga bundle | `10000000000000000000000000` |
| Fee 2% (sekali) | `200000000000000000000000` |
| Σroyalti (setelah merge) | `440000000000000000000000` |
| Seller (residual) | `9360000000000000000000000` |
| Receiver unik | 2 (≤10 ✓) |

- Verifikasi: `0.2 + 0.44 + 9.36 = 10 Ⓝ` ✓. Bila receiver unik > 10 → `create_bundle` **ditolak** (bukan dipotong).
- Kegagalan residual mid-loop (sangat jarang) → status `PARTIAL` + event `market_bundle_partial` + refund dana belum terpakai + kompensasi G7 (insurance fund 0.1% dari fee) — [order-protocol-security.md](../security/order-protocol-security.md) §6.

## Anggaran Gas per Jalur (Breakdown)

> Batas keras **300 Tgas/call** (FACT NEAR). Nilai **FACT/riset** dari [system-architecture.md](../architecture/system-architecture.md) § Anggaran gas; sisanya PROPOSED. Total dipantau; naikkan bila jalur payout berubah.

| Jalur | Langkah | Gas |
|---|---|---|
| **Buy (listing)** | `buy` (writes + dual verify) → `nft_transfer_payout` (1 yocto, NEP-199) → `resolve_purchase` (`#[private]`) | < 150 Tgas total; `nft_transfer_payout` **15 Tgas** (FACT), `resolve_purchase` **115 Tgas** (FACT) |
| **Accept offer** | `accept_offer` → `nft_transfer_payout` → `resolve_purchase` | Sama seperti buy (~15 + 115) |
| **Buy bundle (10 token)** | pre-validasi semua item → loop `nft_transfer_payout` ×10 → agregasi → `resolve_purchase` | PROPOSED — **wajib diukur** di sandbox (INV-021); jalur paling rawan gas |
| **Cancel/refund** | `cancel_offer` / `cancel_bundle` / refund lazy → transfer refund | ~5–10 (PROPOSED); transfer refund gas dijalankan kontrak |
| **Listing** | `nft_approve` (Tx-1) → `list_nft_for_sale` + 2 view call + `process_listing` | ~10–15 + ~5 + 2×(3–5) (PROPOSED) |

- **Gas kurang di resolve** → gas budget 115 Tgas dipantau; naikkan bila payout path berubah (lihat tabel Error Cases).
- `buy_bundle` dengan 10 item adalah risiko utama → batas statis 10 (INV-021) adalah kontrolnya.

## Refund Path (Detail)

| Jalur | Pemicu | Tujuan refund | Catatan |
|---|---|---|---|
| Buy gagal | payout invalid / stale / revert | `buyer` (pemanggil) | Via `resolve_purchase`, atomik; listing tetap ada bila kegagalan bukan stale (SEC-ORDER-001) |
| Deposit > harga | harga berubah saat signing | `buyer` | Selisih saja; bukan refund penuh |
| Cancel offer | buyer `cancel_offer` | `offer.buyer_id` (**hardcoded**, INV-005) | Refund penuh escrow |
| Offer expire | lazy saat disentuh | `offer.buyer_id` | Event `market_offer_expire` saat refund dieksekusi |
| Offer superseded | offer lain di-accept | `offer.buyer_id` | Auto-cancel + refund penuh |
| Bundle pre-validate gagal | validasi sebelum transfer pertama | `buyer` | Abort bersih, **nol transfer**, refund penuh (INV-025) |
| Bundle partial | residual mid-loop | `buyer` | Refund dana belum terpakai + kompensasi G7 |
| FT gagal (fase 2) | `ft_on_transfer` tolak | via `ft_resolve_transfer` | Refund otomatis standar NEP-141 |

- **Refund selalu ke penerima hardcoded dari state** (`buyer_id`) — tidak ada tujuan arbitrer (INV-014). Refund **tidak** terhalang `paused` (INV-022).
- **Storage deposit tidak otomatis kembali** saat refund — pemilik menarik sendiri via `storage_withdraw` (mencegah gas-DoS refund massal).
- **Dust ≤1 yocto** tidak direfund terpisah (bukan dana buyer yang tertahan; bagian toleransi pembulatan).

## Presisi & Aturan Tampilan Nilai

| Aturan | Nilai |
|---|---|
| Representasi on-chain & API | **string yoctoNEAR** (u128) — dilarang JSON `number` (AGENTS.md, api-overview.md) |
| Representasi DB | `NUMERIC(39,0)` untuk kolom `*_yocto` — bukan `float`/`double`/`bigint` |
| Konversi tampilan | `NEAR.fromDecimal()` / `formatUnits` (lib/format) — **dilarang** aritmetika float pada yoctoNEAR |
| Desimal tampilan | sampai 24 desimal, trailing zero dipangkas; nilai < 0.01 Ⓝ ditampilkan `< 0.01 Ⓝ` |
| Pembulatan tampilan | hanya di layer tampilan — **tidak pernah** memengaruhi nilai yang di-sign/dibayar |
| Timestamp | ISO-8601 UTC di API/DB; `u64` Unix **nanodetik** on-chain (selaras `env::block_timestamp()` + INV-010 — keputusan A2 ronde 16) |
| Satuan minimum | 1 yocto (`1e-24 Ⓝ`); min harga listing/offer 0.01 Ⓝ (INV-030) |

## Akuntansi & Rekonsiliasi

> Chain = otoritas; DB/API hanya proyeksi (SEC-INDEX-001). Tidak ada backend payment (ADR-010).

- **Invarian yang direkonsiliasi tiap settlement**: `fee + Σroyalti + seller == price` (internal, exact) dan `Σpayout ≤ price − fee` (INV-001/002).
- **Sumber rekonsiliasi**: event `market_sale` / `market_offer_accept` + `(receipt_id, event_index)` — bukan `tx_hash` (satu tx bisa banyak event). Skema event dimiliki [webhooks.md](../api/webhooks.md).
- **Treasury**: fee ditransfer **langsung ke `treasury` di dalam settlement yang sama** (dikonfirmasi ronde 23 saat TASK-005 — model akumulasi + `withdraw_fees` dibatalkan; lihat [contracts/market.md](../contracts/market.md) §4a). Market contract **tidak pernah** memegang dana fee, sehingga tidak ada akuntansi solvensi fee-vs-escrow-vs-storage yang perlu dijaga. Alamat treasury diisi saat deploy (§MVP); default = owner market.
- **Deteksi anomali**: reconcile `sum(payout) == harga − fee` tiap settle; **alert bila selisih > 1 yocto** ([catastrophic-failure-scenarios.md](../security/catastrophic-failure-scenarios.md)).
- **Fase 2 (indexer)**: proyeksi `sales.payout` (jsonb) disalin dari event; query per-penerima dinormalisasi = ⏳ open-by-design.

## FT Storage Deposit (Fase 2 — NEP-141)

> Belum ada di MVP (NEAR saja). Spesifikasi target agar desain tidak tambal-sulam.

- Market contract harus **terdaftar** di setiap FT sebelum bisa menerima `ft_transfer_call`: panggil `storage_deposit(market, registration_only=true)` di kontrak FT → market punya saldo storage FT.
- **Jumlah deposit** per FT ⏳ open-by-design (bergantung storage FT; dibayar saat pendaftaran/whitelist FT).
- `ft_on_transfer` validasi `predecessor == FT contract` **dan** FT ada di whitelist; jika tidak → tolak (refund otomatis via `ft_resolve_transfer`).
- Payout FT + royalti banyak akun harus dipangkas — batas **10 penerima** tetap berlaku (batas gas sehat, RESEARCH.md §11).
- Sisa pembulatan FT: standar NEP-141 mengembalikan `amount` sisa ke pengirim via `ft_resolve_transfer` (bukan seperti NEAR yang residual-nya masuk seller) — detail ⏳ saat implementasi fase 2.

## Error Cases

| Kasus | Perlakuan |
|---|---|
| Payout > harga−fee / kosong / > 10 penerima / amount 0 | tolak → refund buyer (resolve_purchase, INV-001..003) |
| Royalti **per token** > 10% harga wajar token | tolak → refund (INV-027; agregat bundle = penjumlahan, tanpa cap) |
| Deposit > harga (harga berubah saat signing) | selisih dikembalikan ke buyer via resolve |
| FT belum terdaftar (fase 2) | `ft_on_transfer` tolak → refund otomatis via ft_resolve_transfer |
| Gas kurang di resolve | gas budget 115 Tgas dipantau; naikkan bila payout path berubah |

> **Bukan error case: residual seller besar.** `max_pay − Σpayout` = proceeds seller (§Algoritma
> Distribusi Payout), nilainya wajar besar. Tidak ada lagi aturan "sisa > 1 yocto → tolak"
> (koreksi ronde 23 — aturan itu berasal dari model tutorial yang tidak dipakai NearSea).

## Acceptance Criteria

- Given penjualan via buy now atau accept offer, When settlement sukses, Then distribusi = harga − 2% fee → seller + royalti, fee → treasury.
- Given settlement gagal, When resolve, Then buyer menerima refund penuh dan listing tetap **ada** (tetap aktif bila kegagalan bukan karena stale).
