# Changelog

Semua perubahan penting pada proyek ini didokumentasikan di file ini.

Format mengikuti [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
dan proyek ini memakai **SemVer per-artefak** dengan tag berprefiks:
`contract-vX.Y.Z`, `web-vX.Y.Z`, `indexer-vX.Y.Z`.
Kebijakan lengkap: [docs/development/versioning-and-release.md](./docs/development/versioning-and-release.md).

> Status: **pra-rilis** — belum ada artefak ter-deploy. Entri pertama (scaffold TASK-001)
> ada di bawah; **belum ada versi rilis** (`contract-v*`/`web-v*`) karena belum ada artefak
> yang di-deploy atau di-tag.

---

## [Unreleased]

### Repo / Infra (scaffold — TASK-001, 2026-10-07)
### Added
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

### Contract (kontrak NFT + market + factory)
### Added
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
