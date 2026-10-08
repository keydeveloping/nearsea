# 06: Market listing — storage, 2-tx, dual verification

**What to build:** Seller bisa memasang NFT untuk dijual dengan harga tetap **tanpa menyerahkan tokennya**. Listing butuh dua transaksi (approve, lalu list), dan market memverifikasi sendiri kepemilikan **dan** approval sebelum menerima — jadi listing tidak pernah lahir dari klaim sepihak.

**Blocked by:** 04 — butuh kontrak koleksi dengan `nft_token`/`nft_is_approved` untuk diverifikasi.

**Status:** done — kontrak + 26 unit test baru hijau (ronde 22).

- [x] `list_nft_for_sale` membuat listing **hanya** bila verifikasi silang milik market lolos: pemanggil adalah pemilik token **dan** market sedang di-approve.
      → `test_list_creates_sale_after_dual_verification` (jalur sukses) + `test_list_rejected_when_caller_is_not_token_owner`,
      `test_list_rejected_when_market_is_not_approved`. Callback `process_listing` menolak `token.owner_id != seller_id` **atau** `approved != Ok(true)`.
- [x] Verifikasi dijalankan sebagai cross-contract call ke koleksi, bukan dari argumen pemanggil (tidak percaya klaim).
      → `list_nft_for_sale` membangun 2 promise view (`nft_token` + `nft_is_approved`) ke `nft_contract_id`; tidak ada
      argumen "owner"/"approved" di ABI. Dibuktikan dengan membaca receipt: `test_list_creates_sale_after_dual_verification`.
- [x] NFT **tetap di wallet seller** selama listing — non-custodial, dibuktikan dengan pembacaan on-chain, bukan diklaim.
      → Test yang sama meng-assert receipt jalur listing **tidak** memuat `nft_transfer`/`nft_transfer_payout` — hanya 2 view + callback.
      `Sale` menyimpan `owner_id` (bukan token); tidak ada field yang menyimpan NFT.
- [x] Listing butuh deposit storage NEP-145; listing dengan deposit kurang gagal.
      → `test_list_rejected_without_storage_deposit` (tanpa deposit) + `test_list_rejected_when_attached_deposit_below_minimum`
      (`required − 1` yocto). Bounds `min = storage_per_sale()` di `new`; `test_new_sets_owner_and_storage_bounds`.
- [x] `remove_sale` membatalkan listing dan mencabut approval market; mewajibkan tepat 1 yocto.
      → ⚠️ **Sebagian, dengan koreksi.** 1 yocto ✅ (`test_remove_sale_requires_one_yocto`), hapus entry ✅
      (`test_remove_sale_deletes_listing_and_emits_delist`), storage kembali ✅ (`test_remove_sale_releases_storage_to_seller`).
      **Pencabutan approval TIDAK dilakukan** — `nft_revoke_token` yang disebut AC ini **tidak ada di NEP-178**
      (hanya `nft_revoke`/`nft_revoke_all`, owner-only). Keputusan + alasannya di
      [contracts/market.md](../../../docs/contracts/market.md) §2a; TC-044 dikoreksi mengikuti.
- [x] `update_price` mengubah harga in-place dan menolak harga di bawah minimum.
      → `test_update_price_changes_price_in_place` (harga baru + `approval_id` tetap) + `test_update_price_rejects_below_minimum`.
- [x] Harga di bawah 0,01 Ⓝ ditolak.
      → `test_list_rejects_price_below_minimum`, `test_update_price_rejects_below_minimum`; batas inklusif
      diuji `test_list_accepts_price_exactly_at_minimum` (INV-030).
- [x] Listing ganda untuk token yang sama ditolak.
      → `test_list_rejects_duplicate_listing` (`CONFLICT_ALREADY_LISTED`), dicek dua kali: di `list_nft_for_sale`
      (sinkron) dan di `process_listing` (setelah XCC, INV-007).
- [x] Harga/listing tidak bisa diubah oleh selain pemilik token.
      → `test_remove_sale_rejected_for_non_owner`, `test_update_price_rejected_for_non_owner`; otorisasi
      listing sendiri datang dari dual verification (owner on-chain ≠ pemanggil → callback panic).
- [x] Event `market_list` dan `market_delist` ter-emit dengan payload sesuai katalog.
      → assert payload di `test_list_creates_sale_after_dual_verification` (`seller`/`token_id`/`price_yocto`/`approval_id`),
      `test_remove_sale_deletes_listing_and_emits_delist`, `test_update_price_changes_price_in_place`
      (harga lama + baru). Bentuk `data` = array (envelope `SingleEvent`, sama dengan koleksi).
- [x] **Tambahan (bukan di AC, wajib untuk flow 2-tx):** `nft_on_approve` (NEP-178 receiver) ada dan
      **tidak** membuat listing (ADR-002) — `test_nft_on_approve_accepts_valid_payload`,
      `test_nft_on_approve_does_not_create_listing`, `test_nft_on_approve_rejects_empty_payload`,
      `test_nft_on_approve_rejects_self_as_owner`. Tanpa method ini, tx-1 (`nft_approve(market, msg)`)
      memanggil method yang tidak ada.
- [ ] Sandbox hijau: list → cancel, plus jalur gagal (TC-002 paruh list, TC-013, TC-020, TC-044).
      → ⚠️ **Unit-level, bukan sandbox.** Semua jalur di atas dibuktikan di unit (26 test); suite **sandbox**
      (`near-workspaces`, 2 kontrak) milik tiket `08` — lihat
      [test-cases.md](../../../docs/testing/test-cases.md) §Cakupan slice M1 dan catatan "Sudah dibuktikan di level unit untuk paruh listing".

**Done-when (TASK-004):** List/cancel + storage deposit lolos (INV-020, TC-002).
→ ✅ **List/cancel + storage deposit lolos** di level unit: dual verification dua kasus, non-custodial
(dibuktikan receipt), storage kurang → revert, storage kembali saat cancel, harga min inklusif, duplikat,
owner-only, paused (INV-022). Paruh **buy** TC-002 dan angka gas penuh tetap milik TASK-005/006.

**Gate lokal (ronde 22):** `cargo fmt --all -- --check` ✅ ·
`cargo clippy --all-targets -- -D warnings` (0 warning) ✅ · `cargo test --workspace` ✅
(**75 test**: 1 factory + 26 market + 42 koleksi) · `cargo near build non-reproducible-wasm --no-abi`
(197 KB; ABI memuat `list_nft_for_sale`, `process_listing`, `remove_sale`, `update_price`, view, dan
method NEP-145/Pause) ✅. Sama seperti tiket sebelumnya, build ABI penuh dijalankan CI (Linux) karena
keterbatasan Windows ([ci-cd.md](../../../docs/development/ci-cd.md) §14).

**Catatan keputusan implementasi (ronde 22):**
- `process_listing` menerima konteks sebagai **argumen eksplisit** (`seller_id`, `nft_contract_id`, …),
  bukan dari `env::predecessor_account_id()` — predecessor di callback adalah market sendiri, jadi pemanggil
  asli harus diteruskan. `#[private]` tetap menjaga callback hanya bisa dipanggil kontrak ini (INV-013).
- `nft_token` mengembalikan `Option<Token>` → callback bertipe `Result<Option<Token>, PromiseError>`;
  `None` (token tidak ada) = verifikasi gagal → `CHAIN_REVERT`.
- Pesan panic = kode registry, bukan teks library (mis. "Signer is not NFT owner" dari RESEARCH.md §10.4
  adalah contoh tutorial) — [error-handling.md](../../../docs/development/error-handling.md) §4.
- Deposit storage dicek **sinkron** di `list_nft_for_sale` (setelah kredit ke saldo seller), bukan di
  callback — supaya deposit kurang = revert tx (jelas bagi FE), bukan callback gagal senyap.
- `GAS_FOR_DUAL_VERIFY` **diturunkan** dari komponennya (`2 × GAS_FOR_NFT_VIEW + GAS_FOR_PROCESS_LISTING`)
  supaya tidak ada angka kedua yang bisa menyimpang; nilainya PROPOSED sampai diukur di TASK-006.
- `get_sales` memakai `limit` clamp 100 (bukan error) dan `offset` di luar `usize` → hasil kosong
  (pola §Discovery [features/marketplace.md](../../../docs/features/marketplace.md)). Urutan hasil
  **tidak dijamin** (urutan map) — sort `newest` dilakukan FE atas hasil view (ADR-004), jadi view ini
  tidak menjanjikan urutan apa pun.
- **Batas storage PROPOSED = plafon, bukan nominal eksak.** `assert_storage_for_listing` mengecek
  `available ≥ storage_per_sale()` (500 byte) secara **sinkron**; pertumbuhan **aktual** ditagih di
  callback `settle_storage_delta`. Bila entry `Sale` ternyata tumbuh >500 byte (mis. `token_id` sangat
  panjang), callback panic `CHAIN_REVERT` — listing tidak jadi, **tanpa kehilangan dana**: deposit tetap
  di saldo storage seller dan bisa **dipakai untuk listing lain**. Nominal final diukur di TASK-006 (OQ-007).
  **Catatan koreksi:** "deposit bisa ditarik" **tidak** berlaku untuk deposit tepat `min` — NEP-145
  menolak `storage_withdraw` bila `total` akan turun di bawah `bounds.min`, jadi deposit minimum tidak
  pernah bisa ditarik selama akun tetap terdaftar. Ia reusable, bukan withdrawable. (Salah satu alasan
  `min` diisi `storage_per_sale()`, bukan angka terkecil.)
- **Pause ditegakkan juga di callback.** `process_listing` mengecek `is_paused` sendiri karena ia
  receipt terpisah: kontrak bisa di-pause antara tx listing dan callback, dan listing baru tetap
  "mutasi baru" yang dilarang INV-022 → `CHAIN_PAUSED`. Diuji `test_process_listing_rejected_when_paused`.
  (`storage_deposit` bawaan derive NEP-145 tidak di-pause-gate; ia hanya menambah saldo, bukan membuat
  listing — `storage_withdraw` memang harus tetap jalan saat paused.)
- **Urutan callback belum diuji end-to-end.** `process_listing` mengasumsikan `token` = hasil
  `nft_token` (indeks 0) dan `approved` = hasil `nft_is_approved` (indeks 1), sesuai urutan `.and()`.
  Di unit test callback dipanggil langsung, jadi urutan promise nyata belum terbukti; kalau tertukar,
  deserialisasi gagal → `CHAIN_REVERT` (gagal aman, bukan lubang keamanan). Dibuktikan di sandbox TASK-006.

**Spec:** [docs/contracts/market.md](../../../docs/contracts/market.md) §2, §2a, §5 · [docs/features/marketplace.md](../../../docs/features/marketplace.md) §Flow — List · INV-007/013/020/030 · ADR-002 (dual verification saat list), SEC-CONTRACT-001/002/007/011, SEC-ORDER-005.

> **Catatan requirement:** SEC-ORDER-004 ("re-verify ownership+approval **di settle**") paruh
> settle-nya milik **TASK-005** — di tiket ini yang dibuktikan adalah verifikasi saat **list**
> (anti-klaim, ADR-002), yang memakai dua view call yang sama. SEC-CONTRACT-007 (Pausable)
> terpenuhi: `list_nft_for_sale`/`update_price` menolak saat paused, `remove_sale` tetap boleh (INV-022).
