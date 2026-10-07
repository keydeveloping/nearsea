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
  Prettier, Vitest 3 + jsdom, `i18n/` bertipe (`common.appName`, dst.).
- CI nyata: `ci.yml` (fmt/clippy/test + build wasm + lint/format/typecheck/test/build) dan `security.yml`
  (gitleaks + `cargo audit` + `pnpm audit` + dependency review) — semua action di-pin commit SHA,
  `cargo-near` 0.22.0 di-pin sha256.
- `.github/CODEOWNERS`, `.github/dependabot.yml` (PR dependency → `dev`).
### Changed
- `rust-toolchain.toml`: Rust **1.77.1 → 1.93.1**; `.nvmrc`: Node **20 → 24**.
- `deploy-{dev,testnet,mainnet}.yml`: auto-trigger → **manual-only + guard variabel environment**
  (auto-deploy menyusul di TASK-029; environment belum di-provision — TASK-028).
- Glob artifact CI dikoreksi: `target/near/*.wasm` → **`target/near/*/*.wasm`** (cargo-near menaruh hasil
  di sub-folder per crate).
- `.gitignore`: tambah `artifacts/`, `.cargo-near/`, `next-env.d.ts`.
### Security
- Secret scan (gitleaks) + audit dependensi aktif di CI (SEC-CICD-001/002); semua action pihak ketiga
  di-pin SHA (cicd-security.md §5).

### Contract (kontrak NFT + market + factory)
### Added
- Placeholder crate (init + owner/pause) — **belum ada perilaku kontrak**; surface NEP + listing/buy
  diimplementasikan di TASK-002..005.

### Web (frontend Next.js)
### Added
- Scaffold app + halaman placeholder + modul i18n — **belum ada wallet/marketplace UI** (TASK-007/008).

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
