# Milestones

> Checkpoint yang bisa dibuktikan (demo-able), bukan rentang tanggal.
>
> **⚠️ Rombak ronde 17 (ADR-016):** M1 dipotong jadi **vertical slice** (8 task) yang membuktikan tesis
> inti lebih dulu. Sisa fitur MVP lama pindah ke **M1+**. **ID task tidak berubah** — hanya
> pengelompokan. Alasan: proyek ini pembelajaran/portofolio, dan 23 task sebelum umpan balik pertama
> = risiko mangkrak tinggi.

## M0 — Fondasi dokumen & scaffold

- Semua dokumen inti terisi (hasil tanya-jawab + audit), stack final, repo scaffold.
- Done when: agent bisa mulai koding tanpa bertanya arah.
- **Status TASK-001 (2026-10-07): ✅ `done`.** Workspace + manifest + workflow ada dan gate hijau di dua tempat: lokal (fmt/clippy/test kontrak; lint/format/typecheck/test/build FE) **dan** CI GitHub di branch `dev` (PR #1, commit `223015b` — run CI + Security `success`). Repo remote: `github.com/keydeveloping/nearsea`.
- **Status TASK-031 (2026-10-07, ronde 18c): ✅ `done`.** Model 3 branch ditegakkan platform: proteksi aktif di `dev`/`testnet`/`mainnet` (PR wajib, force-push & delete diblokir termasuk admin, 5 required status checks, conversation resolution) + ruleset tag protection. Bukti: push langsung ke `dev` ditolak GitHub. Catatan terbuka: required approval ditunda (repo satu akun) — dinaikkan saat maintainer kedua ada.
- **Status TASK-032 (2026-10-07, ronde 20): ✅ `done` (mekanisme).** Versi artefak kini tertanam di build dan dibaca dari luar (NEP-330) untuk ketiga crate; `release.yml` menegakkan **versi manifest == tag** + membuktikan metadata tertanam == tag + melampirkan artifact ke GitHub Release; `ci.yml` membangun ABI + memverifikasi metadata tiap PR. Bukti lokal: metadata ketiga wasm memuat `version=0.1.0` + `link=https://github.com/keydeveloping/nearsea`; gate fmt/clippy/test hijau. **Catatan terbuka (bukan blocker M0):** **tag pertama belum dibuat** — tag sah hanya di `testnet`/`mainnet` setelah PR promosi, dan itu **wajib persetujuan user** ([git-workflow.md](../docs/development/git-workflow.md) §3/§12). Rilis pertama yang disiapkan: `contract-v0.1.0`. **M0 tertutup.**

## M1 — Vertical slice: mint → list → buy (testnet) ⭐ **milestone pertama yang wajib**

- **Tesis yang dibuktikan:** mint NFT → list (2-tx, non-custodial) → orang lain beli → royalti + fee
  terbayar → **NFT tidak pernah dipegang kontrak**.
- **8 task:** TASK-001, 002, 003, 004, 005, 006, 007, 008.
- Koleksi **di-deploy manual** (`cargo near deploy`) — **factory tidak dipakai** di slice.
- Market hanya menerima **koleksi NearSea** di slice (menutup temuan H1).
- **Testnet saja.** Tanpa VPS, domain, IPFS pinning, audit, atau DAO.
- Done when: **dua akun testnet** bisa list→buy end-to-end via UI; seller + royalti terbayar dengan
  fee 2% terlihat di breakdown; sandbox test hijau; NFT tetap di wallet seller selama listing
  (dibuktikan, bukan diklaim).
- **Status TASK-002 (2026-10-07, ronde 19): ✅ `done` (kode).** Kontrak koleksi `contract/src/lib.rs`
  mengimplementasikan NEP-171/177/178/181 + NEP-145 + event NEP-297 + `nft_mint` launchpad-aware +
  `set_phases` (satu fase publik, dan allowlist penuh sekalian). Gate lokal hijau: `fmt --check`,
  `clippy -D warnings` (0 warning), `cargo test --workspace` (**34 test**), build wasm 265 KB dengan ABI
  lengkap. **Belum di-deploy** ke testnet — deploy wajib tanya user dulu ([git-workflow.md](../docs/development/git-workflow.md) §3);
  karena itu item "kontrak ter-deploy" pada tiket slice `04` ditandai ⚠️ sebagian. Sisa slice M1:
  TASK-003 → 004 → 005 → 006, dan jalur FE 007 → 008.
- **Status TASK-003 (2026-10-08, ronde 21): ✅ `done` (kode).** Royalti NEP-199:
  `nft_transfer_payout` (transfer + payout dalam satu panggilan, 1 yocto, `max_len_payout` dihormati)
  + helper murni `royalty_amount = floor(balance × bps / 10_000)` dengan `checked_mul`. Payout dari
  konfigurasi royalti **level kontrak**, satu penerima (`creator_id`), selalu ≤10% (INV-027). Bukti:
  **45 test** workspace (11 baru — dust `19` → `0` / `20` → `1` yocto di 500 bps, tepat 10% di cap,
  matriks 4 rate × 6 basis, wajib 1 yocto, penolakan pengirim tanpa approval, approval invalid setelah
  transfer/INV-011), `fmt`/`clippy -D warnings` bersih, wasm 272 KB (ABI memuat `nft_transfer_payout`).
  **Belum diklaim**: angka gas 15 Tgas + TC-003 versi sandbox (sisi market validasi payout & refund) —
  keduanya butuh suite dua-kontrak TASK-006. Sisa slice M1: TASK-004 → 005 → 006, dan jalur FE 007 → 008.
- **Status TASK-004 (2026-10-08, ronde 22): ✅ `done` (kode).** Jalur listing market di `market/src/lib.rs`:
  `list_nft_for_sale` 2-tx non-custodial + callback `process_listing` `#[private]` yang memverifikasi
  sendiri lewat dua view XCC (kepemilikan **dan** approval — SEC-ORDER-004, ADR-002), `remove_sale`,
  `update_price`, view listing, storage NEP-145 (bounds `min = storage_per_sale()`), Pausable (INV-022),
  dan tiga event kanonik. Bukti: **75 test** workspace (32 baru) — non-custodial dibuktikan dengan membaca
  receipt (hanya 2 view + callback; tidak ada `nft_transfer*`); `fmt`/`clippy -D warnings` bersih; wasm
  197 KB. **Koreksi dokumen**: `nft_revoke_token` tidak ada di NEP-178 → `remove_sale` tidak mencabut
  approval ([contracts/market.md](../docs/contracts/market.md) §2a). **Belum diklaim**: paruh buy TC-002,
  race TC-016/017, TC-022, TC-048, angka gas penuh — butuh TASK-005/006. Sisa slice M1: TASK-005 → 006,
  dan jalur FE 007 → 008.
- **Status TASK-007 (2026-10-08, ronde 25): ✅ `done` (kode).** Wallet connect di
  `frontend/features/auth/` (`@hot-labs/near-connect` 0.11.4 + `near-connect-hooks` 1.1.6, di-pin
  exact): state app-wide via `WalletContext`/`useWallet()` dengan satu tipe `WalletApi`, header dengan
  indikator jaringan + kontrol wallet, banner peringatan jaringan + flag gerbang
  `transactionsDisabled`, mode baca tanpa wallet. Gate FE hijau: `lint`, `format:check`, `typecheck`,
  `test` (**52 test**), `build` (4/4 statis); bundle bersih dari secret. **Belum diklaim**: connect
  dengan wallet testnet nyata (butuh akun testnet + browser; jalur emas Playwright = TASK-008), dan
  penonaktifan tombol aksi berbasis `transactionsDisabled` (tombol aksi baru ada di TASK-008). Sisa
  slice M1: TASK-008 saja. Lihat juga TASK-037 (tinjauan dependency `function-call-key-plugin`).
- **Status TASK-008 (2026-10-09, ronde 26): 🔄 `in-progress` (kode lengkap, satu AC menunggu deploy).**
  UI browse/list/buy di `frontend/features/marketplace/` + `lib/format/fees.ts`,
  `lib/errors/market-errors.ts`, `lib/near/contracts.ts`, namespace i18n `marketplace`/`errors`.
  Listing = **dua transaksi** (`nft_approve` → baca `approval_id` konkret → `list_nft_for_sale` +
  deposit storage NEP-145); buy = **re-verify sebelum signing** (`CONFLICT_PRICE_CHANGED`/
  `CONFLICT_STALE` tanpa tanda tangan) dengan deposit = harga hasil verifikasi; stale disembunyikan
  dari grid dan ditampilkan "no longer available" di halaman token; breakdown fee/royalti BigInt
  sebelum konfirmasi. Gate FE hijau: `lint`, `format:check`, `typecheck`, `test` (**127 test**,
  20 di antaranya jalur emas browse→list→buy), `build` (`/` statis + `/token/...` partial prerender).
  **Yang belum**: jalur emas **Playwright** dengan dua akun testnet — butuh kontrak ter-deploy di
  testnet, dan deploy testnet **wajib persetujuan user** ([git-workflow.md](../docs/development/git-workflow.md) §3).
  Prasyarat baru: `@tanstack/react-query` + `NEXT_PUBLIC_MARKET_CONTRACT_ID`. **Koreksi**: klaim tiket 09
  bahwa kode `function-call-key-plugin` ter-tree-shake dari bundle **tidak benar** (kode penandatanganan
  lokal ada di chunk klien) → TASK-037 naik prioritas.

- **Status TASK-005 (2026-10-08, ronde 23): ✅ `done` (kode).** Jalur settlement di `market/src/lib.rs`:
  `buy` (tulis `pending_purchases` sebelum optimistic removal — INV-031) → `process_purchase` (dual
  verification saat settle; stale dua kasus INV-016 → refund + event; verifikasi tak pasti → refund +
  restore `Sale`) → `nft_transfer_payout` (1 yocto, `max_len_payout=10`) → `resolve_purchase` (validasi
  payout UNTRUSTED `1..=10` penerima / `amount>0` / `Σ ≤ harga−fee`; fee → treasury, royalti → receiver,
  residual → seller, kelebihan deposit → buyer; gagal → refund penuh + restore), `recover_stuck_purchase`
  (permissionless, INV-031), `update_fee_bps`/`update_treasury`, `fee_bps`/`treasury` di init. Bukti:
  **116 test** workspace (41 baru) — Σ keluar == Σ masuk exact, 6 jalur payout invalid → refund, stale
  dua kasus, recovery dengan/tanpa restore, gas worst case; `fmt`/`clippy -D warnings` bersih; wasm 239 KB.
  **Dua koreksi dokumen**: (1) `withdraw_fees` dihapus — fee masuk treasury saat settlement
  ([contracts/market.md](../docs/contracts/market.md) §4a; TC-047 dialihkan ke `update_fee_bps`/`update_treasury`);
  (2) aturan `sisa ≤1 yocto` (INV-002) dibatalkan — residual = proceeds seller, bukan dust.
  **Belum diklaim**: race TC-016/017, TC-022, TC-048, angka gas terukur — butuh TASK-006. Sisa slice M1:
  TASK-006 (suite sandbox), jalur FE 007 → 008.

- **Status TASK-006 (2026-10-08, ronde 24): ✅ `done`.** Suite sandbox dua-kontrak di
  `market/tests/slice_sandbox.rs` (**18 test**) + harness `market/tests/common/mod.rs` — men-deploy wasm
  **koleksi + market nyata** di sandbox chain (bukan mock) dari `target/near/<crate>/`, jalur yang sama
  dipakai gate CI. Yang dibuktikan: **TC-002** jalur bahagia penuh dengan angka exact (fee 2% → treasury,
  royalti 5% → kreator, proceeds seller = residual; NFT **terbukti** tetap di wallet seller selama
  listing — non-custodial; approval lama invalid setelah transfer), **TC-001**, **TC-003** (payout tidak
  valid dari koleksi pihak ketiga — fixture `market/tests/fixtures/rogue-collection` — → refund penuh +
  listing dipulihkan, empat bentuk payout invalid + kontrol `Valid`), **TC-006/TC-053** (stale dua kasus),
  **TC-013**, **TC-016** (race 20 pembeli → tepat 1 menang, 19 revert `CONFLICT_SOLD` dengan deposit
  kembali penuh), **TC-017**, **TC-020**, **TC-022**, **TC-044**, **TC-047**, **TC-048**, **TC-054**
  (recovery permissionless), plus INV-014 dan INV-004/INV-027 (split konsisten saat fee & royalti di cap).
  Bukti: `cargo test --workspace` = **134 test** (73 market unit + 18 sandbox + 42 koleksi + 1 factory),
  `fmt --check` + `clippy -D warnings` bersih, 4 wasm ter-build. **Bukti CI (ronde 28)**: suite ini
  `#![cfg(unix)]`, jadi verifikasi nyatanya = [run 37895683629](https://github.com/keydeveloping/nearsea/actions/runs/37895683629)
  (PR #14, `ubuntu-latest`) — `18 passed; 0 failed`, 221s. **Sandbox hanya jalan di Linux/macOS**
  (binary nearcore tidak dipublikasikan untuk Windows) — dev-dependency di-scope `cfg(unix)` supaya gate
  lokal Windows tidak berubah; CI membangun wasm + fixture sebelum test.
  **Temuan F1 (bug nyata, bukan test yang salah):** `nft_transfer` atas token yang **masih di-approve**
  gagal `ExcessiveUnlockError` (storage-accounting NEP-145 di kontrak koleksi) → dicatat sebagai
  **TASK-036**; jalur uang NearSea (`list` → `buy`) tidak terkena. **Ronde 29: TASK-036 diperbaiki**
  (hook `RevokeApprovalsBeforeStorageAccounting` mencabut approval sebelum snapshot storage accounting)
  — regression test unit + sandbox, dan TC-006 kembali memakai jalur aslinya. **Ronde 28: tiket 05/06/07/08
  semuanya mendarat di `dev`** (PR #12/#13/#14); AC sandbox tiket 05/06 ditutup dengan bukti run CI di
  atas. Sisa slice M1: hanya jalur emas Playwright (tiket 10) yang menunggu deploy testnet.

## M1+ — MVP completion (lanjutan eksplisit, bukan dibuang)

- Sisa fitur MVP lama: offers (009), private listing + bundle (010), factory (012), notifikasi (015),
  report + badge (018, 019), launchpad kontrak + UI (020, 021), stale cleanup (022), admin panel (023),
  branding (008b), error module (033), security P0 (026), infra VPS/CI-CD/backup (028, 029, 030).
- **Prasyarat sebelum bundle (TASK-010) dikerjakan:** temuan C1/C2/C3 dari review desain ronde 17 wajib
  diselesaikan lebih dulu (PARTIAL tanpa alokasi kerugian; gas `buy_bundle` >300 Tgas; dana buyer bisa
  nyangkut bila callback gagal). Lihat [contracts/market.md](../docs/contracts/market.md) §Catatan review.
- Done when: fitur MVP lengkap jalan di testnet + demo penuh (skrip demo lama, §Demo M1).

## M2 — Trading lanjutan (fase 2)

- Auctions (fase 2), multi-currency FT, custom indexer fase awal (ingestion Neardata + floor/volume dasar).
- Done when: auction end-to-end lolos AC + indexer menampilkan stats dasar.

## M3 — Ekosistem

- Lazy minting + indexer penuh (trait filter, rarity, search) + notifikasi real-time dari indexer.
- Done when: pencarian trait berfungsi dari data indexer + notifikasi real-time aktif.

## M4 — Rilis mainnet ⚠️ **hanya bila proyek beralih ke jalur produk**

> **ADR-016:** karena proyek ini pembelajaran/portofolio, M4 (dan seluruh gate-nya) **tidak wajib**.
> Gate di bawah berlaku **hanya bila** user memutuskan mengubah niat proyek menjadi produk nyata.
> Sampai itu terjadi, dokumen **dilarang** menyatakan proyek "aman" atau "siap produksi".

- **Audit eksternal lulus (TASK-027) + temuan kritis ditutup** (bukan sekadar checklist internal).
- **Transfer ownership kontrak ke Sputnik DAO V2 council 2-of-3 + timelock** (ADR-013).
- **Reproducible build terverifikasi (NEP-330)** untuk semua kontrak.
- Monitoring + alerting jalan; drill pause & restore lulus (SEC-IR-001, SEC-DB-004).
- Done when: semua gate di atas lulus → deploy mainnet (keputusan eksplisit user).

> **Tanpa tanggal target** — dipandu kriteria *done when* per milestone; ritme review = demo per milestone.

## Pemetaan task → milestone (lengkap)

> SSOT penugasan milestone per task. Sumber daftar task: [backlog.md](./backlog.md); urutan kerja: [implementation-plan.md](./implementation-plan.md).

| Task | Judul singkat | Milestone penutup | Fase |
|---|---|---|---|
| TASK-001 | Scaffold repo + CI gates | **M0** | Fase 1 |
| TASK-002 | Kontrak NFT core | **M1** ⭐ | Fase 1 |
| TASK-003 | Royalty NEP-199 | **M1** ⭐ | Fase 1 |
| TASK-004 | Market storage + listing 2-tx | **M1** ⭐ | Fase 1 |
| TASK-005 | Buy + settle + refund | **M1** ⭐ | Fase 1 |
| TASK-006 | Sandbox tests 2-kontrak | **M1** ⭐ | Fase 1 |
| TASK-007 | FE connect wallet | **M1** ⭐ | Fase 2 |
| TASK-008 | FE browse/list/buy | **M1** ⭐ | Fase 2 |
| TASK-008b | Sesi branding → design system | **M1+** | Fase 2 (prasyarat) |
| TASK-009 | Offers | **M1+** | Fase 1 |
| TASK-010 | Private listing + bundle ⚠️ butuh C1/C2/C3 selesai | **M1+** | Fase 1 |
| TASK-011 | Multi-currency FT | **M2** | Fase 3 |
| TASK-012 | Factory koleksi | **M1+** | Fase 1 |
| TASK-013 | Lazy minting | **M3** | Fase 4 |
| TASK-014 | Custom indexer (Neardata) | **M2** (lanjut M3) | Fase 3–4 |
| TASK-015 | Notifikasi in-app minimal | **M1+** | Fase 2 |
| TASK-016 | Meta-transactions | **M3** | Fase 4 |
| TASK-017 | Auctions | **M2** | Fase 3 |
| TASK-018 | Report system | **M1+** | Fase 2 |
| TASK-019 | Badge verified | **M1+** | Fase 2 |
| TASK-020 | Launchpad kontrak | **M1+** | Fase 1 |
| TASK-021 | Launchpad UI | **M1+** | Fase 2 |
| TASK-022 | Stale listing | **M1+** | Fase 1 (UI Fase 2) |
| TASK-023 | Admin panel | **M1+** | Fase 2 |
| TASK-024 | Onboarding guide | **M3** | Fase 4 |
| TASK-025 | Leaderboard koleksi | **M3** | Fase 4 |
| TASK-026 | Implement SEC-* P0 | **M1+** (berlanjut) | Fase 1–2 |
| TASK-027 | Audit eksternal | **M4** ⚠️ hanya jalur produk | Fase 5 |
| TASK-028 | Provisioning VPS + Compose | **M1+** | Fase 2 |
| TASK-029 | CI/CD deploy + domain/SSL | **M1+** | Fase 2 |
| TASK-030 | Backup + restore drill + monitoring | **M1+** | Fase 2 |
| TASK-031 | Git/repo hygiene (3 branch) | **M0** | Fase 1 |
| TASK-032 | Versioning & rilis | **M0** | Fase 1 |
| TASK-033 | Error & notifikasi terpusat | **M1+** | Fase 2 |
| TASK-034 | Prep scaling | **M2** | Fase 3 |

- ⭐ **M1 = vertical slice** (8 task) — milestone pertama yang wajib lulus. Lihat [ADR-016](../docs/decisions/ADR-016-project-intent-and-scope-cut.md).
- ⚠️ **M4 = hanya bila proyek beralih ke jalur produk.** Karena proyek ini pembelajaran/portofolio,
  gate audit/DAO/mainnet **tidak wajib** (ADR-016).
- Setiap task punya **tepat satu** milestone penutup; task yang berlanjut (mis. TASK-014 ke M3) ditulis "lanjut" dan tetap dihitung selesai di milestone penutup keduanya.
- Milestone tidak boleh dianggap lulus bila ada task `P0` yang belum `done` (lihat definisi *done when* masing-masing).

## Definisi "audit lulus" (eksplisit)

> Dipakai untuk gate M4 (TASK-027). "Audit lulus" **bukan** "checklist internal hijau" — harus ada auditor pihak ketiga dan temuan yang ditutup.

| Kriteria | Ambang |
|---|---|
| Auditor | Pihak ketiga independen (kandidat: OtterSec / Halborn / Block Security — G8), lingkup = kontrak market + launchpad (+ NFT/factory bila termasuk) |
| Laporan | Laporan tertulis diterima, dengan versi commit/kode yang diaudit **identik** dengan commit yang akan di-deploy (tag) |
| Temuan **Critical** | **0 terbuka** — semua diperbaiki dan diverifikasi ulang auditor |
| Temuan **High** | **0 terbuka** — diperbaiki; bila ada penerimaan risiko, wajib keputusan eksplisit user + catatan ADR |
| Temuan **Medium** | Ditangani: diperbaiki **atau** diterima dengan mitigasi terdokumentasi (keputusan user) |
| Temuan **Low/Info** | Ditriase; perbaikan opsional, dicatat di backlog bila ditunda |
| Retest | Auditor mengonfirmasi perbaikan (retest) atas temuan Critical/High |
| Bukti | Laporan akhir + commit/tag + catatan penutupan temuan disimpan di repo/`docs/` (tanpa data sensitif) |
| Cakupan uji | Minimal: INV-001..030, jalur payout/refund, launchpad, bundle, akses owner/pause |
| Reproducible build | Hash artifact == metadata on-chain (NEP-330) untuk kode yang diaudit (SEC-CONTRACT-006) |

- Selama ada temuan **Critical/High terbuka** → M4 **tidak boleh** lulus, apa pun status task lain.
- Audit yang mengubah kode → perubahan itu mengulang verifikasi (retest) atas bagian terkait; tidak boleh deploy kode yang berbeda dari yang diaudit tanpa persetujuan auditor.
- Keputusan deploy mainnet setelah audit lulus tetap milik **user** (AGENTS.md).

## Risiko & rollback per milestone

| Milestone | Risiko utama | Mitigasi | Rollback |
|---|---|---|---|
| **M0** | Scaffold salah struktur → semua task berikutnya terhambat | Review struktur sebelum koding; CI menyala sejak awal | Revert PR scaffold (belum ada deploy) |
| **M1** | Bug kontrak menyentuh dana; deploy testnet gagal | Sandbox tests + INV P0; testnet = tanpa dana nyata | Kontrak: redeploy wasm versi sebelumnya; Web: deploy tag sebelumnya |
| **M2** | Indexer tidak konsisten; auction/fitur baru melanggar INV | Final-only ingest + dedup; property test | Hentikan indexer + rebuild dari `events_raw`; revert fitur (merge commit) |
| **M3** | Fitur ekosistem (lazy mint/rarity) menambah permukaan serang | Review keamanan per fitur + test | Revert merge commit fitur; matikan endpoint/fitur via flag bila ada |
| **M4** | Deploy mainnet dengan bug/audit belum tuntas; owner key | Gate audit (di atas) + DAO 2-of-3 + timelock + drill pause/restore | Kontrak: redeploy versi terverifikasi + migrate; Web: rollback tag; break-glass pause (guardian) |

- Rollback kontrak **tidak** menghapus state; bila storage layout berubah → migrate yang sesuai ([deployment.md](../docs/deployment/deployment.md)).
- Setiap rollback dicatat di [CHANGELOG.md](../CHANGELOG.md) + [incident-response.md](../docs/security/incident-response.md) bila berdampak user.
- Drill rollback/pause wajib sebelum M4 (SEC-IR-001) — bukti drill = syarat lulus.

## Demo script per milestone

> Demo = bukti *done when*. Dijalankan di **testnet** (kecuali M0), di depan user, dengan akun seed kanonik.

**M0 — Fondasi**
```text
1. Tunjukkan repo: struktur contract/market/factory/frontend/indexer + docs/tasks.
2. Push commit dummy ke `dev` → CI + Security hijau (tunjukkan tab Actions).
3. Tunjukkan branch protection aktif di 3 branch + tag protection.
4. Tunjukkan `cargo test` & `pnpm build` jalan lokal (skema/placeholder).
```

**M1 — Vertical slice (mint → list → buy)** ⭐
```text
1. Dua akun testnet (seller/buyer) connect wallet via UI.
2. Seller: mint 1 NFT (koleksi di-deploy manual, satu fase publik) → list (2 langkah: approve + list).
3. Tunjukkan NFT TETAP di wallet seller selama listing (bukan dipegang kontrak) — bukti non-custodial.
4. Buyer: buy → NFT pindah, seller + royalti dibayar, fee 2% terlihat di breakdown.
5. Tunjukkan `cargo test` hijau (suite slice) + race 20 pembeli → 1 menang, 19 refund.
```

**M1+ — MVP completion** (dijalankan setelah M1 lulus)
```text
1. Offers: buyer make offer → seller accept offer (escrow + refund).
2. Bundle: seller buat bundle 3 item → buyer beli (atau tunjukkan pre-validasi menolak item invalid).
3. Launchpad: creator buat koleksi via wizard → upload allowlist → mint fase allowlist + fase public.
4. Report: user kirim report → muncul di antrean /admin; admin hide target (step-up signature).
5. Tunjukkan CI hijau + deploy otomatis staging + /api/health 200.
```

**M2 — Trading lanjutan**
```text
1. Auction: seller buat auction → dua bidder bid → end → claim (refund bidder kalah).
2. Multi-currency: settlement dengan FT selain NEAR.
3. Indexer: buka discovery (floor/volume) dari data indexer; lag indicator = 0.
```

**M3 — Ekosistem**
```text
1. Lazy mint: mint terjadi saat buy (belum ada on-chain sebelumnya).
2. Trait filter + rarity + search dari data indexer.
3. Notifikasi real-time dari indexer (bukan polling NearBlocks).
```

**M4 — Mainnet**
```text
1. Tunjukkan laporan audit + tabel temuan (semua Critical/High tertutup).
2. Tunjukkan owner kontrak = Sputnik DAO 2-of-3 (view policy) + timelock.
3. Tunjukkan reproducible build: hash artifact == contract_source_metadata on-chain.
4. Tunjukkan monitoring + alert Telegram hidup; drill pause & restore lulus.
5. Deploy mainnet atas keputusan eksplisit user → smoke test produksi.
```

## Metrik keberhasilan per milestone

| Milestone | Metrik | Ambang |
|---|---|---|
| **M0** | CI hijau di `dev`; semua dokumen inti ada & 0 tautan rusak | 100% |
| **M1** | INV-001..030 punya ≥1 test; method mutasi punya test sukses+gagal; jalur emas E2E lulus | 100% (WAJIB) |
| **M1** | Alur list→buy & offer→accept end-to-end di UI testnet | lulus tanpa intervensi manual |
| **M1** | Smoke test pasca-deploy (`/api/health`, halaman utama, header keamanan) | 100% hijau |
| **M2** | Indexer lag = 0 pada jam sibuk uji; floor/volume akurat vs sample rekonsiliasi | lag 0; selisih 0 |
| **M2** | AC auction + multi-currency | 100% lulus |
| **M3** | Pencarian trait/rarity dari indexer; notifikasi real-time | lulus; latensi notifikasi ≤ 10 dtk |
| **M3** | Coverage kontrak ≥ 90% line (PROPOSED) | terpenuhi atau disetujui deviasi |
| **M4** | Temuan audit Critical/High terbuka | **0** |
| **M4** | Owner = DAO 2-of-3; drill pause/restore | lulus + bukti |
| **M4** | Reproducible build terverifikasi untuk semua kontrak | hash cocok 100% |
| **M4** | RPO ≤ 24 jam / RTO < 4 jam terbukti lewat drill | lulus |

- Metrik `WAJIB` = gate; metrik `PROPOSED` = target yang boleh disetujui deviasinya lewat keputusan user.
- Metrik diukur pada **akhir** milestone; hasil dicatat di [00-project-overview.md](../docs/00-project-overview.md) § Status.
