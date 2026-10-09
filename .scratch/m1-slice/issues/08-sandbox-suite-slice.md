# 08: Suite sandbox dua-kontrak (slice)

**What to build:** Satu suite yang membuktikan seluruh slice berjalan end-to-end di chain lokal sungguhan: mint → list → buy → royalti dan fee terbayar → jalur refund → race 20 pembeli. Inilah artefak yang membuat tesis jadi **terbukti**, bukan sekadar terimplementasi.

**Blocked by:** 04, 05, 06, 07 — suite ini menguji keempatnya; tidak ada yang bisa diuji sebelum ada.

**Status:** done

- [x] Suite menjalankan **dua kontrak nyata** (koleksi + market) di sandbox chain, bukan mock.
- [x] Jalur bahagia hijau: mint → list (2 tx) → buy → kreator + treasury + seller terbayar dengan angka yang benar.
- [x] NFT **terbukti** berada di wallet seller selama window listing (diasersi on-chain, bukan disimpulkan).
- [x] Jalur gagal hijau: payout tidak valid → refund penuh; storage kurang; listing stale (dua kasus); harga di bawah minimum.
- [x] Race: 20 pembeli bersamaan → tepat 1 penjualan, 19 refund penuh.
- [x] Pemulihan pembelian nyangkut hijau (TC-054).
- [x] Stale karena approval dicabut hijau (TC-053) **dan** stale karena kepemilikan pindah (TC-006).
- [x] Setiap invariant subset slice punya **minimal satu test**: INV-001..016, 023, 030, 031. Invariant M1+ (bundle/launchpad-penuh/pause) di-defer eksplisit, bukan dibiarkan tanpa jejak.
- [x] Test case slice yang disepakati semuanya runnable dan hijau: TC-001, 002, 013, 016, 017, 020, 022, 044, 047, 048, 053, 054.
- [x] Suite berjalan di CI dan **hijau** (bukan hanya hijau di lokal).

**Done-when (TASK-006):** Suite sandbox hijau; semua INV **slice** punya test (INV M1+ di-defer eksplisit).
→ **18 test sandbox hijau** di `market/tests/slice_sandbox.rs` (+ harness `market/tests/common/mod.rs`);
`cargo test --workspace` = **134 test** (73 market unit + 18 sandbox + 42 koleksi + 1 factory);
`fmt --check` + `clippy -D warnings` bersih; 3 wasm ter-build.
> **Koreksi (ronde 28):** catatan ronde 24 menulis "17 test sandbox / 133 test". Hitungan CI
> sungguhan adalah **18 / 134** — jumlah `#[tokio::test]` di `slice_sandbox.rs` memang 18, dan
> run CI menjalankan ke-18-nya. Angka lama keliru satu, bukan test yang hilang.

**Bukti CI (ronde 28):** AC "Suite berjalan di CI dan hijau" baru terverifikasi nyata di
[CI run 37895683629](https://github.com/keydeveloping/nearsea/actions/runs/37895683629) (PR #14,
`ubuntu-latest`), step `Unit + sandbox tests (near-workspaces)` **success**:
`test result: ok. 18 passed; 0 failed … finished in 221.50s` untuk `tests/slice_sandbox.rs`,
plus 73 (market unit), 42 (koleksi), 1 (factory). Sebelum ini AC tersebut hanya **diasersikan**:
branch-nya belum pernah di-push, jadi CI belum pernah menjalankannya.

**Bukti & catatan implementasi (ronde 24):**
- **Lokasi**: `market/tests/slice_sandbox.rs` (18 test) + `market/tests/common/mod.rs` (harness: deploy
  dua wasm nyata, mint/list/buy, baca saldo & event). Wasm dibaca dari `target/near/<crate>/<crate>.wasm`
  — artefak `cargo near build` yang sama dipakai gate CI, jadi CI **membangun wasm sebelum test**.
- **Sandbox hanya jalan di Linux/macOS** (binary nearcore tidak dipublikasikan untuk Windows). Harness
  diberi `#![cfg(unix)]`; dev-dependency di-scope `[target.'cfg(unix)'.dev-dependencies]` supaya gate
  lokal Windows (`cargo test`/`clippy`) tidak ikut menarik `near-workspaces`. Di Windows suite
  terkompilasi menjadi nol test; di CI (`ubuntu-latest`) dan WSL ia berjalan penuh.
- **Saldo dibaca setelah rantai "tenang"** (`Slice::quiesce` = `fast_forward(2)`). Query
  `near-workspaces` default memakai `Finality::Optimistic`; membacanya tepat setelah tx bisa melihat
  state antara (sebagian transfer belum masuk, sebagian refund deposit belum selesai) → assert split
  jadi salah. Temuan ini muncul saat implementasi dan didokumentasikan di harness.
- **TC-006 memakai urutan `nft_revoke` → `nft_transfer`** karena `nft_transfer` atas token yang masih
  ter-approve gagal di kontrak koleksi (temuan **F1**, `tasks/backlog.md` **TASK-036** — bug
  storage-accounting NEP-145, bukan staleness). Kasus yang diuji tetap kasus A sesungguhnya:
  kepemilikan berpindah sementara entry `Sale` masih ada → market menolak settle + refund penuh.
- **TC-054 meng-inject `pending_purchases`** langsung ke state sandbox (`patch` + borsh key
  `prefix 1 + SaleKey`), sesuai precondition TC ("inject: callback revert / gas habis"): jalur
  "callback mati" tidak bisa dicapai lewat kontrak NearSea sendiri karena `resolve_purchase` selalu
  menutup pending. Yang diuji: gerbang jeda blok, refund penuh permissionless, pemulihan listing.
- **Deferred eksplisit (M1+)**: bundle (INV-025/028, TC-009/010/014/015/043/046), offers
  (INV-005/006/024, TC-004/005/008/018/019), private listing penuh (INV-026 diuji; TC-011 penuh),
  launchpad penuh (INV-017/018 diuji lewat satu fase publik; TC-007/021), pause (INV-022, TC-012),
  API/admin/notifikasi/E2E (TC-023..TC-052).
- **Belum diklaim**: angka gas terukur per-call (anggaran `GAS_FOR_*` masih PROPOSED — TASK-006
  menyediakan harness-nya, kalibrasi nominal menyusul bersama `STORAGE_PER_SALE_BYTES`).

**Spec:** [docs/testing/test-cases.md](../../../docs/testing/test-cases.md) §Cakupan slice M1 · [docs/testing/testing-strategy.md](../../../docs/testing/testing-strategy.md) · [docs/security/smart-contract-invariants.md](../../../docs/security/smart-contract-invariants.md)
