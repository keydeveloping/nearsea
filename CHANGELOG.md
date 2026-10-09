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

### Web (frontend Next.js)
### Added
- Scaffold app + halaman placeholder + modul i18n — **belum ada wallet/marketplace UI** (TASK-007/008).

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
