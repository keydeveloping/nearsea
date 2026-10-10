# Smart Contract Invariants

> Invariant formal SEBELUM kode ada. Setiap invariant nanti menjadi: unit test, property/fuzz test (cargo-fuzz / proptest), dan target review audit. ID **INV-001..030** stabil.

## Dana & payout

- **INV-001**: Total Ⓝ keluar dari settlement ≤ Ⓝ masuk pada transaksi tersebut (fee + royalti + proceeds ≤ attached deposit).
- **INV-002**: `Σpayout ≤ harga − fee` (tidak ada pembayaran melebihi plafon); pelanggaran → tolak + refund buyer. **Koreksi ronde 23:** klausa lama "sisa pembulatan ∈ {0,1} yocto" **dibatalkan** — ia berasal dari model tutorial (RESEARCH.md §10.5) di mana `nft_transfer_payout` mengembalikan **seluruh** distribusi (seller + royalti), sehingga `max_pay − Σpayout` hanya sisa pembulatan. NearSea memakai model berbeda: koleksi mengembalikan **hanya royalti** ([contracts/nft-collection.md](../contracts/nft-collection.md) §4) dan market menambahkan seller sebagai residual ([contracts/market.md](../contracts/market.md) §3), jadi `max_pay − Σpayout` = **proceeds seller** (wajar besar, mis. 93% harga saat royalti 5%). Meng-`assert sisa ≤ 1` akan menolak setiap penjualan normal dan bertentangan dengan TC-002 + AC TASK-005. Aritmetika internal tetap exact: `fee + Σroyalti + seller == harga`.
- **INV-003**: Payout memiliki 1..10 penerima; tidak ada duplikat receiver (**DEFAULT: digabung**).
- **INV-004**: `fee_bps` hanya boleh ≤ `MAX_FEE_BPS`. **CATATAN NEAR**: `const` di kode TIDAK kebal upgrade — perubahan `MAX_FEE_BPS` hanya mungkin via upgrade yang lolos governance (mainnet: Sputnik DAO 2-of-3 + timelock, ADR-013) dan terverifikasi reproducible build (NEP-330). Property test: pada kode yang deployed, attempt `update_fee_bps > 500` selalu revert.
- **INV-005**: Escrow offer hanya bisa keluar sebagai: (a) refund ke `offer.buyer_id`, atau (b) distribusi saat `accept_offer` sukses. Tidak ada jalur ketiga.
- **INV-006**: `attached_deposit` pada `make_offer` = escrow yang tercatat (exact match, bukan ≥) agar tidak ada Ⓝ tanpa state.

## Order lifecycle

- **INV-007**: Key listing unik — satu token tidak bisa punya dua listing aktif (map key).
- **INV-008**: Listing/offer CANCELLED/EXPIRED/STALE tidak bisa dibeli / di-accept.
- **INV-009**: Offer sudah di-accept tidak bisa di-accept lagi (entry dihapus atomik sebelum settle; revert mengembalikan entry).
- **INV-010**: Accept offer hanya oleh owner token **saat itu** dan `env::block_timestamp() < expires_at`.
- **INV-011**: Setelah `nft_transfer_payout` sukses, approval lama invalid (FACT NEP-178); market tidak boleh beroperasi dengan approval_id lama (cek ulang setiap settle).

## Otorisasi

- **INV-012**: Tidak ada mutasi seller/owner tanpa `predecessor == owner` yang relevan + (`assert_one_yocto` bila berlaku).
- **INV-013**: Callback internal (`process_listing`, `resolve_purchase`, `nft_resolve_transfer`) hanya callable oleh kontrak sendiri (`#[private]`). Endpoint masuk `nft_on_approve` BUKAN `#[private]` — memvalidasi payload NEP-178 + predecessor = NFT contract yang sah.
- **INV-014**: Tidak ada fungsi publik yang memindahkan Ⓝ ke alamat arbitrer (tujuan transfer selalu turunan state yang tervalidasi: buyer/seller/payout/treasury).

## Non-custodial NFT

- **INV-015**: Market contract tidak pernah memanggil `nft_transfer` tanpa approval_id yang valid saat itu (jalur transfer hanya lewat settlement yang sah).
- **INV-016**: Settlement berhenti bila listing **stale**. Stale mencakup **DUA kasus** (perbaikan H4 ronde 17b): **(A)** ownership mismatch — `nft_token(token_id).owner_id != sale.owner_id`; **(B)** approval tidak valid — `!nft_is_approved(sale.owner_id, market, approval_id)`. Keduanya menolak `buy` dengan `CONFLICT_STALE` dan keduanya dapat dibersihkan oleh `remove_stale_listing` (permissionless). Tidak pernah transfer NFT yang bukan milik seller listing.

## Jaminan dana keluar (recovery)

- **INV-031**: **Setiap deposit yang masuk ke `buy` selalu punya jalur keluar tanpa bergantung pada governance.** Saat `buy`, entri `pending_purchases[sale_key] = {buyer, deposit, created_height}` ditulis **sebelum** optimistic removal. Entri itu dihapus hanya oleh: (a) `resolve_purchase` sukses, (b) `resolve_purchase` gagal (restore + refund), atau (c) `recover_stuck_purchase` (permissionless, syarat tunggal `block_height > created_height + RECOVERY_DELAY_BLOCKS`). Tidak ada dana yang bisa tertahan permanen bila callback gagal. *(Menutup temuan C3.)*

## Launchpad

- **INV-017**: Mint ditolak jika: tidak ada fase aktif (waktu), alokasi fase habis, jumlah mint melebihi max/wallet, atau allowlist phase mensyaratkan dan minter tidak terdaftar.
- **INV-018**: `attached_deposit == phase.price` (exact-match; selain itu revert — deposit otomatis kembali via rollback tx).
- **INV-019**: Storage mint dibayar pemicu mint (invoker) — kontrak tidak menanggung.

## Ketersediaan & kontrol darurat

- **INV-020**: Tidak ada path yang membuat listing/offer/user menambah state tanpa membayar storage (NEP-145 di semua map yang tumbuh).
- **INV-021**: Semua loop terikat batas statis. Dua batas **berbeda** yang sering tertukar:
  - **Maks 10 item per bundle** (jumlah token dalam satu `buy_bundle`).
  - **Maks 10 penerima payout** (seller + royalti + treasury setelah di-merge per receiver) — batas protokol NEP-199/300 Tgas.
  Allowlist tidak diiterasi penuh di method payable.
- **INV-022**: Saat `paused`: SEMUA mutasi baru revert KECUALI cancel/withdraw/refund. Pause dipanggil owner (MVP) / guardian `pause_callers` (mainnet — terpisah dari owner-DAO).

## Business rules terkunci

- **INV-023**: Self-buy ditolak — pemanggil `buy` ≠ `sale.owner_id`; pemanggil `buy_bundle` ≠ owner SEMUA token bundle; `make_offer` ≠ owner token saat itu (offer sah pada token yang tidak di-list; cek ke `nft_token().owner_id`, bukan `sale.owner_id`).
- **INV-024**: Maks **1 offer aktif per buyer per token** (unique state); durasi default 7 hari bila tidak dispesifikasi. (Min nominal offer = INV-030.)
- **INV-025**: **Bundle all-or-nothing via pre-validation**: sebelum transfer PERTAMA, market memvalidasi SEMUA item (ownership + approval + simulasi payout ≤ harga) — kegagalan pre-validasi → abort bersih + refund penuh, nol transfer terjadi. **Kegagalan residual mid-loop** (concurrent transfer/gas — sangat jarang): NEAR TIDAK punya rollback atomik lintas-receipt (FACT) → status partial tercatat on-chain + event `market_bundle_partial`, dana belum terpakai di-refund, selisih diselesaikan via kompensasi (G7 insurance fund). Buyer tidak pernah kehilangan dana tanpa kompensasi tercatat.
- **INV-026**: Private listing — hanya `allowed_buyer` yang bisa membeli; buyer lain selalu ditolak.
- **INV-027**: Cap royalti **per token** 10% — tiap token punya rate royalti sendiri (maks 10% dari harga wajarnya); pada bundle, royalti per token DIJUMLAHKAN **tanpa cap agregat** (keputusan seller saat membundle; standar industri). Penerima unik hasil merge tetap ≤10 (INV-021).

## Konsistensi order & launchpad

- **INV-028**: Token yang tergabung dalam bundle AKTIF tidak boleh di-list/di-offer terpisah; jika salah satu token bundle dipindah/stale → seluruh bundle tidak bisa dibeli (dicek saat `buy_bundle`).
- **INV-029**: Launchpad phase **berurutan, tidak overlap** — maksimum satu fase aktif pada satu waktu (dicek saat `nft_mint`).
- **INV-030**: Min harga **listing & offer** 0.01 Ⓝ (ditolak bila kurang) — scope diselaraskan dengan [01-PRD.md](../01-PRD.md) §13.

## Peta invariant → predikat formal → test → SEC → prioritas

> Peta kerja untuk implementasi: setiap invariant punya **predikat formal** (notasi code-like, siap diterjemahkan ke assert/test), **test ID** yang membuktikannya, **SEC-ID** terkait, dan **prioritas**. Test ID `unit`/`fuzz`/`property` = dijalankan di level unit/property (lihat §Cara diverifikasi nanti); TC-xxx = [test-cases.md](../testing/test-cases.md). Prioritas mengikuti [security-requirements.md](./security-requirements.md) (P0 = wajib sebelum gate terkait).

| INV | Predikat formal (code-like) | Test ID | SEC-ID | Prioritas |
|---|---|---|---|---|
| INV-001 | `Σ(amount_out) ≤ attached_deposit` | TC-002, TC-022, TC-040 | SEC-ORDER-001, SEC-CONTRACT-005 | P0 |
| INV-002 | `Σ(payout) ≤ price − fee`; residual `price − fee − Σpayout` → seller (tanpa batas atas — koreksi ronde 23) | TC-003, TC-022 | SEC-CONTRACT-005 | P0 |
| INV-003 | `1 ≤ |unique(receivers)| ≤ 10 ∧ ∀p ∈ payout: p.amount > 0` (duplikat digabung) | TC-003, TC-009 | SEC-CONTRACT-005 | P0 |
| INV-004 | `fee_bps ≤ MAX_FEE_BPS (500)` | unit + fuzz | SEC-CONTRACT-004 | P0 |
| INV-005 | `escrow_out ∈ { refund(offer.buyer_id), distribute(accept_offer) }` | TC-004, TC-005, TC-018, TC-019 | SEC-ORDER-002 | P0 |
| INV-006 | `attached_deposit(make_offer) == offer.amount` | TC-004 | SEC-ORDER-002 | P0 |
| INV-007 | `¬∃ two active Sale with same (nft_contract_id, token_id)` | TC-016, TC-017, TC-042 | SEC-ORDER-005 | P0 |
| INV-008 | `sale.status == ACTIVE ⇒ buy/accept allowed; else revert` | TC-016, TC-042 | SEC-ORDER-005/006 | P0 |
| INV-009 | `count(accepted) ≤ 1 per offer` (entry dihapus atomik sebelum settle) | TC-018, TC-019 | SEC-ORDER-006 | P0 |
| INV-010 | `predecessor == nft_token(token).owner_id ∧ block_timestamp < expires_at` | TC-004 | SEC-ORDER-006 | P0 |
| INV-011 | `settle ⇒ nft_is_approved(token, market, approval_id) at that moment` | TC-002 | SEC-ORDER-004 | P0 |
| INV-012 | `mutasi_owner ⇒ predecessor == owner ∧ (assert_one_yocto where applies)` | TC-008, TC-012 | SEC-CONTRACT-001 | P0 |
| INV-013 | `callback ⇒ predecessor == current_account` ; `nft_on_approve` validates payload NEP-178 + NFT contract sah | unit + TC-048 | SEC-CONTRACT-003 | P0 |
| INV-014 | `∀transfer: target ∈ derived_state(buyer │ seller │ payout │ treasury)` | property + fuzz | SEC-ORDER-002 | P0 |
| INV-015 | `market ⇒ nft_transfer only with valid approval (settlement path)` | TC-002 | SEC-ORDER-004 | P0 |
| INV-016 | `stale ⇒ ¬settle`, dgn stale = `owner_id ≠ sale.owner_id` **∨** `¬nft_is_approved(owner, market, approval_id)` | TC-006, TC-010, TC-022, **TC-053** | SEC-ORDER-004 | P0 |
| INV-017 | `mint ⇒ ∃active_phase ∧ alloc_left ∧ count ≤ max_per_wallet ∧ allowlist_ok` | TC-007 | SEC-CONTRACT-009 | P0 |
| INV-018 | `attached_deposit(nft_mint) == phase.price` | TC-007 | SEC-CONTRACT-009 | P0 |
| INV-019 | `storage_mint paid by invoker` | TC-001, TC-007 | SEC-CONTRACT-011 | P0 |
| INV-020 | `∀growing_map insert ⇒ storage_deposit ≥ required` | TC-020 | SEC-CONTRACT-011 | P0 |
| INV-021 | `|bundle_items| ≤ 10 ∧ |merged_receivers| ≤ 10 ∧ ∀loop bounded` | TC-014, TC-009 | SEC-CONTRACT-010 | P1 |
| INV-022 | `paused ⇒ all new mutations revert except {cancel, withdraw, refund}` | TC-012 | SEC-CONTRACT-007 | P0 |
| INV-023 | `buy: caller ≠ sale.owner_id; buy_bundle: caller ≠ owner all; make_offer: caller ≠ nft_token.owner_id` | TC-008, TC-011 | — (business rule) | P0 |
| INV-024 | `count(active offers where (token, buyer)) ≤ 1` | TC-008 | — (business rule) | P0 |
| INV-025 | `prevalidate(all items) before first transfer; residual failure ⇒ status PARTIAL ∧ event market_bundle_partial ∧ refund ∧ G7` | TC-009, TC-010 | SEC-ORDER-001/002 | P0 |
| INV-026 | `sale.allowed_buyer ≠ null ⇒ buyer == sale.allowed_buyer` | TC-011 | — (business rule) | P0 |
| INV-027 | `∀token: royalty_rate ≤ 10% × fair_price; bundle Σ tanpa cap; |merged_receivers| ≤ 10` | TC-015, TC-009 | SEC-CONTRACT-005 | P0 |
| INV-028 | `token ∈ active bundle ⇒ ¬(listed separately ∨ offered separately)` | TC-009, TC-010, TC-043 | — (business rule) | P1 |
| INV-029 | `count(active launchpad phases) ≤ 1` | TC-021, TC-007 | SEC-CONTRACT-009 | P0 |
| INV-030 | `price_listing ≥ 0.01 Ⓝ ∧ offer_amount ≥ 0.01 Ⓝ` | TC-013, TC-022 | — (business rule) | P0 |
| INV-031 | `∀deposit masuk buy ⇒ ∃ jalur keluar` (resolve sukses/gagal **∨** `recover_stuck_purchase` permissionless) | TC-054 | SEC-ORDER-001 | P0 |

- **Test "unit"/"property"/"fuzz"** = belum ada TC bernomor; dibuktikan di level unit/property/fuzz (lihat §Cara diverifikasi nanti). TC bernomor = bukti sandbox/API/E2E.
- Baris tanpa SEC-ID = aturan bisnis terkunci (INV-023/024/026/028/030) — tetap wajib diuji, tetapi tidak dipetakan ke requirement keamanan register.
- **P0** = wajib hijau sebelum gate fasenya; **P1** = wajib sebelum mainnet gate.
- **Bukti sandbox dua-kontrak (TASK-006, ronde 24):** `market/tests/slice_sandbox.rs` men-deploy
  koleksi + market nyata dan membuktikan INV-001, INV-002, INV-003, INV-004, INV-007, INV-008,
  INV-011, INV-014, INV-015, INV-016, INV-019, INV-020, INV-023, INV-026, INV-027, INV-030, INV-031
  pada rantai sungguhan (bukan mock) — lihat [test-cases.md](../testing/test-cases.md) §blok
  "Sudah dibuktikan di level sandbox dua-kontrak". INV-005/006/009/010/012/013/017/018/021/022/024/
  025/028/029 tetap dibuktikan di level unit atau di-defer ke M1+ (dinyatakan eksplisit, bukan
  dibiarkan tanpa jejak).
- **Temuan F1 (TASK-036) — DIPERBAIKI ronde 29:** suite sandbox menemukan `nft_transfer` atas token
  yang **masih di-approve** gagal `ExcessiveUnlockError` (storage-accounting NEP-145) — bug kontrak
  koleksi, bukan pelanggaran INV di atas. **Akar masalah**: approval NEP-178 tidak melakukan storage
  accounting (`all_hooks` kontrak = `()`), jadi entry approval tidak pernah ditagihkan ke siapa pun;
  saat transfer, pencabutan approval membebaskan storage itu dan hook NEP-145 bawaan membacanya
  sebagai kredit lalu mencoba meng-`unlock_storage` ke **receiver** yang belum menyetor →
  `ExcessiveUnlockError`. **Perbaikan**: `transfer_hook` kustom
  (`RevokeApprovalsBeforeStorageAccounting`) mencabut approval **sebelum** hook NEP-145 mengambil
  snapshot `storage_usage`, sehingga delta yang dilihatnya nol. Regression test di dua level:
  **4 unit** di `contract/src/lib.rs` (2 gagal sebelum perbaikan) + 1 sandbox `task_036_*` di
  `market/tests/slice_sandbox.rs`. **Yang TIDAK diklaim**: INV-020 untuk map approval sendiri masih
  belum ditegakkan (`nft_approve` tidak menagih storage) — **TASK-040**, sebelum mainnet.

## Cara diverifikasi nanti (tooling DIPUTUSKAN — riset 2026-10-01)

1. **Unit test per invariant** — near-workspaces (sandbox, standar docs smart-contracts/testing/integration-test).
2. **Property/invariant test** — pola quickcheck/proptest di atas near-workspaces (referensi ekosistem: `near-prop`); generator urutan aksi (mint→list→offer→cancel/accept/stale/pause) → asersi seluruh INV.
3. **Fuzz** — `cargo-fuzz` (+ `arbitrary`) untuk fungsi murni non-chain: parsing payout, konversi u128, perhitungan fee/split.
4. **Review + audit** manual di M4 (mainnet gate).
