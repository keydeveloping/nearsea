# Order Protocol Security

> Protokol order NearSea. **DESIGN (ADR-003/ADR-012)**: order = state ON-CHAIN (orderbook on-chain), bukan pesan signed off-chain ala Seaport. Dokumen ini mendefinisikan lifecycle, identity, dan pembatasan.

## 1. Dua jenis order

| | Listing (jual) | Offer (beli) | Bundle |
|---|---|---|---|
| Identity key | `nft_contract_id + "." + token_id` | `nft_contract_id + "." + token_id` + `buyer_id` | bundle_id (counter kontrak) |
| Signer awal | Seller (2 tx: approve + list — ADR-002) | Buyer (`make_offer`, attach Ⓝ escrow) | Seller (approve semua token + create) |
| State on-chain | `Sale` | `Offer` + escrow Ⓝ | `BundleSale` + approvals per token |
| Dihapus saat | terjual/cancel | accept/expire/cancel | terjual/cancel |

**Constraint terkunci**: maks 1 offer aktif per buyer per token; harga min 0.01 Ⓝ (listing & offer); durasi offer default 7 hari bila tidak dispesifikasi.

## 2. Lifecycle

```text
LISTING:  ACTIVE ──► SOLD (settlement sukses)
             ├──► CANCELLED (seller, assert_one_yocto; market hapus entry — tanpa revoke approval)
             └──► STALE (ownership mismatch terdeteksi → disembunyikan; remove_stale_listing)

OFFER:    ACTIVE ──► ACCEPTED (seller → nft_transfer_payout; escrow terdistribusi)
             ├──► CANCELLED (buyer, refund penuh)
             ├──► EXPIRED (expires_at lewat → refund oleh siapa pun/saat disentuh)
             └──► SUPERSEDED (offer lain pada token yang sama di-accept → auto-cancel + refund)

BUNDLE:   ACTIVE ──► SOLD (semua token sukses ditransfer)
             ├──► CANCELLED (seller; cukup hapus membership — approval tidak dicabut market)
             ├──► PRE_VALIDATE_FAILED (pre-validasi gagal sebelum transfer pertama → abort bersih + refund penuh)
             └──► PARTIAL (residual mid-loop failure — sangat jarang: concurrent transfer/gas;
                  NEAR tidak punya rollback lintas-receipt (FACT) → status partial tercatat on-chain
                  + event market_bundle_partial + refund dana belum terpakai + kompensasi via G7)
```

## 3. Field order & sumber kebenaran

| Field | Sumber | Bisa diubah? |
|---|---|---|
| owner (seller) | `nft_token().owner_id` saat verifikasi | Tidak (di-sync via dual verification) |
| approval_id | hasil `nft_approve` | Tidak; invalid setelah transfer (FACT NEP-178) |
| sale_conditions (harga) | argumen list/update_price | Ya, owner-only + 1 yocto |
| escrow amount | attached deposit `make_offer` | Tidak (immutable per offer) |
| expires_at | now + durasi saat make (default 7 hari) | Tidak |
| allowed_buyer (private) | argumen list | Tidak (batal + list ulang) |

## 4. OFF-CHAIN vs ON-CHAIN state (pemisahan wajib)

```text
ON-CHAIN SETTLEMENT STATE (otoritatif):
  sales map, offers map + escrow balances, bundles, approvals, storage_deposits

OFF-CHAIN ORDER STATE (proyeksi, boleh basi):
  API cache listing/offer/bundle, riwayat dari NearBlocks (explorer/riwayat) — sumber proyeksi
  order saat fase 2 = Neardata (ADR-015); keduanya bukan otoritas

ATURAN: transaksi uang TIDAK PERNAH boleh dipicu dari state off-chain.
Sebelum sign, FE wajib view-call on-chain: harga, ownership, expiry, stale flag. (SEC-ORDER-003)
```

## 5. Aturan keamanan per transisi

- **ACTIVE→SOLD (listing)**: deposit = harga (kelebihan deposit → selisih dikembalikan buyer via resolve), buyer ≠ seller (INV-023), token masih milik seller & approval valid (dual verification), payout tervalidasi: **royalti ≤ 10% harga**, ≤10 penerima, sum ≤ harga−fee, sisa ≤1 yocto, fee 2% dipotong dulu.
- **ACTIVE→ACCEPTED (offer)**: hanya oleh owner token saat itu & sebelum expiry; **1 offer aktif/buyer/token** & min 0.01 Ⓝ dicek saat make; escrow exact = attached deposit.
- **ACTIVE→CANCELLED**: seller/listing — market **hanya menghapus entry `Sale`** (tidak mencabut approval: NEP-178 tidak punya revoke untuk approved account — [contracts/market.md](../contracts/market.md) §2a); buyer/offer — refund ke `offer.buyer_id` hardcoded; storage deposit offer TIDAK otomatis kembali (ditarik via `storage_withdraw` oleh pemiliknya — mencegah gas-DoS refund massal).
- **EXPIRED**: lazy evaluation — dicek saat sentuh (accept/cancel/withdraw); tidak butuh cron (FACT: tidak ada cron di MVP).
- **STALE**: deteksi = `nft_token(token_id).owner_id != sale.owner_id` saat view/dibeli; setelah stale → tidak bisa dibeli.
- **STALE dalam bundle**: jika salah satu token bundle dipindah/di-list/di-offer terpisah → seluruh bundle tak bisa dibeli (cek tiap item saat offer) + seller ditandai untuk batal manual; token dalam bundle AKTIF tidak boleh di-list/di-offer terpisah (dicek saat list/offer via cek bundle membership).
- **Partial fill**: TIDAK ada di MVP (NFT unit = 1). Overfill tidak mungkin: unique map key (SEC-ORDER-005).

## 6. Kondisi balik (revert) yang dijamin

Pola tutorial (RESEARCH.md §10): sale dihapus sebelum settlement (optimistic); jika promise gagal/payout invalid → `resolve_purchase` mengembalikan state sale + refund buyer atomik dalam callback. Untuk **bundle**: tidak ada rollback atomik lintas-receipt di NEAR (FACT) → keandalan dicapai lewat **pre-validation sebelum transfer pertama** (ownership + approval + simulasi payout semua item; INV-025) dan **agregasi payout**: royalti per token dijumlahkan lalu **di-merge per receiver**; bila receiver unik hasil merge > 10 → bundle DITOLAK saat `create_bundle` (pre-validasi). Fee 2% dihitung dari harga bundle, dipotong sekali. Kegagalan residual mid-loop → status `PARTIAL` + kompensasi G7 (di atas). Invariant: **buyer tidak pernah kehilangan dana tanpa aset pindah atau kompensasi tercatat**. (SEC-ORDER-001/002)

## 7. Launchpad (mint, bukan order — tapi menyentuh state yang sama)

> Ditambahkan pasca-audit: launchpad tidak membuat "order" tapi menulis state token yang nanti di-list, jadi aturannya harus konsisten.

- **Phase berurutan, tidak overlap** (INV-029): maksimum satu fase aktif; dicek saat `nft_mint`.
- **Deposit exact-match** (INV-018); **storage dibayar pemicu mint** (INV-019); allowlist = set on-chain (creator pre-deposit).
- **Mint ≠ order**: token hasil mint masuk wallet minter langsung; untuk menjual, minter memakai jalur listing biasa (2-tx) — tidak ada auto-list dari mint.
- **Interaksi dengan bundle/listing**: token yang di-mint dari koleksi yang sama tetap tunduk pada INV-028 (tidak bisa masuk bundle aktif + di-list terpisah).

## 8. Race & konkurensi (banyak pembeli, satu NFT)

> Detail lengkap + jawaban "20 pembeli 1 NFT": [../development/concurrency-and-races.md](../development/concurrency-and-races.md).

- **Penentu pemenang = urutan eksekusi on-chain, bukan RPC.** NEAR tidak punya priority fee; sharding berbasis akun → semua listing di satu akun kontrak market, sehingga `buy` diproses **berurutan** di satu shard.
- **Optimistic removal** (§6): `buy` pertama menghapus entry sale; `buy` berikutnya revert → **deposit Ⓝ kembali otomatis** (hanya gas hangus).
- **Unique key** (INV-007 / SEC-ORDER-005) → tidak mungkin dua listing aktif / overfill.
- **Pemenang gagal promise** → `resolve_purchase` memulihkan sale + refund → listing aktif lagi.
- **Double-submit** (pembeli sama 2×) → nonce per access key + entry sale sudah terhapus → tx kedua revert.
- Invariant: **tidak pernah ada dua pembeli membayar NFT yang sama**. (SEC-ORDER-006)

## 9. Contoh numerik terhitung — payout & merge bundle (yocto)

> Contoh kerja untuk **bundle** (satu harga, royalti per token, merge per receiver, fee sekali). Semua nilai = **string yoctoNEAR**; `1 Ⓝ = 1000000000000000000000000`. Aturan merge & fee dipotong sekali **dikunci**; **basis royalti per token = DECIDED (ronde 16)**: harga listing aktif token > harga mint fase-nya > alokasi proporsional sisa (aturan lengkap: [payments.md](../features/payments.md) § Contoh terhitung). Contoh di bawah memakai alokasi proporsional menurut nilai wajar token — konsisten dengan aturan tersebut.

**Setup**: bundle 3 token, satu harga `5 Ⓝ`; nilai wajar token 2 Ⓝ + 2 Ⓝ + 1 Ⓝ; fee 2% (`fee_bps=200`).

**Langkah 1 — royalti per token** (dibaca dari `nft_transfer_payout` tiap koleksi; UNTRUSTED, divalidasi ≤10% per token — INV-027):

| Token | Basis (yocto) | Rate royalti | Penerima (receiver) | Royalti (yocto) |
|---|---|---|---|---|
| A | `2000000000000000000000000` | 5% (500 bps) | `creatorX.testnet` | `100000000000000000000000` |
| B | `2000000000000000000000000` | 2.5% (250 bps) | `creatorX.testnet` | `50000000000000000000000` |
| C | `1000000000000000000000000` | 10% (1000 bps) | `creatorY.testnet` | `100000000000000000000000` |
| | | | **Σroyalti (pra-merge)** | `250000000000000000000000` |

**Langkah 2 — merge per receiver** (duplikat receiver DIJUMLAHKAN; receiver unik hasil merge > 10 → bundle ditolak saat `create_bundle` — INV-003/021):

| Receiver | Asal | Jumlah (yocto) |
|---|---|---|
| `creatorX.testnet` | A + B | `150000000000000000000000` |
| `creatorY.testnet` | C | `100000000000000000000000` |
| **Receiver unik** | | **2 (≤10 ✓)** |

**Langkah 3 — distribusi final** (fee dipotong **sekali** dari harga bundle; sisa pembulatan jatuh ke seller):

| Komponen | Perhitungan | Nilai (yocto) |
|---|---|---|
| Harga bundle | — | `5000000000000000000000000` |
| Fee platform 2% | `floor(5 Ⓝ × 200 / 10000)` | `100000000000000000000000` |
| Plafon payout | `harga − fee` | `4900000000000000000000000` |
| Royalti `creatorX` (merged) | A + B | `150000000000000000000000` |
| Royalti `creatorY` (merged) | C | `100000000000000000000000` |
| Seller (residual) | `plafon − Σroyalti` | `4650000000000000000000000` |
| **Sisa pembulatan** | `plafon − Σpayout` | **`0`** (≤1 yocto ✓) |

- Verifikasi exact: `fee + Σroyalti + seller = 0.1 + 0.25 + 4.65 = 5 Ⓝ` ✓ (INV-001/002).
- **Varian dust (sisa 1 yocto)**: bila payout eksternal NFT contract menghasilkan `Σroyalti = plafon − 1` (`4899999999999999999999999`), sisa `1` yocto **diterima** (batas inklusif INV-002); 1 yocto tersebut tetap di kontrak (tidak terdistribusi) dan **tidak** direfund terpisah. Bila `sisa ≥ 2` yocto → tolak + refund penuh (SEC-ORDER-001).

## 10. Tabel per-aktor (siapa boleh apa)

| Aktor | Aksi sah | Otoritas (auth) | Batasan | Bukti on-chain |
|---|---|---|---|---|
| **Buyer** | `buy`, `buy_bundle` (deposit = harga), `make_offer` (escrow exact), `cancel_offer` | `predecessor_account_id` = pemanggil; deposit ≥ harga / = offer | ≠ seller (INV-023); private → `allowed_buyer` (INV-026); 1 offer aktif/buyer/token (INV-024); min 0.01 Ⓝ (INV-030) | `market_sale` / `market_offer` / `market_offer_cancel` |
| **Seller** | `list_nft_for_sale`, `update_price`, `remove_sale`, `accept_offer`, `create_bundle`, `cancel_bundle` | `predecessor` = `sale.owner_id` / owner token saat itu; `assert_one_yocto()` (kecuali list/create = storage) | harga ≥ min (INV-030); bundle ≤10 token & receiver unik ≤10 (INV-021); phase launchpad berurutan (INV-029) | `market_list` / `market_update_price` / `market_delist` / `market_offer_accept` |
| **Market contract** | settle (`nft_transfer_payout`), refund, distribusi fee/royalti, tandai stale | **internal** — bukan aktor eksternal; `#[private]` callback (INV-013) | tujuan transfer hanya turunan state tervalidasi (INV-014); dual verification sebelum transfer (INV-016) | event `market_*` (emitter = market) |
| **NFT contract** (koleksi) | `nft_transfer_payout` (paksa royalti NEP-199), `nft_token`, `nft_is_approved`, `nft_approve` | dipanggil market sebagai approved account (NEP-178); **`nft_revoke` owner-only** → market tidak mencabut approval | payout UNTRUSTED → market validasi (≤10, sum, remainder — INV-002/003); approval invalid setelah transfer (INV-011) | `nft_transfer` / `nft_mint` (NEP-171/297) |
| **Siapa pun** | `remove_stale_listing` (permissionless), refund offer expired (lazy) | bebas | hanya bila mismatch/stale **terbukti on-chain**; refund selalu ke `buyer_id` (INV-005) | `market_stale_detected` / `market_offer_expire` |

## 11. Anggaran gas (Tgas) per jalur order

> Batas keras **300 Tgas/tx** (FACT NEAR). Nilai **FACT** = terukur/dari riset; sisanya **PROPOSED** (diukur di sandbox). Sumber lintas-dokumen: [system-architecture.md](../architecture/system-architecture.md) § Anggaran gas, [payments.md](../features/payments.md) § Anggaran Gas.

| Jalur / method | Komponen | Gas | Jenis |
|---|---|---|---|
| `buy` (listing) | writes + dual verify → `nft_transfer_payout` (15) → `resolve_purchase` (115) | **< 150 total** | PROPOSED (komponen FACT) |
| `accept_offer` | writes + `nft_transfer_payout` (15) → `resolve_purchase` (115) | **< 150 total** | PROPOSED (komponen FACT) |
| `buy_bundle` (≤10 token) | pre-validasi semua item → `nft_transfer_payout` ×10 (15 each = 150) → agregasi → `resolve_purchase` (115) | **wajib diukur** (risiko utama) | PROPOSED |
| `list_nft_for_sale` | writes + 2 view call (`nft_token`, `nft_is_approved`) + `process_listing` | ~10–20 | PROPOSED |
| `make_offer` | tulis state escrow | ~5 | PROPOSED |
| `cancel_offer` / `cancel_bundle` | tulis state + transfer refund | ~5–10 | PROPOSED |
| `remove_sale` / `update_price` / `remove_stale_listing` | tulis state (tanpa XCC — approval tidak dicabut market) | ~5 | PROPOSED |
| `nft_approve` (NFT contract) | Tx-1 listing; + storage approval | ~10–15 | PROPOSED |

- `buy_bundle` 10 token = satu-satunya jalur yang mendekati 300 Tgas → **batas statis 10 token** (INV-021) adalah kontrolnya (SEC-CONTRACT-010).
- Resolve budget **115 Tgas** dipantau; bila jalur payout berubah → ukur ulang (lihat tabel Error Cases payments.md).

## 12. Attack tree — manipulasi order

```text
MANIPULASI ORDER (tujuan penyerang: bayar < nilai / ambil aset tanpa bayar / kuras dana)
├── A. Manipulasi harga & payout
│   ├── A1 payout > harga−fee ......................... INV-002 → tolak + refund (SEC-ORDER-001)
│   ├── A2 royalti per token > 10% .................... INV-027 → tolak + refund (TC-015)
│   ├── A3 receiver duplikat / > 10 unik .............. INV-003/021 → merge; > 10 ditolak saat create_bundle
│   ├── A4 fee dinaikkan > cap ........................ INV-004/MAX_FEE_BPS=500 → revert
│   └── A5 drain pembulatan (dust) ................... INV-002 sisa ≤1 yocto; dust tetap di kontrak
├── B. Manipulasi identitas & approval
│   ├── B1 self-buy (beli milik sendiri) .............. INV-023 → revert
│   ├── B2 bypass private listing ..................... INV-026 → hanya allowed_buyer (TC-011)
│   ├── B3 replay approval_id lama .................... INV-011/016 → cek ulang tiap settle
│   ├── B4 forge nft_on_approve (kontrak jahat) ....... INV-013 → payload NEP-178 + predecessor = NFT sah
│   └── B5 beli listing stale ......................... INV-016 → dual verification (TC-006)
├── C. Manipulasi state & race
│   ├── C1 double-buy listing sama .................... INV-007/008 → optimistic removal + unique key (TC-016/017)
│   ├── C2 double-accept offer sama ................... INV-009 → entry dihapus atomik (TC-018)
│   ├── C3 offer ganda buyer/token .................... INV-024 → unique state (TC-008)
│   └── C4 token bundle di-list/di-offer terpisah ..... INV-028 → seluruh bundle tak bisa dibeli (TC-009/010)
└── D. Manipulasi escrow & refund
    ├── D1 refund ke alamat arbitrer .................. INV-005/014 → tujuan = buyer_id hardcoded
    ├── D2 kuras escrow via cancel .................... INV-005 → hanya ke offer.buyer_id
    └── D3 storage-DoS refund massal .................. storage tidak auto-refund; tarik via storage_withdraw
```

- Setiap daun tree = satu kontrol + invariant + test; tidak ada daun tanpa kontrol (SEC-ORDER-001..006).
- Aset tidak pernah dipegang kontrak (NFT non-custodial, ADR-007) → permukaan D hanya menyangkut escrow offer + akumulasi fee.

## 13. Peta transisi → test case (TC)

| Transisi / skenario | Event | Invariant | Test |
|---|---|---|---|
| ACTIVE→SOLD (listing) | `market_sale` | INV-001/002/011 | TC-002, TC-022, TC-040 |
| ACTIVE→CANCELLED (listing, seller) | `market_delist` | INV-012/022 | unit (belum ada TC khusus) |
| ACTIVE→STALE (ownership mismatch) | `market_stale_detected` | INV-016 | TC-006 |
| OFFER ACTIVE→ACCEPTED | `market_offer_accept` | INV-005/010 | TC-005, TC-018, TC-041 |
| OFFER ACTIVE→CANCELLED (buyer) | `market_offer_cancel` | INV-005 | unit (jalur refund) |
| OFFER ACTIVE→EXPIRED (lazy) | `market_offer_expire` | INV-005 | TC-004 |
| OFFER ACTIVE→SUPERSEDED | `market_offer_cancel` (refund) | INV-005/009 | TC-019 |
| BUNDLE ACTIVE→SOLD | `market_sale` | INV-025/027 | TC-009 |
| BUNDLE ACTIVE→CANCELLED | (event bundle ⏳ open-by-design) | INV-022/028 | unit |
| BUNDLE ACTIVE→PRE_VALIDATE_FAILED | (abort bersih) | INV-025 | TC-010 |
| BUNDLE ACTIVE→PARTIAL (residual) | `market_bundle_partial` | INV-025 | PROPOSED — butuh fault injection |
| Private listing (bypass ditolak) | `market_list` | INV-026 | TC-011 |
| Min harga listing/offer | — | INV-030 | TC-013 |
| Bundle > 10 token | — | INV-021 | TC-014 |
| Royalti per token > 10% / agregat sah | — | INV-027 | TC-015 |
| Race banyak pembeli / double-submit | `market_sale` | INV-007/008/009 | TC-016, TC-017, TC-042 |
| Pause blokir mutasi, izinkan refund | — | INV-022 | TC-012 |
| Launchpad phase & allowlist | `launchpad_mint` | INV-017/029 | TC-007, TC-021 |

> Katalog TC lengkap dimiliki [test-cases.md](../testing/test-cases.md); tabel ini memetakan tiap transisi lifecycle §2 ke bukti ujinya. Baris "unit" = belum ada TC bernomor — dicatat sebagai pekerjaan test saat implementasi.
