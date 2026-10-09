# 05: Royalti NEP-199

**What to build:** Saat market menyelesaikan penjualan, koleksi memberi tahu cara membagi uangnya — jatah kreator, dibatasi 10% dari harga. Ini paruh kedua tesis: royalti ditegakkan on-chain, bukan dijanjikan.

**Blocked by:** 04 — butuh token yang ada dan bisa di-transfer sebelum payout bisa diuji.

**Status:** done — kontrak + 11 unit test baru hijau (ronde 21). Satu AC (TC-003 versi sandbox) sengaja **tidak** diklaim: validasi payout di sisi market + refund butuh suite sandbox dua-kontrak yang baru ada di tiket `08`.

- [x] `nft_transfer_payout` mengembalikan payout map dengan kreator sebagai satu-satunya penerima.
      → `test_transfer_payout_moves_token_and_returns_creator_royalty` — `payout.len() == 1` dan
      `payout[creator] == floor(harga × 500 / 10_000)`. Derive: `royalty_payout()` di `contract/src/lib.rs`.
- [x] Amount = `floor(harga × royalty_bps / 10_000)`.
      → helper murni `royalty_amount(balance, bps)`; `test_transfer_payout_never_exceeds_ten_percent`
      menguji 4 rate × 6 basis dan membandingkan hasilnya dengan `balance × bps / 10_000` (floor eksak).
- [x] Royalti dibatasi **10% per token**; konfigurasi di atas cap ditolak saat init.
      → `MAX_ROYALTY_BPS = 1000` sudah ditegakkan di `new` (TASK-002); di sisi payout
      `test_transfer_payout_at_cap_is_exactly_ten_percent` (1000 bps → tepat 10%) dan invarian
      `amount × 10 ≤ balance` di seluruh matriks test.
- [x] Payout diturunkan dari konfigurasi royalti **level kontrak** (bukan `TokenMetadata.extra`, bukan per-token).
      → `royalty_payout` membaca `self.royalty_bps` + `self.creator_id`; dibuktikan
      `test_payout_uses_contract_level_royalty_config` (rate 750 bps → payout mengikuti config, bukan metadata).
- [x] View `royalty_config()` mengembalikan `{ receiver, bps }` untuk dipakai market.
      → sudah ada sejak TASK-002; test di atas meng-assert `config.receiver`/`config.bps` sebagai sumber payout.
- [x] Gas 15 Tgas yang di-attach market ke `nft_transfer_payout` **terbukti cukup** untuk jalur payout.
      → ✅ **Cukup, dengan catatan.** `test_transfer_payout_fits_market_gas_budget` menjalankan
      panggilan dengan `prepaid_gas = 15 Tgas` (mock near-sdk memakai VMLogic asli, jadi prepaid gas
      benar-benar ditegakkan) dan meng-assert pemakaian **< 5 Tgas**; angka terukurnya **~1,70 Tgas**.
      Angka itu biaya host function (storage/register) **tanpa** gas CPU wasm, jadi ia batas bawah;
      pengukuran penuh tetap di sandbox (TASK-006), dan konstanta `GAS_FOR_NFT_TRANSFER` di market
      tetap PROPOSED.
- [ ] Sandbox hijau: payout ≤10% untuk beberapa harga berbeda, termasuk batas bawah (dust) dan batas atas cap (INV-003/027, TC-003 sebagian).
      → ⚠️ **Unit-level, bukan sandbox.** Batas bawah (dust) dan batas atas (cap) sudah diuji di unit:
      `test_transfer_payout_floors_to_zero_on_dust_basis` (19 → 0, 20 → 1 yocto di 500 bps) dan
      `test_transfer_payout_at_cap_is_exactly_ten_percent`. Suite **sandbox** (`near-workspaces`,
      2 kontrak) milik tiket `08` — lihat [test-cases.md](../../../docs/testing/test-cases.md) §Cakupan slice M1.

**Done-when (TASK-003):** Payout royalti ≤10% teruji (INV-003/027).
→ ✅ **Payout ≤10% teruji** untuk 4 rate × 6 basis (termasuk dust dan cap) di level unit, plus
perpindahan kepemilikan dalam panggilan yang sama, wajib 1 yocto, penolakan pengirim tanpa approval,
`max_len_payout` terlalu kecil, dan approval invalid setelah transfer (INV-011).

**Gate lokal (ronde 21):** `cargo fmt --all -- --check` ✅ ·
`cargo clippy --all-targets -- -D warnings` (0 warning) ✅ · `cargo test --workspace` ✅
(**45 test**: 1 factory + 2 market + 42 koleksi) · `cargo near build non-reproducible-wasm --no-abi`
(272 KB; ABI wasm memuat `nft_transfer_payout`) ✅. Build ABI penuh gagal lokal karena
`LNK2019: unresolved external symbol value_return` — keterbatasan Windows yang sudah tercatat
([ci-cd.md](../../../docs/development/ci-cd.md) §14); CI (Linux) yang memverifikasinya.

**Catatan keputusan implementasi (ronde 21):**
- Otorisasi mengikuti `nft_transfer`: `approval_id` → jalur approval NEP-178; tanpa `approval_id` → owner.
  Ini yang dibutuhkan market (selalu mengirim `approval_id` listing) dan tetap sesuai NEP-199
  ("menerima semua argumen `nft_transfer`").
- `approval_id` bertipe `Option<u64>` di ABI (dipatok [market.md](../../../docs/contracts/market.md)),
  dipetakan ke `u32` internal derive NEP-178. Id di luar rentang `u32` diperlakukan sebagai
  "bukan approval" (jalur owner) — bukan panic, karena id itu memang tidak mungkin valid.
- `max_len_payout = Some(n)` dengan `payout.len() > n` → panic `CHAIN_REVERT`; `None` = tanpa plafon.
  Payout koleksi selalu 1 penerima, jadi `max_len_payout = 10` dari market selalu lolos.
- Payout ber-amount `0` (basis di bawah granularitas rate) **tetap dikembalikan** sebagai entri —
  kontrak tidak menyembunyikan penerima dust; INV-003 divalidasi market pada payout final.
- Aritmetika `checked_mul` → floor `/10_000`; overflow → `CHAIN_REVERT` (SEC-CONTRACT-005).

**Spec:** [docs/contracts/nft-collection.md](../../../docs/contracts/nft-collection.md) §2/§4 · [docs/features/payments.md](../../../docs/features/payments.md) §Algoritma · INV-003/027
