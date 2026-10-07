# Git Workflow

> Model branching NearSea: **tepat 3 branch permanen** (`mainnet`, `testnet`, `dev`)
> dengan alur promosi satu arah. Aturan ini mengikat semua kontributor (manusia & agent).
> Aturan kerja umum ada di [AGENTS.md](../../AGENTS.md); versi & rilis di
> [versioning-and-release.md](./versioning-and-release.md); pipeline di [ci-cd.md](./ci-cd.md).

## 1. Tiga branch permanen

| Branch | Peran | Environment | Deploy | Proteksi |
|---|---|---|---|---|
| `mainnet` | **Produksi** — kode yang jalan di NEAR mainnet | mainnet | manual-trigger + approval | paling ketat |
| `testnet` | **Rilis testnet** — kandidat rilis publik | testnet | otomatis (gate CI) | ketat |
| `dev` | **Integrasi developer** — tempat semua fitur masuk | staging/dev (RPC testnet, kontrak & subdomain terpisah) | otomatis | sedang |

`mainnet` adalah **default branch** repo (kode produksi = yang pertama dilihat).

## 2. Alur promosi (satu arah)

```text
feat/<nama>  ──PR──►  dev  ──PR──►  testnet  ──PR──►  mainnet
   (kerja)          (integrasi)     (rilis test)     (produksi)
```

- **Tidak ada** jalur pintas `feat/* → mainnet`. Setiap naik tingkat = 1 PR.
- Satu arah saja: perubahan mengalir `dev → testnet → mainnet`, tidak pernah sebaliknya.
- Jika `mainnet` perlu perbaikan darurat → lihat **hotfix** (§6), bukan cherry-pick langsung.

## 3. ATURAN WAJIB: tanya user sebelum merge ke `testnet` / `mainnet`

> Ini aturan eksplisit dari pemilik proyek.

- Merge PR **ke `dev`** → boleh dilakukan agent tanpa tanya (selama CI hijau).
- Merge PR **ke `testnet` atau `mainnet`** → **agent WAJIB bertanya dulu ke user** dan
  menunggu jawaban. Tanpa jawaban = tidak merge.
- Sama untuk **deploy** ke testnet/mainnet (workflow manual) dan **transfer ownership**
  kontrak. Deploy mainnet tanpa persetujuan eksplisit dilarang (AGENTS.md).
- Alasan: `testnet`/`mainnet` adalah titik rilis yang dilihat publik; keputusan promosi
  ada di tangan user, bukan agent.

## 4. Penamaan branch

| Tipe | Pola | Contoh |
|---|---|---|
| Fitur | `feat/<slug>` | `feat/offer-escrow`, `feat/bundle-builder` |
| Perbaikan | `fix/<slug>` | `fix/refund-edge`, `fix/stale-listing` |
| Dokumen | `docs/<slug>` | `docs/git-workflow` |
| Infra/ops | `chore/<slug>` | `chore/ci-gates`, `chore/gitignore` |
| Keamanan | `security/<slug>` | `security/secret-scan` |
| Darurat | `hotfix/<slug>` | `hotfix/payout-rounding` |

- Huruf kecil, pemisah `-`, ringkas, deskriptif.
- Satu branch = satu tujuan. Branch fitur bercabang dari `dev`, bukan dari `mainnet`.
- Hapus branch fitur setelah PR-nya merge.

## 5. Pull Request

- **Wajib** lewat PR — tidak ada push langsung ke `dev`/`testnet`/`mainnet`.
- Syarat merge:
  1. CI hijau (semua job di [ci.yml](../../.github/workflows/ci.yml)) + Security hijau.
  2. Minimal **1 approval** (2 approval untuk `mainnet`).
  3. Tidak ada konflik & branch sudah up-to-date dengan target.
  4. Perubahan menyentuh file workflow (`.github/workflows/**`) → review wajib.
  5. Perubahan keputusan → update `docs/` sesuai [DOCUMENTATION-MAP.md](../DOCUMENTATION-MAP.md).
- Template PR wajib menyebut: tujuan, cara uji, hasil test (lolos/gagal), dampak ke docs.
- PR tanpa hasil test yang jelas **tidak boleh merge** (AGENTS.md Testing Rules).

## 6. Hotfix

Untuk perbaikan darurat yang harus segera ke `testnet`/`mainnet`:

1. Cabang dari `mainnet`: `hotfix/<slug>`.
2. PR → `mainnet` (**tetap wajib tanya user + approval**).
3. Setelah merge, **wajib** alirkan balik ke `testnet` dan `dev` (PR) agar tidak divergen.
4. Catat di [CHANGELOG.md](../../CHANGELOG.md) pada bagian artefak terkait.

## 7. Conventional commits

Format: `tipe(scope): deskripsi`. Tipe: `feat`, `fix`, `docs`, `test`, `chore`, `refactor`,
`perf`, `security`, `build`, `ci`. Contoh: `feat(market): add private listing guard`.

- Satu commit = satu perubahan logis.
- Commit yang menutup task sebutkan ID: `feat(market): bundle pre-validation (TASK-010)`.
- **Signed commits** direkomendasikan (RECOMMENDED, belum aktif — lihat [cicd-security.md](../security/cicd-security.md)).

## 8. Proteksi branch (AKTIF — ronde 18c)

Konfigurasi **aktif** di GitHub (repo `keydeveloping/nearsea`), dipasang lewat API branch protection:

| Setting | `mainnet` | `testnet` | `dev` |
|---|---|---|---|
| Force push | diblokir (termasuk admin) | diblokir (termasuk admin) | diblokir (termasuk admin) |
| Delete branch | diblokir | diblokir | diblokir |
| PR wajib sebelum merge | ya | ya | ya |
| **Required approval** | **0 (ditunda)** | **0 (ditunda)** | **0 (ditunda)** |
| Dismiss stale review | ya | ya | ya |
| Required status checks | 5 check (CI + Security) | 5 check | 5 check |
| Branch harus up-to-date (`strict`) | ya | ya | tidak |
| Conversation resolution | ya | ya | ya |

- **Kenapa approval 0 (bukan 2/1 seperti tabel awal):** repo saat ini hanya punya **satu akun** (`keydeveloping`), dan GitHub **melarang self-approve** — required approval akan mengunci SEMUA PR (termasuk PR Dependabot) tanpa bisa di-merge. **Keputusan user (ronde 18c): tunda approval** sampai ada maintainer kedua, lalu naikkan ke `testnet`=1 / `mainnet`=2 sesuai §16. Ini mengikuti semangat §16 ("jangan menurunkan setting untuk mengakali") tanpa membuat repo tidak bisa dipakai.
- **`enforce_admins: true`** di ketiga branch: admin pun tidak bisa bypass (spec: "diblokir (termasuk admin)").
- **Tag protection**: ruleset `protect-release-tags` memblokir **delete** + **update** tag pola `contract-v*`/`web-v*`/`indexer-v*` (§12).
- **CODEOWNERS** sudah ada (`.github/CODEOWNERS`) dan aktif sebagai reviewer otomatis. *Require review from Code Owners* **belum** diaktifkan — mengaktifkannya sama dengan menambah required approval, yang saat ini akan mengunci repo (lihat di atas).
- **2FA GitHub wajib** untuk semua kontributor (FACT kemampuan GitHub).
- Perubahan aturan branch ini = keputusan proses → update dokumen ini + ADR bila mengubah kebijakan.

### 8a. Required status checks (nama persis — sudah dipasang)

Nama di GitHub **harus sama persis** dengan `name:` job; kalau tidak, check tidak pernah hijau dan PR terkunci. Lima nama di bawah sudah diverifikasi cocok terhadap job yang benar-benar dilaporkan:

| Required check | Workflow |
|---|---|
| `Contracts — fmt, clippy, test` | ci.yml |
| `Frontend — lint, typecheck, build` | ci.yml |
| `Secret scanning (gitleaks)` | security.yml |
| `Dependency audit (cargo + npm)` | security.yml |
| `Dependency review (PR)` | security.yml |

- `Dependency review (PR)` hanya berjalan pada event `pull_request`; pada push ia `skipped` dan tidak memblokir promosi.

## 9. Strategi merge (DECIDED)

Satu strategi per jenis PR — tidak ada kebebasan memilih di GitHub UI:

| Jenis PR | Strategi | Alasan |
|---|---|---|
| `feat/*` `fix/*` `docs/*` `chore/*` `security/*` → `dev` | **Squash merge** | Satu PR = satu commit logis di `dev`; riwayat `dev` linear & mudah di-`git bisect`. |
| Promosi `dev → testnet` | **Merge commit (`--no-ff`)** | Mempertahankan ancestry: promosi berikutnya hanya membawa commit baru, bukan mengulang seluruh `dev`. Wajib agar promosi berulang tidak rusak. |
| Promosi `testnet → mainnet` | **Merge commit (`--no-ff`)** | Idem; `mainnet` dapat di-`git revert -m 1 <merge>` untuk rollback rilis. |
| `hotfix/* → mainnet` | **Merge commit (`--no-ff`)** | Sama seperti promosi; direvert per fitur bila perlu. |
| Back-merge `mainnet → testnet → dev` | **Merge commit (`--no-ff`)** | Menjaga ketiga branch tidak divergen (lihat §6 & §15). |

- **Rebase terlarang** pada branch permanen (`dev`/`testnet`/`mainnet`) dan pada branch yang sudah di-push/di-review. Rebase hanya boleh untuk merapikan branch fitur **lokal** sebelum PR dibuka.
- Squash merge branch fitur: judul commit akhir = judul PR (format conventional commit, §7); nomor PR dicantumkan di body.
- Konsekuensi yang **diterima**: `testnet`/`mainnet` **tidak** memakai *require linear history* (promosi = merge commit). Penyetelan ini menggantikan nilai PROPOSED di [cicd-security.md](../security/cicd-security.md) §8 dan wajib disinkronkan saat scaffold (lihat §11).
- Larangan: `git push --force` ke branch permanen (diblokir branch protection, §8); `git merge` lokal langsung ke branch permanen tanpa PR.

## 10. CODEOWNERS (aktif — TASK-001)

File: `.github/CODEOWNERS` (di-root, berlaku ke seluruh repo) — **sudah dibuat**. Karena kepemilikan saat ini satu maintainer, semua pola menunjuk handle nyata `@keydeveloping` (merangkap semua peran); saat tim bertambah, handle diganti per peran seperti di tabel bawah.

Tabel di bawah = **pemetaan peran** yang jadi acuan saat handle diganti (`@nearsea/core`, `@nearsea/contract-owner`, dst.):

| Pola | Owner (peran) | Alasan |
|---|---|---|
| `*` | `@nearsea/core` | Default reviewer semua perubahan. |
| `/contract/` `/market/` `/factory/` | `@nearsea/contract-owner` | Kode menyentuh dana — review wajib. |
| `/frontend/` `/indexer/` | `@nearsea/app-owner` | Aplikasi & data proyeksi. |
| `/.github/workflows/` | `@nearsea/infra-owner` + `@nearsea/security-owner` | Jalur injeksi secret/supply chain (cicd-security §3). |
| `/docs/security/` `/docs/development/` | `@nearsea/security-owner` | Kebijakan keamanan & proses. |
| `/.gitignore` `/.env.example` `/**/secrets*` | `@nearsea/security-owner` | Higienitas secret (secrets-and-gitignore.md). |
| `/**/Cargo.toml` `/**/package.json` `/**/pnpm-lock.yaml` | `@nearsea/infra-owner` | Dependency & versi artefak. |
| `/CHANGELOG.md` | `@nearsea/release-owner` | Konsistensi rilis (versioning-and-release.md). |

- CODEOWNERS **tidak** menggantikan required approval; ia menambahkan reviewer otomatis saat file tersebut tersentuh.
- Branch protection: `mainnet`/`testnet` **direncanakan** mengaktifkan *Require review from Code Owners*; `dev` tidak (lihat [cicd-security.md](../security/cicd-security.md) §8). **Saat ini belum diaktifkan** karena sama dengan menambah required approval — akan mengunci repo dengan satu akun (§8).
- Bila file CODEOWNERS sendiri diubah → perubahan itu juga butuh review dari owner yang tercantum (self-protecting).

## 11. Required status checks (nama persis)

Nama check **wajib sama persis** dengan `name:` job di workflow — beda satu karakter = check tidak pernah hijau dan PR terkunci. Lima nama di bawah **sudah dipasang** di ketiga branch permanen (ronde 18c).

| Required check | Workflow | Berlaku |
|---|---|---|
| `Contracts — fmt, clippy, test` | [ci.yml](../../.github/workflows/ci.yml) | `dev`, `testnet`, `mainnet` |
| `Frontend — lint, typecheck, build` | [ci.yml](../../.github/workflows/ci.yml) | `dev`, `testnet`, `mainnet` |
| `Secret scanning (gitleaks)` | [security.yml](../../.github/workflows/security.yml) | `dev`, `testnet`, `mainnet` |
| `Dependency audit (cargo + npm)` | [security.yml](../../.github/workflows/security.yml) | `dev`, `testnet`, `mainnet` |
| `Dependency review (PR)` | [security.yml](../../.github/workflows/security.yml) | hanya PR |

- **Job yang belum ada** (ditambahkan saat task pemiliknya, lalu dimasukkan ke required list): `API — test` (TASK-018), `Fuzz smoke` (TASK-006+), dan (pra-rilis) `E2E — golden path` (TASK-008/010). Nama finalnya dikunci saat workflow dibuat; sampai itu ada, jangan menambahkannya sebagai required (check yang tidak pernah muncul = PR tak bisa merge).
- Nama job di atas sudah final untuk job yang ada (TASK-001) — jangan mengubahnya tanpa update tabel ini + cicd-security §8.
- Aturan: menambah/mengganti nama job → update tabel ini + [cicd-security.md](../security/cicd-security.md) §8 **dalam PR yang sama**.
- Status check bersifat *strict* di `mainnet`/`testnet` (branch harus up-to-date sebelum merge) — lihat §8.

## 12. Tag protection

Aturan tag selaras [versioning-and-release.md](./versioning-and-release.md) §1.

| Aturan | Nilai |
|---|---|
| Pola tag dilindungi | `contract-v*`, `web-v*`, `indexer-v*` |
| Buat tag | Hanya via maintainer (manusia) setelah PR promosi merge — **tidak** dari PR/branch fitur |
| Ubah/hapus tag | **Diblokir** — ruleset `protect-release-tags` (ronde 18c) |
| Tag bergerak (moving tag) | **Dilarang** — tag selalu menunjuk commit tetap |
| Tag anotasi | **Wajib** (`git tag -a`) — memuat tagger, tanggal, pesan rilis |
| Tag di branch mana | Hanya pada `testnet`/`mainnet` setelah merge (bukan `dev`) |

```bash
# Contoh pembuatan tag rilis kontrak (setelah PR promosi merge & CI hijau)
git checkout mainnet
git pull --ff-only
git tag -a contract-v0.1.0 -m "contract-v0.1.0 — rilis market + factory (TASK-032)"
git push origin contract-v0.1.0

# Verifikasi tag menunjuk commit yang benar sebelum deploy
git rev-parse contract-v0.1.0^{commit}
git show --no-patch --format='%H %s' contract-v0.1.0
```

- GitHub: aktifkan **tag protection rule** (Settings → Tags) untuk pola di atas; hanya role maintainer yang boleh membuat tag yang cocok.
- Tag **tidak boleh** dibuat sebelum artifact lolos gate deploy (versioning-and-release.md §5 langkah 5).

## 13. Prosedur resolusi konflik

```text
1. JANGAN selesaikan konflik langsung di branch permanen.
2. Sinkronkan branch fitur dengan target:
   git checkout feat/<slug>
   git fetch origin
   git merge origin/dev          # merge, bukan rebase (branch sudah di-push)
3. Selesaikan konflik file per file; jangan "terima semua" tanpa membaca.
4. Untuk file yang dimiliki owner tertentu (CODEOWNERS §10) → minta owner memutuskan isi.
5. Jalankan test lokal yang relevan (cargo test / pnpm test) sebelum push.
6. git add <file> && git commit (merge commit; jangan amend commit yang sudah di-push)
7. git push origin feat/<slug>
8. Bila konflik menyentuh dokumen → jalankan checklist DOCUMENTATION-MAP.md sebelum minta review ulang.
```

- Konflik pada **lockfile** (`pnpm-lock.yaml`, `Cargo.lock`): jangan merge manual — regenerate (`pnpm install` / `cargo update -p <crate>` sesuai kebutuhan) lalu commit hasilnya.
- Konflik pada **migration** (dua migration nomor sama): migration yang belum di-merge boleh di-rename; yang sudah di-merge **tidak** — buat migration koreksi baru ([migrations.md](../database/migrations.md) §3).
- Konflik pada **branch promosi**: konflik `dev → testnet` menandakan promosi sebelumnya terlewat — hentikan, selesaikan di `dev` dulu, jangan paksa merge.
- Setelah konflik besar (menyentuh > 10 file atau file kritis): wajib **re-review** dari awal, bukan sekadar approve lama (branch protection *dismiss stale approvals* sudah menangani ini di `mainnet`/`testnet`).

## 14. Dependabot / Renovate (kebijakan)

**Keputusan: Dependabot** (bawaan GitHub; selaras [cicd-security.md](../security/cicd-security.md) §2 "Dependabot on"). Renovate **tidak** dipakai (menambah app/secret pihak ketiga tanpa manfaat pada skala ini).

| Ekosistem | Direktori | Jadwal | PR dibuka ke |
|---|---|---|---|
| `cargo` | `/` (workspace: `contract/`, `market/`, `factory/`) | mingguan (Senin) | `dev` |
| `npm` (pnpm) | `/frontend`, `/indexer` | mingguan (Senin) | `dev` |
| `github-actions` | `/` | mingguan | `dev` |
| `docker` | `/` (base image) | bulanan | `dev` |

Kebijakan PR dependency:

- Dependabot **selalu** membuka PR ke `dev` (tidak pernah langsung ke `testnet`/`mainnet`) — alur promosi tetap berlaku.
- Patch/minor: boleh auto-merge **hanya** bila (a) CI + Security hijau, (b) bukan paket kritis (kontrak, wallet, auth, crypto), (c) tidak mengubah lockfile lebih dari paket itu sendiri.
- **Major** / paket kritis (`near-sdk`, `near-workspaces`, `next`, `near-connect`, library crypto) → review manusia wajib + catat di CHANGELOG bila memengaruhi perilaku.
- Security advisory (critical/high) → PR diperlakukan **prioritas**, tetap lewat `dev` → `testnet` → `mainnet`.
- Bump action GitHub = bump **commit SHA** (bukan tag) — lihat [cicd-security.md](../security/cicd-security.md) §5.
- Setiap PR dependency tetap tunduk DOCUMENTATION-MAP T10/T16 bila mengubah proses build/test.

```yaml
# .github/dependabot.yml — AKTIF (dibuat TASK-001); jalur cargo di root workspace
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"            # workspace root (Cargo.lock ada di sini)
    schedule: { interval: weekly, day: monday }
    target-branch: dev
    open-pull-requests-limit: 5
  - package-ecosystem: npm
    directory: "/frontend"
    schedule: { interval: weekly, day: monday }
    target-branch: dev
    open-pull-requests-limit: 5
  - package-ecosystem: github-actions
    directory: "/"
    schedule: { interval: weekly }
    target-branch: dev
```

## 15. Semantik rilis: tag dari hotfix & back-merge

Melengkapi §6 (alur hotfix) dan [versioning-and-release.md](./versioning-and-release.md) §8.

- **Tag hanya dibuat di ujung alur**, setelah PR merge ke branch target — bukan di branch `hotfix/*`.
- **Hotfix menaikkan PATCH** artefak yang diperbaiki (mis. `contract-v0.2.3` → `contract-v0.2.4`), lalu tag dibuat di `mainnet` **dan** nomor yang sama dialirkan balik ke `testnet`/`dev` (satu versi = satu fakta, tidak ada versi berbeda per branch).
- Back-merge **wajib memakai merge commit** (§9) dan menyertakan entri CHANGELOG yang sama; dilarang cherry-pick yang menghasilkan hash berbeda tanpa catatan.
- Bila perbaikan hanya relevan untuk mainnet (mis. konfigurasi produksi) → tetap back-merge kode; konfigurasi environment tidak pernah di-commit ke branch.
- Urutan hotfix rilis:
  ```text
  1. hotfix/<slug> dari mainnet → PR → mainnet (WAJIB tanya user, §3)
  2. Merge → CI hijau → deploy mainnet → verifikasi hash (kontrak) / smoke test (web)
  3. Tag PATCH di mainnet (git tag -a contract-vX.Y.Z ...) + push tag
  4. Back-merge mainnet → testnet → dev (merge commit, PR, boleh tanpa tanya user karena menurunkan status)
  5. Catat di CHANGELOG pada versi PATCH + bagian artefak terkait
  ```
- **Rollback rilis** = `git revert -m 1 <merge-commit-promosi>` di branch target + deploy ulang tag sebelumnya; dicatat di CHANGELOG ([ci-cd.md](./ci-cd.md) §6).

## 16. Named approvers (peran)

Approval dihitung per **peran** (satu orang boleh memegang beberapa peran saat tim masih kecil, tetapi peran tetap dicatat agar tidak ada akses tanpa pemilik).

| Peran | Tanggung jawab approval | Branch yang di-approve |
|---|---|---|
| `release-owner` (maintainer) | Approval akhir promosi & tag rilis | `testnet`, `mainnet` |
| `contract-owner` | Review kontrak & perubahan storage/API on-chain | semua PR kontrak |
| `app-owner` | Review FE/API/indexer | semua PR aplikasi |
| `infra-owner` | Review workflow, Docker, provisioning | semua PR infra |
| `security-owner` | Review kebijakan security, secret, workflow | PR `.github/**`, `docs/security/**` |

- Placeholder handle: `@nearsea/core`, `@nearsea/contract-owner`, dst. — diisi di CODEOWNERS saat scaffold (§10).
- `mainnet` = **2 approval** (minimal satu dari `release-owner`, satu dari `security-owner` atau `contract-owner`); `testnet`/`dev` = 1 approval (lihat §8).
- Deploy ke `testnet`/`mainnet` tetap memerlukan **persetujuan user** (§3) — approval GitHub tidak menggantikan aturan itu.
- Bila hanya ada satu maintainer, peran boleh dirangkap **tetapi** requirement approval tetap berlaku; jangan menurunkan setting branch protection untuk mengakalinya.

## 17. Docs-only fast-path

Perubahan **hanya dokumen** (tanpa file kode/workflow) boleh dipercepat **di `dev` saja**:

| Syarat | Nilai |
|---|---|
| Path yang diubah | hanya `*.md`, `docs/**`, `tasks/**` (tanpa `.github/**`, `.gitignore`, `.env.example`, `CHANGELOG.md`) |
| CI | job kode kontrak/FE boleh di-*skip* via path filter — **Secret scanning tetap jalan** |
| Approval | 1 approval (tetap) — fast-path hanya soal job CI, bukan soal review |
| Target | `dev` saja; promosi `docs` ke `testnet`/`mainnet` tetap lewat alur normal |
| Pengecualian | perubahan yang mengubah **kebijakan** (docs/security, docs/development) = review `security-owner` wajib |

- Implementasi: `paths-ignore`/`paths` filter di [ci.yml](../../.github/workflows/ci.yml) — PROPOSED, ditambahkan saat scaffold.
- Aturan: fast-path **tidak** berlaku untuk file yang memengaruhi build/runtime meski berekstensi `.md` (mis. dokumen yang di-generate ke FE). Ragu → jalankan CI penuh.

## 18. Commit signing (setup)

Selaras [cicd-security.md](../security/cicd-security.md) §2 (signed commits **RECOMMENDED**) dan §7 (provenance). Status: **PROPOSED** — diaktifkan saat scaffold.

```bash
# Opsi A — SSH signing (paling sederhana; kunci SSH sudah ada)
ssh-keygen -t ed25519 -C "signing@nearsea" -f ~/.ssh/nearsea_signing
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/nearsea_signing.pub
git config --global commit.gpgsign true
git config --global tag.gpgsign true

# Opsi B — GPG signing
gpg --full-generate-key                      # pilih ed25519
gpg --list-secret-keys --keyid-format=long   # catat KEY_ID
git config --global user.signingkey <KEY_ID>
git config --global commit.gpgsign true
git config --global tag.gpgsign true
```

Langkah onboarding:

```text
1. Generate kunci signing (Opsi A atau B di atas) — kunci khusus signing, bukan kunci dana.
2. Tambahkan public key ke GitHub (Settings → SSH and GPG keys → Signing keys).
3. Set config lokal/global (perintah di atas).
4. Uji: git commit --allow-empty -m "chore: verify signing" → pastikan "Verified" di GitHub.
5. Aktifkan "Require signed commits" di branch protection `mainnet`/`testnet` (setelah semua kontributor siap).
6. Simpan public key signing di secret store tim agar bisa diverifikasi; private key tidak pernah keluar dari mesin.
```

- Kunci signing **terpisah** dari owner/deploy key (key-management §1) — kompromi kunci signing tidak boleh menyentuh dana.
- Bila signing belum aktif, commit tetap wajib punya identitas yang benar (`user.name`/`user.email` sesuai akun GitHub).

## 19. Status

- Model 3 branch — **DECIDED (ronde 14)**.
- Strategi merge (§9) — **DECIDED (ronde 15)**; menyesuaikan *linear history* `testnet`/`mainnet` (sinkronkan cicd-security §8).
- Proteksi branch + tag protection — **AKTIF (ronde 18c)**, dipasang lewat API setelah repo dijadikan publik. **Required approval ditunda = 0** sampai ada maintainer kedua (alasan di §8). Commit signing — **PROPOSED**. **`CODEOWNERS` & `dependabot.yml` sudah dibuat (TASK-001)** — lihat §10/§14.
- Named approvers (§16) — **PROPOSED** (handle peran diganti saat tim bertambah; saat ini satu maintainer `@keydeveloping` merangkap semua peran).
