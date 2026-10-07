# Versioning & Release

> Skema versi NearSea: **SemVer per-artefak** dengan tag git berprefiks. Versi harus
> jelas, terlihat, dan dapat ditelusuri sampai ke build on-chain.
> Terkait: [git-workflow.md](./git-workflow.md), [ci-cd.md](./ci-cd.md),
> [CHANGELOG.md](../../CHANGELOG.md), [NEP-330](https://nomicon.io/Standards/ContractMetadata/ContractMetadata).

## 1. Skema: SemVer per-artefak

Tiga artefak punya siklus rilis sendiri (kontrak & FE sering tidak rilis bersamaan):

| Artefak | Tag | Contoh | Diletakkan di |
|---|---|---|---|
| Kontrak (NFT + market + factory) | `contract-vX.Y.Z` | `contract-v0.1.0` | `contract/Cargo.toml` version + metadata NEP-330 on-chain |
| Frontend (web) | `web-vX.Y.Z` | `web-v0.1.0` | `frontend/package.json` version |
| Indexer (fase 2) | `indexer-vX.Y.Z` | `indexer-v0.1.0` | `indexer/package.json` version |

Aturan SemVer (`MAJOR.MINOR.PATCH`):

- **MAJOR** — perubahan yang tidak kompatibel:
  - Kontrak: perubahan **storage layout** yang butuh migrate, atau perubahan
    argumen method (perubahan API on-chain) yang memutus klien lama.
  - Web/indexer: breaking change API publik.
- **MINOR** — fitur baru yang kompatibel (method baru, halaman baru, endpoint baru).
- **PATCH** — perbaikan bug/keamanan tanpa perubahan API.

Selama pra-1.0 (`0.y.z`), MINOR boleh memuat breaking change — tetap dicatat di CHANGELOG.

## 2. Versi kontrak harus bisa diverifikasi on-chain (NEP-330)

- Setiap rilis kontrak **wajib** menerbitkan `contract_source_metadata` (NEP-330) berisi
  versi + sumber + hash build (SEC-CONTRACT-006).
- Alur verifikasi: hash artifact CI → bandingkan dengan metadata on-chain
  (`near view <contract> contract_source_metadata`) → cocok = build terbukti reproducible.
- Deploy tanpa verifikasi hash = **tidak sah** (lihat [ci-cd.md](./ci-cd.md) §5).

## 3. Versi storage layout (SEC-CONTRACT-008)

- Perubahan struktur state → dokumentasikan layout versi di setiap rilis
  (lihat [smart-contract-security-architecture.md](../security/smart-contract-security-architecture.md) §2).
- **Persistent prefix stabil** — jangan ubah key prefix collection yang sudah terisi.
- Migrasi state pakai `#[init(ignore_state)]` + fungsi migrate; dicatat di CHANGELOG bagian kontrak.

## 4. Versi event & API

- Event NEP-297 memakai `standard: "x-nearsea-market"`, `version: "1.0.0"` — versi ini
  naik bila **skema payload event berubah** (bukan versi kontrak).
- API report memakai prefix versi pada path (`/api/...`) bila ada breaking change.

## 5. Siklus rilis

1. Kumpulkan perubahan di `Unreleased` pada [CHANGELOG.md](../../CHANGELOG.md).
2. Tentukan bump (MAJOR/MINOR/PATCH) per artefak yang berubah.
3. Update nomor versi di manifest artefak (`Cargo.toml` / `package.json`).
4. PR promosi `dev → testnet → mainnet` (**wajib tanya user**, lihat git-workflow §3).
5. Setelah merge ke branch target: buat tag `git tag contract-vX.Y.Z` + push tag.
6. Deploy lewat workflow manual; verifikasi hash (kontrak) / smoke test (web).
7. Pindahkan entri CHANGELOG dari `Unreleased` ke versi + tanggal rilis.

## 6. Checklist rilis (per artefak)

**Kontrak**
- [ ] `cargo fmt` + `clippy` bersih, semua test sandbox hijau.
- [ ] Storage layout terdokumentasi (jika berubah) + migrate disiapkan.
- [ ] `contract_source_metadata` NEP-330 diterbitkan & hash terverifikasi.
- [ ] Requirement SEC-* P0 relevan terpenuhi.
- [ ] Audit eksternal lulus — **khusus mainnet** (TASK-027).

**Web / Indexer**
- [ ] Lint + typecheck + test + build hijau.
- [ ] Smoke test endpoint `/api/health` lulus setelah deploy.
- [ ] Variabel environment baru tercatat di [environments.md](../deployment/environments.md).

## 7. Kebijakan deprecation

- Fitur/endpoint yang akan dihapus → tandai **DEPRECATED** minimal 1 MINOR sebelum dihapus.
- Catat di CHANGELOG (`Deprecated`) + beri jalur pengganti.
- Penghapusan = MAJOR (untuk API publik kontrak/FE).

## 8. Format entri CHANGELOG (contoh konkret)

Format mengikuti [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) + bagian per-artefak (lihat [CHANGELOG.md](../../CHANGELOG.md)). Setiap entri menyebut **nomor task** dan **kode SEC-/INV-** yang relevan bila ada.

```markdown
## [contract-v0.2.0] - 2026-10-20

### Added
- `create_bundle` + `buy_bundle` (maks 10 item, pre-validasi semua item) (TASK-010, INV-025).
- Event `market_bundle_partial` untuk kegagalan residual mid-loop.

### Changed
- `buy` menolak harga < 0.01 Ⓝ (INV-030) — aturan baru, bukan perubahan argumen (non-breaking untuk klien).

### Fixed
- `resolve_purchase` mengembalikan entry sale bila payout gagal (INV-001, TC-003).

### Security
- `assert_one_yocto` ditambahkan pada `update_price` (SEC-CONTRACT-001).

### Deprecated
- `legacy_buy(contract_id, token_id)` → gunakan `buy(sale_id)`; dihapus di `contract-v0.3.0`.
```

Aturan penulisan entri:

- Satu entri = satu versi artefak + tanggal rilis (`YYYY-MM-DD`, UTC).
- Bagian yang dipakai: `Added` / `Changed` / `Deprecated` / `Removed` / `Fixed` / `Security` (tidak perlu menulis bagian kosong).
- Selama pengembangan, perubahan masuk ke `## [Unreleased]` di bawah sub-bagian artefak (`### Contract` / `### Web` / `### Indexer`); saat rilis, entri dipindah ke versi + tanggal.
- **Rollback** dicatat sebagai entri `### Changed`/`### Fixed` dengan kalimat eksplisit ("rollback ke `contract-v0.2.0` karena …") — jangan menghapus entri rilis yang di-rollback.
- Jangan mencatat perubahan internal murni (refactor tanpa dampak) kecuali memengaruhi perilaku/API.

## 9. Matriks keputusan bump

| Perubahan | Kontrak | Web | Indexer |
|---|---|---|---|
| Bugfix tanpa perubahan API | PATCH | PATCH | PATCH |
| Perbaikan keamanan tanpa perubahan API | PATCH | PATCH | PATCH |
| Method/endpoint/halaman **baru** (kompatibel) | MINOR | MINOR | MINOR |
| Field baru opsional di respons API | — | MINOR | MINOR |
| Tambah `enum` value baru (aditif) | — | MINOR | MINOR |
| Argumen method kontrak berubah / method dihapus | **MAJOR** | MAJOR | MAJOR |
| Storage layout berubah (butuh migrate) | **MAJOR** | — | — |
| Breaking change API publik FE (`/v2`) | — | **MAJOR** | MAJOR |
| Perubahan dependency mayor (perilaku) | MINOR/PATCH | MINOR/PATCH | MINOR/PATCH |
| Perubahan hanya dokumen/komentar | — (tanpa tag) | — | — |

- **Pra-1.0** (`0.y.z`): MINOR boleh memuat breaking change, tetapi tetap dicatat eksplisit di CHANGELOG.
- Bila ragu antara PATCH dan MINOR → pilih **MINOR** (lebih aman: memberi ruang bagi konsumen untuk memperhatikan).
- Satu rilis boleh memuat bump untuk beberapa artefak sekaligus (tag terpisah per artefak).
- Perubahan yang **hanya** menyentuh CI/docs tidak menaikkan versi artefak mana pun.

## 10. Pre-release (`-rc`) handling

Pra-rilis dipakai untuk kandidat yang diuji di `testnet` sebelum dipromosikan ke `mainnet`.

| Aspek | Aturan |
|---|---|
| Format | `X.Y.Z-rc.N` (mis. `contract-v0.3.0-rc.1`, `web-v0.2.0-rc.2`) |
| Urutan | `-rc.1` < `-rc.2` < … < `X.Y.Z` (rilis final) |
| Di mana | Tag `-rc` dibuat di **`testnet`** setelah PR promosi merge; **tidak** di `mainnet` |
| Artifact | `-rc` **boleh** di-deploy ke testnet untuk uji; **tidak pernah** ke mainnet |
| Metadata kontrak | `contract_source_metadata.version` memuat string `-rc` yang sama; hash tetap diverifikasi |
| Promosi ke final | Setelah uji testnet hijau: buat tag final `X.Y.Z` di `mainnet` (setelah PR `testnet → mainnet`), tanpa `-rc` |
| Bila `-rc` gagal | Perbaiki di `dev`, naikkan `-rc.N`, ulangi — jangan mengubah tag `-rc` yang sudah di-push |

```bash
# Kandidat rilis di testnet
git checkout testnet && git pull --ff-only
git tag -a contract-v0.3.0-rc.1 -m "contract-v0.3.0-rc.1 — kandidat uji testnet (TASK-020)"
git push origin contract-v0.3.0-rc.1

# Setelah uji testnet lolos → final di mainnet (setelah PR promosi merge)
git checkout mainnet && git pull --ff-only
git tag -a contract-v0.3.0 -m "contract-v0.3.0 — rilis produksi"
git push origin contract-v0.3.0
```

- Entri CHANGELOG untuk `-rc` **tidak** dipisah; perubahan tetap di `Unreleased` sampai rilis final (atau ditulis di bawah heading `-rc` bila perlu jejak, lalu digabung saat final).
- Dilarang memakai `-rc` pada rilis yang sudah pernah dipromosikan ke mainnet.

## 11. Hotfix: back-merge & penomoran versi ke `testnet`/`dev`

Melengkapi [git-workflow.md](./git-workflow.md) §6 & §15.

- Hotfix = bump **PATCH** pada artefak yang diperbaiki; **satu nomor versi untuk ketiga branch** (tidak ada versi berbeda per branch).
- Urutan penomoran:

```text
1. mainnet: perbaikan darurat → tag contract-vX.Y.(Z+1) (setelah PR hotfix merge + deploy sukses).
2. testnet: back-merge commit mainnet (merge commit) → TIDAK membuat tag baru; versi testnet
   ikut X.Y.(Z+1) setelah back-merge (versi = fakta global, bukan per-branch).
3. dev: back-merge yang sama → versi ikut X.Y.(Z+1).
4. CHANGELOG: entri PATCH ditulis SEKALI, mencakup ketiga branch.
```

- Bila hotfix juga perlu versi testnet eksplisit (mis. artefak web di-deploy testnet lebih dulu) → boleh tag `web-vX.Y.(Z+1)-rc.1` di testnet, lalu final `web-vX.Y.(Z+1)` di mainnet.
- **Dilarang** membuat tag berbeda (`vX.Y.Z` di mainnet, `vX.Y.W` di dev) untuk perbaikan yang sama — menyebabkan versi bercabang dan membingungkan provenance.
- Setelah back-merge, verifikasi `git merge-base --is-ancestor <tag-mainnet> dev` benar-benar true (kode perbaikan ada di ketiga branch).

## 12. Provenance check (tag == artifact ter-deploy)

> Tujuan: membuktikan artifact yang berjalan berasal dari commit yang di-tag, bukan build lokal. Ini melengkapi NEP-330 (§2) untuk kontrak dan berlaku juga untuk FE/indexer.

```text
CHECK 1 — tag → commit
  git rev-parse <tag>^{commit}          # commit yang di-tag
  git show --no-patch --format='%H %ci %s' <tag>

CHECK 2 — commit → artifact
  Kontrak : hash wasm artifact CI == contract_source_metadata on-chain (NEP-330)
            near view <contract> contract_source_metadata
  Web     : image/artifact dibangun dari commit SHA yang sama (tercatat di run summary);
            bandingkan git rev-parse HEAD == SHA di artifact metadata
  Indexer : idem web

CHECK 3 — artifact → running
  Web/indexer : docker compose ps → image tag == <artefak>-<semver> rilis
  Kontrak     : near view <contract> get_version / contract_source_metadata

HASIL
  Ketiganya cocok → provenance sah.
  Ada yang tidak cocok → deploy dianggap GAGAL (ci-cd.md §5); rollback ke tag sebelumnya.
```

- Build ulang di mesin lokal **tidak** membatalkan provenance selama hash artifact CI cocok; bila hash berbeda → artifact berbeda (investigasi supply chain, cicd-security §3).
- Catat hasil verifikasi (tanggal, tag, hash, akun kontrak) di entri CHANGELOG atau log rilis.

## 13. Penomoran versi migration

Migration database punya skema nomor **sendiri** (tidak memakai SemVer artefak) — detail di [migrations.md](../database/migrations.md) §1.

- File: `prisma/migrations/<YYYYMMDDHHMMSS>_<snake_case_nama>/`.
- Nomor urut 4 digit di tabel Log [migrations.md](../database/migrations.md) = rujukan manusia; **bukan** versi SemVer.
- Migration **tidak** diberi tag git terpisah; ia ikut rilis artefak (web/indexer) yang membawanya — versi migration dijelaskan di entri CHANGELOG pada versi artefak tersebut, mis.:
  ```markdown
  ## [web-v0.3.0] - 2026-11-01
  ### Changed
  - Migration `20261030120000_add_reports_status_index` (0002) — index antrean report admin.
  ```
- Migration yang memutus kompatibilitas skema = bagian dari bump **MAJOR/MINOR** artefak pemiliknya (sesuai §9), bukan versi sendiri.

## 14. Perintah tag FE & indexer

```bash
# Frontend (web) — setelah PR promosi merge ke branch target & CI hijau
git checkout testnet && git pull --ff-only
git tag -a web-v0.2.0-rc.1 -m "web-v0.2.0-rc.1 — kandidat testnet"
git push origin web-v0.2.0-rc.1

git checkout mainnet && git pull --ff-only
git tag -a web-v0.2.0 -m "web-v0.2.0 — rilis produksi (TASK-021)"
git push origin web-v0.2.0

# Indexer — fase 2 (tag dibuat saat indexer pertama kali dirilis)
git checkout testnet && git pull --ff-only
git tag -a indexer-v0.1.0 -m "indexer-v0.1.0 — ingestion Neardata + floor/volume dasar (TASK-014)"
git push origin indexer-v0.1.0

# Verifikasi cepat semua tag terbaru + commit-nya
git tag -l --sort=-creatordate | head -n 10
git for-each-ref --sort=-creatordate --format='%(refname:short) %(objectname:short) %(subject)' refs/tags | head
```

- Versi di manifest wajib **sama** dengan tag: `frontend/package.json` `"version"` ↔ `web-vX.Y.Z`; `indexer/package.json` ↔ `indexer-vX.Y.Z`; `contract/Cargo.toml` ↔ `contract-vX.Y.Z`.
- CI (PROPOSED) memeriksa kesesuaian ini saat tag di-push; tag tanpa versi manifest yang cocok → gagalkan rilis.

## 15. Status

- Skema SemVer per-artefak — **DECIDED (ronde 14)**.
- Format CHANGELOG, matriks bump, pre-release, provenance check, penomoran migration — **DECIDED (ronde 15)**.
- Pemeriksaan otomatis versi↔tag di CI — **PROPOSED** (TASK-032).
- Tag & CHANGELOG diisi mulai rilis pertama (Fase 1 scaffold).
