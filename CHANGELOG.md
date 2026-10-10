# Changelog

Semua perubahan penting pada proyek ini didokumentasikan di file ini.

Format mengikuti [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
dan proyek ini memakai **SemVer per-artefak** dengan tag berprefiks:
`contract-vX.Y.Z`, `web-vX.Y.Z`, `indexer-vX.Y.Z`.
Kebijakan lengkap: [docs/development/versioning-and-release.md](./docs/development/versioning-and-release.md).

> Status: **pra-rilis** — belum ada artefak ter-deploy. Entri pertama (scaffold TASK-001)
> ada di bawah; **belum ada versi rilis** (`contract-v*`/`web-v*`) karena belum ada artefak
> yang di-deploy atau di-tag.
>
> **Rilis pertama yang disiapkan (TASK-032): `contract-v0.1.0`.** Isi rilis = seluruh entri
> `### Contract` di bawah. Sesuai [versioning-and-release.md](./docs/development/versioning-and-release.md) §5,
> heading versi + tanggal dibuat **saat tag di-push** (langkah 7), bukan sekarang — sehingga
> tidak ada heading versi yang menjanjikan rilis yang belum terjadi. Tag dibuat di `mainnet`
> setelah PR promosi `dev → testnet → mainnet` (wajib persetujuan user — [git-workflow.md](./docs/development/git-workflow.md) §3/§12).
> Prosedur rilis & rollback: §Rilis pertama di bawah.

---

## [Unreleased]

### Repo / Infra (scaffold — TASK-001, 2026-10-07)
### Added
- **Slice M1 mendarat di `dev` (2026-10-09)** — seluruh branch bertumpuk yang sebelumnya hanya lokal
  di-push dan di-merge lewat PR berurutan: #11 (TASK-032), #12 (TASK-003), #13 (TASK-004),
  #15 (TASK-007), #16 (TASK-008 + TASK-038). CI hijau di tiap langkah. TASK-005/006 (PR #14) menyusul
  setelah advisory RUSTSEC-2026-0285 diterima + dicatat (lihat Security).
- Cargo workspace root (`Cargo.toml` + `Cargo.lock`) dengan anggota `contract/` (nearsea-nft-collection),
  `market/` (nearsea-market), `factory/` (nearsea-factory) — placeholder init + Owner (+Pause) + unit test.
- Frontend `frontend/` — Next.js 16 App Router, TypeScript strict, Tailwind 4, ESLint 9 (+ aturan proyek),
  Prettier, Vitest 4 + jsdom, `i18n/` bertipe (`common.appName`, dst.).
- CI nyata: `ci.yml` (fmt/clippy/test + build wasm + lint/format/typecheck/test/build) dan `security.yml`
  (gitleaks + `cargo audit` + `pnpm audit` + dependency review) — semua action di-pin commit SHA,
  `cargo-near` 0.22.0 di-pin sha256.
- `.github/CODEOWNERS`, `.github/dependabot.yml` (PR dependency → `dev`).
- Repo remote `github.com/keydeveloping/nearsea` (kini **publik**); 3 branch permanen di-push; default branch
  di-set `mainnet`. PR #1 squash-merge ke `dev` (commit `223015b`) — **CI + Security hijau di `dev`**
  (bukti: run CI 37627825466, Security 37627825490).
- **Proteksi branch aktif** (TASK-031) di `dev`/`testnet`/`mainnet`: PR wajib, force-push & delete diblokir
  **termasuk admin** (`enforce_admins`), 5 required status checks, conversation resolution; `strict` di
  testnet/mainnet. Tag protection lewat ruleset `protect-release-tags`
  (`contract-v*`/`web-v*`/`indexer-v*`: delete + update diblokir).
### Changed
- `rust-toolchain.toml`: Rust **1.77.1 → 1.93.1**; `.nvmrc`: Node **20 → 24**; `vitest` **3 → 4**.
- `deploy-{dev,testnet,mainnet}.yml`: auto-trigger → **manual-only + guard variabel environment**
  (auto-deploy menyusul di TASK-029; environment belum di-provision — TASK-028).
- Glob artifact CI dikoreksi: `target/near/*.wasm` → **`target/near/*/*.wasm`** (cargo-near menaruh hasil
  di sub-folder per crate).
- `.gitignore`: tambah `artifacts/`, `.cargo-near/`, `next-env.d.ts`.
- Script FE `typecheck` kini menjalankan `next typegen` lebih dulu — tipe route Next.js di-generate, jadi
  gate tidak lagi bergantung pada sisa `.next/` dari `next build`.
### Fixed
- `security.yml`: gitleaks butuh scope `pull-requests: read` (sebelumnya 403 "Resource not accessible by
  integration"); `cargo audit` dipasang dengan `--locked` (sebelumnya menarik dependency yang menuntut
  rustc lebih baru daripada toolchain ter-pin); dependency review di-skip terarah selama branch target
  belum punya dependency graph.
### Security
- Secret scan (gitleaks) + audit dependensi aktif di CI (SEC-CICD-002 → READY-FOR-IMPLEMENTATION); semua
  action pihak ketiga di-pin SHA (cicd-security.md §5). **SEC-CICD-001/003 → READY-FOR-IMPLEMENTATION**:
  branch protection aktif setelah repo dijadikan publik (sebelumnya terblokir 403 "butuh GitHub Pro" saat
  private). **Catatan terbuka**: required approval = 0 (ditunda) karena repo masih satu akun — GitHub
  melarang self-approve; dinaikkan ke testnet=1/mainnet=2 saat maintainer kedua ada.
- **Kerentanan diperbaiki**: `tinypool` (critical, prototype pollution → RCE) lewat vitest 3 — naik ke
  vitest 4 yang tidak lagi memakainya. **Satu advisory tanpa patch upstream** (`braces <=3.0.3`, high,
  ReDoS, lewat toolchain `eslint-config-next`) dikecualikan **eksplisit** di
  `frontend/pnpm-workspace.yaml` + dilacak sebagai TASK-035.
- **Advisory RUSTSEC-2026-0285 diterima + dicatat eksplisit (TASK-039, 2026-10-09)** — `rustls 0.23.43`,
  "TLS 1.3 handshake messages incorrectly accepted across encryption level boundaries"; perbaikan
  `>=0.23.45`. Terbit di antara dua run CI hari itu (06:48Z hijau, 06:54Z gagal), jadi bukan akibat
  perubahan proyek. Terjangkau **hanya** lewat dev-dependency `nearsea-market → near-workspaces →
  near-sandbox → ureq`; kontrak produksi tidak menyentuhnya. Perbaikan langsung gagal karena
  `rustls 0.23.45` menuntut `aws-lc-rs ^1.18` sementara `near-crypto` mem-pin `aws-lc-rs = "=1.16.2"`
  exact. Jalan keluar bersih sudah diuji dan tidak ada (`native-tls` tidak cukup; `near-sandbox`/`ureq`
  sudah terbaru; `near-crypto` hanya prerelease). **Keputusan: terima + catat** di
  [`.cargo/audit.toml`](.cargo/audit.toml) dengan alasan + pelacak — bukan di-ignore diam-diam.
  **Tindak lanjut**: hapus entri begitu upstream melonggarkan pin `aws-lc-rs`.

### Contract (kontrak NFT + market + factory)
### Added
- **Suite sandbox dua-kontrak (TASK-006, 2026-10-08)** — `market/tests/slice_sandbox.rs` (**18 test**)
  + harness `market/tests/common/mod.rs` men-deploy wasm **koleksi + market nyata** ke sandbox chain
  (`near-workspaces`), bukan mock, dari `target/near/<crate>/`. Membuktikan tesis slice di rantai
  sungguhan: jalur bahagia penuh dengan angka exact (fee 2% → treasury, royalti 5% → kreator, proceeds
  seller = residual; **NFT terbukti tetap di wallet seller selama listing** — non-custodial diasersi
  on-chain), **race 20 pembeli → tepat 1 menang** dengan 19 deposit kembali penuh (hanya gas hangus),
  stale dua kasus (kepemilikan pindah **dan** approval dicabut) → refund penuh + `market_stale_detected`,
  **payout tidak valid dari koleksi pihak ketiga** → refund penuh + listing dipulihkan, recovery
  pembelian nyangkut permissionless (TC-054), plus TC-001/013/017/020/022/044/047/048 dan
  INV-004/014/027. Crate fixture **TEST** baru `market/tests/fixtures/rogue-collection/` (koleksi
  pihak ketiga "nakal" untuk TC-003) — tidak pernah ikut rilis. `ci.yml` kini **membangun wasm +
  fixture sebelum test**. Suite di-`#![cfg(unix)]` (binary sandbox nearcore tidak dipublikasikan untuk
  Windows) dengan dev-dependency di-scope `[target.'cfg(unix)'.dev-dependencies]` supaya gate lokal
  Windows tidak berubah.
- **Market buy + settlement + refund (TASK-005, 2026-10-08)** — jalur uang `market/src/lib.rs`. `buy`
  menulis `pending_purchases` **sebelum** optimistic removal (INV-031), lalu callback `#[private]`
  `process_purchase` menjalankan dual verification **saat settle** (stale dua kasus INV-016 → refund
  penuh + `market_stale_detected`; verifikasi tak pasti → refund + restore `Sale`), memanggil
  `nft_transfer_payout` (1 yocto, `max_len_payout = 10`), dan callback `#[private]` `resolve_purchase`
  memvalidasi payout **UNTRUSTED** (`1..=10` penerima, `amount > 0`, `Σ ≤ harga−fee`) lalu
  mendistribusikan fee → treasury, royalti → receiver, residual → seller, kelebihan deposit → buyer;
  payout invalid atau promise gagal → refund penuh + restore `Sale` (SEC-ORDER-001). Ditambah
  `recover_stuck_purchase` + `process_recovery` (permissionless setelah `RECOVERY_DELAY_BLOCKS` — setiap
  deposit punya jalur keluar tanpa governance, INV-031), `update_fee_bps`/`update_treasury` (owner-only,
  1 yocto, cap `MAX_FEE_BPS` — INV-004), `fee_bps`/`treasury` di init, view
  `get_fee_bps`/`get_treasury`/`get_pending_purchase`, dan event
  `market_sale`/`market_stale_detected`/`market_purchase_recovered`/`fee_update`/`treasury_update`.
  **Belum termasuk**: `remove_stale_listing` (TASK-022), offers/bundle (TASK-009/010), pengukuran gas
  sandbox + race TC-016/017 (TASK-006).
- **Market listing 2-tx + storage NEP-145 (TASK-004, 2026-10-08)** — `market/src/lib.rs` kini punya jalur
  listing: `list_nft_for_sale` (deposit = storage NEP-145, **bukan** 1 yocto) yang **tidak percaya klaim
  pemanggil** — kontrak mengirim dua view call ke koleksi (`nft_token` → kepemilikan, `nft_is_approved` →
  approval) dan callback `#[private]` `process_listing` yang baru menyimpan `Sale` bila keduanya lolos
  (SEC-ORDER-004, ADR-002). Non-custodial: NFT tetap di wallet seller — jalur listing tidak pernah
  memanggil `nft_transfer*`. Plus `remove_sale` (1 yocto, owner-only, boleh saat paused — INV-022),
  `update_price` (in-place, min harga), view `get_sale`/`get_sales` (paginasi + clamp)/`get_supply_sales`,
  dan event `market_list`/`market_delist`/`market_update_price` (envelope `SingleEvent` sama dengan koleksi).
  Harga min 0.01 Ⓝ (INV-030, batas inklusif), duplikat listing ditolak (INV-007), storage kurang → revert
  (INV-020), `approval_id` di luar rentang `u32` ditolak. Ditambah **`nft_on_approve`** (receiver NEP-178)
  yang wajib ada untuk tx-1 (`nft_approve(market, msg)`) tapi sengaja **tidak** membuat listing (ADR-002),
  dan pause yang ditegakkan **juga di callback** `process_listing` (receipt terpisah — INV-022).
  **Belum termasuk**: `buy`/`resolve_purchase` +
  `pending_purchases`/`recover_stuck_purchase` (TASK-005), `fee_bps`/`treasury` di init (TASK-005),
  offers/bundle (TASK-009/010), `remove_stale_listing` (TASK-022), kalibrasi gas/storage sandbox (TASK-006).
- **Royalti NEP-199 (TASK-003, 2026-10-08)** — `nft_transfer_payout` di `contract/src/lib.rs`:
  memindahkan token ke `receiver_id` (otorisasi lewat approval NEP-178, `assert_one_yocto()`,
  `max_len_payout` dihormati) **dan** mengembalikan payout royalti untuk `balance` dalam panggilan
  yang sama, sesuai NEP-199. Payout = satu penerima (`creator_id`) dengan
  `floor(balance × royalty_bps / 10_000)` (`checked_mul` → overflow = `CHAIN_REVERT`), diturunkan dari
  konfigurasi royalti **level kontrak** — bukan `TokenMetadata.extra`, bukan per-token. Selalu ≤10%
  harga (INV-027); entri ber-amount `0` (basis dust) tetap dikembalikan apa adanya supaya market yang
  memutuskan validasi INV-003 pada payout final. **Belum termasuk**: pengukuran gas 15 Tgas dan
  TC-003 versi sandbox (validasi payout di sisi market + refund) — keduanya milik suite dua-kontrak
  TASK-006.
- **Versioning & rilis (TASK-032, 2026-10-07)** — versi artefak kini **tertanam di build dan
  bisa dibaca dari luar** (SEC-CONTRACT-006, NEP-330):
  - `[package.metadata.near.reproducible_build]` di ketiga crate (`contract/`, `market/`,
    `factory/`) dengan image Docker **ter-pin by digest**
    (`sourcescan/cargo-near:0.21.1-rust-1.96.0`) — build reproducible dijalankan di container,
    sehingga toolchain rilis = isi image, bukan `rust-toolchain.toml` lokal.
  - `repository` di `[package]` → field `link` NEP-330 terisi; `version` → field `version` NEP-330.
  - Workflow baru [`.github/workflows/release.yml`](./.github/workflows/release.yml): dipicu tag
    `contract-v*`/`web-v*`/`indexer-v*`, memeriksa **versi manifest == versi tag**, membangun wasm
    secara reproducible, **membuktikan metadata NEP-330 yang tertanam == tag**, lalu melampirkan
    wasm + `code-hash.txt` ke GitHub Release. Bisa diuji-kering lewat `workflow_dispatch`.
  - `ci.yml` kini membangun **ABI** (bukan lagi `--no-abi`) dan memverifikasi metadata NEP-330
    tiap PR — versi tidak bisa lagi basi tanpa CI memerah.
- **NFT collection core (TASK-002, 2026-10-07)** — `contract/src/lib.rs` kini punya perilaku nyata:
  NEP-171 core (`nft_transfer`/`nft_transfer_call`/`nft_resolve_transfer`/`nft_token`), NEP-177 metadata
  (kontrak + per-token), NEP-178 approval (`nft_approve`/`nft_revoke`/`nft_revoke_all`/`nft_is_approved`),
  NEP-181 enumerasi, NEP-145 storage — semuanya lewat derive `NonFungibleToken` (`near-sdk-contract-tools` 4.0).
  Ekstensi NearSea: `nft_mint` **launchpad-aware** (deposit exact-match `harga fase × quantity`, alokasi
  fase, `max_per_wallet`, allowlist opsional, storage dibayar minter), `set_phases` (replace-all, validasi
  INV-029), `allowlist_add`, `get_launchpad`, `allowlist_contains`, `royalty_config`, `market_id`.
  Event NEP-297 untuk mint/transfer + event NearSea `launchpad_mint` (`data` berbentuk **array** sesuai
  [webhooks.md](./docs/api/webhooks.md); helper `SingleEvent<T>` dipakai karena derive memancarkan objek).
  Init `new` menolak `royalty_bps` di luar `1..=1000` (`INVALID_ROYALTY`) dan memakai `PanicOnDefault`
  (SEC-CONTRACT-002). Batas window fase `[starts_at, ends_at)` ditetapkan DECIDED. **Belum termasuk**:
  `nft_transfer_payout` NEP-199 (TASK-003), fase bebas penuh + Pausable (TASK-020).
- Placeholder crate (init + owner/pause) — surface market/factory menyusul di TASK-004/005/012.

### Changed
- **Fee dibayar langsung ke treasury di settlement (TASK-005, 2026-10-08)** — `withdraw_fees` dan event
  `treasury_withdraw` **dihapus**; fee tidak pernah tertahan di kontrak, sehingga tidak ada akuntansi
  solvensi fee-vs-escrow-vs-storage yang perlu dijaga (temuan M7 tertutup) dan tidak ada honeypot saldo
  fee. Owner-only config kini `update_fee_bps`/`update_treasury`
  ([contracts/market.md](./docs/contracts/market.md) §4a; TC-047 dialihkan).
- **Aturan `sisa ≤ 1 yocto` (INV-002) dibatalkan (TASK-005, 2026-10-08)** — koleksi mengembalikan **hanya
  royalti** dan market menambahkan seller sebagai residual, sehingga `harga − fee − Σpayout` = **proceeds
  seller** (wajar besar, mis. 93% harga saat royalti 5%), bukan dust. Yang divalidasi tetap
  `Σpayout ≤ harga − fee`, `amount > 0`, `1 ≤ len ≤ 10`. Aturan lama berasal dari model tutorial
  (RESEARCH.md §10.5) yang tidak dipakai NearSea.

### Fixed
- **Temuan F1 dari suite sandbox (TASK-006, 2026-10-08)** — `nft_transfer` atas token yang **masih
  di-approve** gagal `Storage accounting error: … cannot unlock more tokens than it has deposited`
  (`ExcessiveUnlockError`) bila penerima belum memegang token lain di koleksi yang sama. Diduga hook
  NEP-145 membebaskan storage entry approval ke **receiver**, padahal entry itu ditagih ke **owner**.
  **Belum diperbaiki** — dicatat sebagai **TASK-036** (P0, wajib selesai sebelum M1 ditutup); jalur uang
  NearSea (`list` → `buy` via NEP-199) tidak terkena, dan TC-006 memakai urutan `nft_revoke` →
  `nft_transfer` sebagai jalan keluar sementara.
- **Koreksi spesifikasi `nft_revoke_token` (TASK-004, 2026-10-08)** — 5 dokumen menyebut `remove_sale`
  memanggil `nft_revoke_token` "sebagai approved account". **Method itu tidak ada di NEP-178**: standar
  hanya punya `nft_revoke`/`nft_revoke_all`, keduanya **owner-only** ("MUST panic if called by someone
  other than token owner"), tanpa varian untuk approved account. Kontrak market karena itu **tidak**
  mencabut approval saat cancel listing; aman karena tanpa entry `Sale` market tidak punya jalur
  memindahkan token, dan transfer oleh owner otomatis mencabut semua approval (FACT NEP-178). Re-list =
  seller `nft_revoke` dulu, baru `nft_approve` baru. Dirujuk di
  [contracts/market.md](./docs/contracts/market.md) §2a; `features/marketplace.md`,
  `security/smart-contract-security-architecture.md`, `security/order-protocol-security.md`,
  `security/signature-architecture.md`, `02-product-requirements.md`, `00-project-overview.md`, dan
  TC-044 disinkronkan.

### Web (frontend Next.js)
### Added
- Scaffold app + halaman placeholder + modul i18n — **belum ada marketplace UI** (TASK-008).
- **Connect wallet (TASK-007, 2026-10-08)** — `frontend/features/auth/`: state wallet app-wide
  (`WalletContext` + `useWallet()`, satu tipe `WalletApi`) di atas `@hot-labs/near-connect` 0.11.4 +
  `near-connect-hooks` 1.1.6 (`NearProvider`/`useNearWallet`); header app dengan indikator jaringan
  permanen + kontrol wallet (connect / connecting / connected(alamat+saldo) / disconnecting /
  error+Retry / disconnect); banner peringatan jaringan + flag gerbang `transactionsDisabled`; mode
  baca tanpa wallet (halaman tetap ter-prerender statis). Modul pendukung: `lib/near/network.ts`
  (konfigurasi jaringan dari env — nilai tak dikenal gagal saat start), `lib/near/wallet-connector.ts`
  (konfigurasi connector; **tanpa daftar wallet hardcoded** — daftar dari manifest resmi near-connect),
  `lib/near/wallet-errors.ts` (klasifikasi error wallet → kode lokal), `lib/format/money.ts`
  (`formatNear`/`YoctoNear` — aritmetika BigInt, tanpa float). Copy UI lewat `i18n/en/auth.json`.
- **Tes komponen (RTL)** — `@testing-library/react` + `@testing-library/dom` (devDependency) +
  `vitest.setup.ts` (cleanup RTL); suite FE **52 test**.
- **Marketplace UI: browse / list / buy (TASK-008, 2026-10-09)** — `frontend/features/marketplace/`:
  grid Explore (`ListingGrid`/`ListingCard` + `useListings`) membaca `get_sales` langsung lewat view
  call RPC; halaman `/token/[contract]/[tokenId]` (`TokenPage` + `useTokenDetail`) menampilkan
  metadata, harga, status listing, dan aksi sesuai peran (`TokenActions`: owner → Sell, non-owner →
  Buy); modal listing **dua langkah tanda tangan** (`nft_approve` "Step 1 of 2" → `list_nft_for_sale`
  "Step 2 of 2", `approval_id` konkret dari `nft_token`, deposit storage NEP-145 ditampilkan sebelum
  signing); modal beli dengan **breakdown fee + royalti** sebelum konfirmasi dan **re-verify sebelum
  signing** (SEC-ORDER-003). Modul pendukung: `lib/format/fees.ts` (breakdown BigInt, `percentFromBps`),
  `lib/format/storage.ts` (kekurangan deposit NEP-145), `lib/format/media.ts` (media hanya `https:`),
  `lib/errors/market-errors.ts` (panic kontrak → kode registry), `lib/near/contracts.ts` (alamat market
  dari env), `lib/constants/near-gas.ts`, `components/ui/Modal.tsx` (focus trap + fokus kembali ke
  pemicu). Namespace i18n baru `marketplace` + `errors` (katalog penuh `error-handling.md` §8).
  `WalletApi` diperluas dengan `viewFunction`/`callFunction`; wallet disuntikkan ke fitur lewat
  interface sempit `MarketChain` (`app/MarketChainProvider.tsx`) agar fitur tidak saling impor.
- **Tes marketplace** — suite FE **127 test** (16 file), termasuk 20 test jalur emas
  browse→list→buy terhadap chain palsu.
### Changed
- `frontend/tsconfig.json`: `target` **ES2017 → ES2020** (literal `BigInt` wajib untuk aritmetika
  yoctoNEAR).
- `frontend/i18n/index.ts`: lookup kunci diperluas ke **kedalaman bebas** (`auth.error.rejected`),
  sebelumnya hanya dua segmen — konvensi `code-standards.md` §10 memakai tiga segmen; namespace
  `errors` didaftarkan dari isi `errors.json` (bukan pembungkusnya) supaya kuncinya tetap
  `errors.<CODE>`.
- `frontend/package.json`: `@hot-labs/near-connect` dan `near-connect-hooks` di-pin **exact**
  (`frontend-security.md` §9 — dependensi kritis); `near-api-js` dihapus dari `dependencies`
  (dipakai sebagai dependency transitif, tidak pernah diimpor langsung); `@tanstack/react-query`
  5.104.1 ditambahkan (server state — `frontend-architecture.md` §Data fetching).
- `.env.example` + `docs/deployment/environments.md` + `docs/architecture/frontend-architecture.md` §7:
  frontend membaca `NEXT_PUBLIC_NEAR_NETWORK`/`NEXT_PUBLIC_NEAR_RPC_URL`/`NEXT_PUBLIC_NEAR_RPC_FALLBACKS`
  (Next.js hanya mengekspos prefix `NEXT_PUBLIC_`) dan `NEXT_PUBLIC_MARKET_CONTRACT_ID`; template memuat
  keduanya agar tidak ada nilai berbeda antara proses server dan browser.
- **Lint FE diperketat (TASK-038, 2026-10-09)** — `frontend/eslint.config.mjs` kini juga mengaktifkan
  `import/order` (grup `code-standards.md` §5: `type` di akhir, satu baris kosong antar grup, alfabetis;
  plus `settings["import/internal-regex"] = "^@/"` supaya alias repo tidak terbaca sebagai paket
  eksternal), `react/jsx-no-useless-fragment`, dan `no-restricted-imports` per-`files` yang menegakkan
  tabel batas dependensi `frontend-architecture.md` §1. **Tanpa dependency baru** — `eslint-config-next`
  sudah membawa `eslint-plugin-import` + `eslint-plugin-react`; dugaan lama "butuh plugin tambahan" tidak
  benar. Konsekuensinya 30 file dirapikan **urutan import-nya saja** (auto-fix, tanpa perubahan perilaku).
### Security
- Tidak ada private key/seed/kredensial di source maupun bundle FE (diverifikasi: `grep` `.next/static`
  + `.next/server` bersih). **Koreksi (ronde 26)**: klaim bahwa kode `function-call-key-plugin`
  ter-tree-shake dari bundle **tidak benar** — `createLocalKeyFor`/`access_key::plugin`/`ed25519:` tetap
  ada di chunk klien, diverifikasi pada build `c7e081b` **dan** build sesudahnya. Jalurnya tetap tidak
  aktif (tidak ada pemanggil `addFunctionCallKey`), tetapi **TASK-037** tidak boleh ditutup dengan
  alasan tree-shaking.
- Media metadata (konten pihak ketiga) hanya dirender lewat `<img>` dan hanya untuk URL **https:**
  (`lib/format/media.ts`); `data:`/`javascript:`/`blob:` ditolak. Tidak ada `dangerouslySetInnerHTML`
  di repo; judul/deskripsi dirender sebagai teks React.

### Indexer (fase 2 — Neardata)
### Added
- (belum ada — menunggu Fase 3)

---

## Rilis pertama — prosedur & rollback

> Ditulis di TASK-032. Ringkas saja; aturan lengkapnya milik
> [versioning-and-release.md](./docs/development/versioning-and-release.md) §5/§12/§14,
> [git-workflow.md](./docs/development/git-workflow.md) §3/§12/§15, dan
> [ci-cd.md](./docs/development/ci-cd.md) §5/§14/§16.

**Artefak pertama:** `contract-v0.1.0` (isi = seluruh entri `### Contract` di `Unreleased`).

```text
1. PR feat/* → dev → CI + Security hijau → squash merge.
2. PR dev → testnet   ← WAJIB tanya user dulu (git-workflow §3).
3. PR testnet → mainnet ← WAJIB tanya user dulu.
4. Di mainnet, pada commit hasil merge:
     git tag -a contract-v0.1.0 -m "contract-v0.1.0 — rilis pertama (TASK-032)"
     git push origin contract-v0.1.0
5. Workflow release.yml jalan otomatis pada tag:
     - versi manifest (contract/market/factory Cargo.toml) harus == 0.1.0;
     - wasm di-build reproducible di container ter-pin;
     - metadata NEP-330 tertanam harus memuat version=0.1.0 + link repo;
     - wasm + code-hash.txt dilampirkan ke GitHub Release.
6. Pindahkan entri `Unreleased` ke heading `## [contract-v0.1.0] - YYYY-MM-DD` (UTC)
   di commit dokumentasi terpisah (versioning §5 langkah 7).
```

**Rollback** (versioning §12, ci-cd §16, git-workflow §15):

| Lapis | Aksi | Catatan |
|---|---|---|
| Branch target | `git revert -m 1 <merge-commit-promosi>` di `mainnet`, lalu promosikan ulang lewat alur normal | revert merge commit = cara standar ([git-workflow.md](./docs/development/git-workflow.md) §15) |
| Kontrak ter-deploy | redeploy wasm dari tag rilis sebelumnya ke akun yang sama (state persist) | bila storage layout berubah → jalankan migrate yang sesuai |
| Tag | **jangan** ubah/hapus tag yang sudah di-push (diblokir ruleset `protect-release-tags`); perbaikan = rilis PATCH baru | [versioning-and-release.md](./docs/development/versioning-and-release.md) §11 |
| Catatan | setiap rollback ditulis di CHANGELOG bagian artefak terkait — entri rilis yang di-rollback **tidak dihapus** | versioning §8 |

- Kontrak **belum di-deploy** ke testnet/mainnet (butuh persetujuan user — git-workflow §3), jadi
  baris "Kontrak ter-deploy" baru relevan setelah deploy pertama.

---

<!--
## [contract-v0.1.0] - YYYY-MM-DD
### Added
### Changed
### Fixed
### Security

## [web-v0.1.0] - YYYY-MM-DD
...
-->

<!-- Jenis perubahan: Added / Changed / Deprecated / Removed / Fixed / Security -->
