# 04: NFT core — mint, metadata, approval, satu fase publik

**What to build:** Koleksi NFT yang bisa di-deploy, di-mint, dan di-transfer — dengan metadata standar dan approval. Ini paruh pertama tesis slice: NFT-nya ada dan pemiliknya yang memegang, bukan kontrak.

**Blocked by:** 01

**Status:** done — kontrak + 31 unit test hijau (ronde 19). Sisa satu item (deploy testnet) sengaja **tidak** dikerjakan: butuh persetujuan user.

- [x] Kontrak koleksi ter-deploy (testnet) dan `nft_mint` berhasil untuk wallet yang lolos validasi fase.
      → ⚠️ **Sebagian.** Kontrak **belum di-deploy** — deploy ke testnet wajib tanya user dulu
      ([git-workflow.md](../../../docs/development/git-workflow.md) §3), dan user belum memberi jawaban.
      Yang terbukti: wasm ter-build (`cargo near build non-reproducible-wasm`, 265 KB) dan `nft_mint`
      berhasil untuk wallet yang lolos validasi — dibuktikan di **unit test**, bukan testnet.
- [x] Mint **bergantung fase**: tanpa fase terkonfigurasi, mint mustahil; dengan satu fase publik terbuka, mint jalan. `set_phases` minimal (satu fase publik, `allowlist_required = false`) ikut di tiket ini — tanpanya koleksi tidak bisa di-mint sama sekali.
      → `test_mint_without_phases_is_impossible` (panic `LAUNCHPAD_PHASE_INACTIVE`), `test_mint_transfer_and_events`,
      `test_set_phases_allows_adjacent_windows`. `set_phases` = replace-all + validasi INV-029.
- [x] Deposit saat mint **cocok persis** dengan `harga fase × quantity` (bukan ≥); mismatch ditolak.
      → `test_mint_requires_exact_deposit` (deposit `price − 1` → `LAUNCHPAD_PRICE_MISMATCH`, INV-018);
      `test_mint_quantity_mints_sequential_ids` membuktikan `price × 3` diterima.
- [x] `nft_transfer` mewajibkan tepat 1 yocto dan memindahkan kepemilikan.
      → `test_transfer_requires_one_yocto` (panic `Requires attached deposit of exactly 1 yoctoNEAR`) +
      `test_mint_transfer_and_events` (kepemilikan alice → bob).
- [x] `nft_metadata` dan metadata per-token mengembalikan name/symbol/base_uri yang dikonfigurasi.
      → `test_new_sets_owner_metadata_and_royalty` (name/symbol/base_uri) + `nft_token` mengembalikan
      metadata per-token (`title` terisi saat mint).
- [x] NEP-178 lengkap: `nft_approve`, `nft_is_approved`, `nft_revoke`, `nft_revoke_all`.
      → `test_approvals_lifecycle` (approve → is_approved → revoke → approve → revoke_all).
      Keempat method terverifikasi ada di ABI wasm.
- [x] Event NEP-297 ter-emit untuk mint dan transfer (envelope `EVENT_JSON:`, satu baris).
      → `test_mint_transfer_and_events` meng-assert log memuat `EVENT_JSON:` dengan
      `"standard":"nep171"` + `"event":"nft_mint"` / `"event":"nft_transfer"`, dan event NearSea
      `launchpad_mint` (`"standard":"x-nearsea-market"`). Bentuk `data` = **array** sesuai
      [webhooks.md](../../../docs/api/webhooks.md) — derive `near-sdk-contract-tools` memancarkan objek,
      jadi dipakai helper envelope `SingleEvent` (dipakai ulang kontrak market nanti).
- [x] Storage mengikuti NEP-145: minter membayar storage token-nya; mint tanpa deposit cukup gagal (bukan diam-diam memakai saldo kontrak).
      → `test_mint_without_storage_deposit_fails` (panic `CHAIN_REVERT`). Pertumbuhan state saat mint
      ditagih ke minter lewat hook `Nep171StorageAccountingHook` (derive `NonFungibleToken`).
- [x] Init menolak `royalty_bps` di luar `1..=1000`.
      → `test_init_rejects_royalty_below_min` (0) + `test_init_rejects_royalty_above_cap` (1001) →
      panic `INVALID_ROYALTY`.
- [x] Init memakai `PanicOnDefault` — memanggil method tanpa init = panic, tidak ada jalur init ulang.
      → `#[derive(PanicOnDefault)]` + `Owner::init` sekali (SEC-CONTRACT-002). Tanpa state default,
      method apa pun sebelum `new` = panic.
- [x] Storage-key prefix terdokumentasi dan tidak berubah sembarangan (layout stabil untuk upgrade).
      → enum `StorageKey { Phases, Allowlist, MintedByWallet }` di kode + tabel prefix di
      [nft-collection.md](../../../docs/contracts/nft-collection.md) §7. Prefix NEP (`~$145`/`~$171`/
      `~$177`/`~$178`/`~$181`/`~o`) dikelola derive; milik kontrak sengaja bukan pola `~*`.
- [x] Sandbox hijau: mint → transfer → events (TC-001).
      → ⚠️ **Unit-level, bukan sandbox.** TC-001 (Layer: sandbox) dibuktikan di unit test sekarang;
      suite **sandbox** (`near-workspaces`, 2 kontrak) milik TASK-006 (slice ticket `08`) — lihat
      [test-cases.md](../../../docs/testing/test-cases.md) § Cakupan slice M1.

**Done-when (TASK-002):** `cargo test` hijau; mint + transfer + events lolos TC-001.
→ ✅ **`cargo test` hijau** (34 test workspace; 31 kontrak) dan **mint + transfer + events lolos TC-001**
di level unit. Gate lain juga hijau: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`
(0 warning), `cargo near build non-reproducible-wasm` (265 KB, ABI memuat semua method NEP + ekstensi).

**Catatan keputusan implementasi (ronde 19):**
- `data` event custom = **array** (`[{…}]`) sesuai `webhooks.md`; helper `SingleEvent<T>` dipakai karena
  derive `near-sdk-contract-tools::event` memancarkan objek — kontrak market/factory memakai helper yang sama.
- Batas window fase `[starts_at, ends_at)` ditetapkan **DECIDED** (sebelumnya PROPOSED): `starts_at`
  inklusif, `ends_at` eksklusif, diuji di dua sisi batas. Fase berdampingan valid.
- `allowlist_required` sudah diimplementasikan penuh (bukan hanya fase publik) karena jalur mint harus
  satu kode untuk M1 dan M1+; `max_mints` per baris = `min(max_per_wallet, max_mints)`.
- `#[allow(clippy::too_many_arguments)]` pada `new` — 7 argumen adalah ABI yang dipatok
  [nft-collection.md](../../../docs/contracts/nft-collection.md) §1 (factory TASK-012 memanggilnya),
  jadi tidak digabung ke struct.
- Nilai `MAX_MINT_PER_CALL=10`, `MAX_PHASES=20`, `MAX_ALLOWLIST_BATCHCH=200`, `STORAGE_MIN_BYTES=300`
  dipakai sebagai konstanta; kalibrasi sandbox menyusul di TASK-006.

**Spec:** [docs/contracts/nft-collection.md](../../../docs/contracts/nft-collection.md) §1–§3, §6–§7 · [docs/features/launchpad.md](../../../docs/features/launchpad.md) · INV-017/018/019/021/029 · SEC-CONTRACT-002/008
