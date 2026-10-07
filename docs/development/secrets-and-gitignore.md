# Secrets & Gitignore

> **Single source of truth untuk higienitas file & kredensial.** Menjawab permintaan pemilik
> proyek: *".gitignore harus sangat ketat; setiap ada file baru, cek dulu isinya; kalau sensitif,
> masukkan ke .gitignore SEBELUM mengisi file itu."*
> Terkait: [.gitignore](../../.gitignore), [.env.example](../../.env.example),
> [key-management.md](../security/key-management.md), [cicd-security.md](../security/cicd-security.md),
> [environments.md](../deployment/environments.md).

## 1. Aturan emas (alur wajib untuk SETIAP file baru)

```text
1. Sebelum membuat/mengisi file baru → tentukan dulu: apakah isinya bisa sensitif?
2. Jika YA (secret, kredensial, data pribadi, dump DB, kunci) →
   tambahkan polanya ke .gitignore DULU, baru isi filenya.
3. Jalankan secret scan (gitleaks) sebelum commit.
4. Verifikasi: `git status` tidak memuat file sensitif.
```

**Dilarang** mengisi file berisi kredensial lalu berharap bisa di-ignore belakangan — begitu
ter-commit, secret dianggap bocor dan harus dirotasi.

## 2. Konfigurasi kredensial (WAJIB)

- Kredensial **hanya** dibaca dari **environment variable** atau **secret store**.
- **Dilarang** menulis kredensial yang bisa dipakai (usable literal) di dalam source, contoh,
  maupun test. Termasuk di `.env.example` — hanya placeholder.
- `.env` asli tidak pernah masuk repo; `.env.example` adalah satu-satunya template yang di-commit.
- Secret di VPS: `.env` server-side `chmod 600`, di luar web-root, terpisah dari app user.

## 3. Klasifikasi

| Kelas | Contoh | Boleh di frontend? | Tempat |
|---|---|---|---|
| PUBLIC | `NEAR_NETWORK`, `NEAR_RPC_URL`, `MARKET_CONTRACT_ID` | ya (`NEXT_PUBLIC_*`) | env publik |
| SECRET | `DATABASE_URL`, `JWT_SECRET`, `TELEGRAM_BOT_TOKEN`, `BACKUP_S3_*`, `IPFS_PINNING_KEY` | **tidak** | env server/CI |
| KEY | owner/deploy key, seed phrase, SSH deploy key | **tidak** | VPS `~/.near-keys/` (600) / GitHub Secret |

Daftar variabel lengkap: [environments.md](../deployment/environments.md).

## 4. Apa yang di-ignore (ringkas)

`.gitignore` menutup: `.env*` (kecuali `.env.example`), `*.pem`/`*.key`/`*.p8`/`*.p12`,
`near-credentials/`, `.near-keys/`, `.near/`, `neardev/`, `*.seed`/`seed-phrase*`, `.ssh/`,
`*.ppk`, `credentials.json`, `service-account*.json`, `.aws/`, `.npmrc`, dump DB
(`*.sql`, `pg_dump*`, `backup/`), log (`*.log`), `target/`, `node_modules/`, `.next/`, `dist/`,
`__pycache__/`, `.venv/`, OS/editor, Docker volume.

Pengecualian penting: **file migration Prisma (`.sql`) tidak di-ignore** — itu kode, bukan dump
(`!**/prisma/migrations/**/*.sql`).

## 5. Enforcement (berlapis)

| Lapis | Alat | Kapan |
|---|---|---|
| Lokal | gitleaks pre-commit (opsional) | sebelum commit |
| CI | gitleaks (`.github/workflows/security.yml`) | setiap PR/push |
| CI | dependency review + `cargo audit`/`npm audit` | setiap PR/push + mingguan |
| Review | perubahan `.github/workflows/**` = review wajib | PR |

- Branch protection + 2FA GitHub wajib (lihat [git-workflow.md](./git-workflow.md)).
- Deploy key khusus (bukan key personal), bisa di-revoke; forced command (RECOMMENDED).

## 6. Jika secret terlanjur bocor

1. **Rotasi segera** — anggap bocor, ganti nilainya (bukan sekadar hapus commit).
2. Revoke kunci terkait (SSH/deploy key, token bot, kredensial DB).
3. Catat insiden di [incident-response.md](../security/incident-response.md).
4. Hapus dari history (git filter-repo) **setelah** rotasi — rotasi yang utama, bukan penghapusan.

## 7. Isi `.gitignore` (kutipan file nyata)

File aktif: [.gitignore](../../.gitignore) — disalin utuh di bawah agar dokumen ini bisa diaudit tanpa membuka file. **Bila berbeda, file yang benar** (dokumen disinkronkan dalam PR yang sama).

```gitignore
# =============================================================================
# NearSea — .gitignore (KETAT)
# Aturan emas (AGENTS.md + docs/development/secrets-and-gitignore.md):
#   Sebelum membuat file baru, cek isinya. Jika sensitif → tambahkan pola ignore
#   DULU ke file ini, baru isi filenya. Tidak ada secret yang boleh masuk git.
# =============================================================================

# -----------------------------------------------------------------------------
# SECRETS — jangan pernah di-commit (SEC-KEY-001, SEC-CICD-001)
# -----------------------------------------------------------------------------
.env
.env.*
!.env.example
*.env
.envrc

# Kunci privat / kredensial NEAR
*.pem
*.key
*.p8
*.p12
*.pfx
id_rsa
id_ed25519
id_ecdsa
near-credentials/
.near-keys/
.near/
neardev/
*.seed
seed.txt
seed-phrase*

# Kredensial cloud / CI / SSH
.ssh/
*.ppk
credentials.json
service-account*.json
gcloud-key*.json
aws-credentials*
.aws/
.npmrc
.pypirc
.netrc

# Database dump / backup (bisa memuat data sensitif)
# CATATAN: file migration Prisma (.sql) DIKECUALIKAN di bawah — itu bagian kode, bukan dump.
*.sql
*.sql.gz
*.dump
pg_dump*
*.bak
backup/
backups/
!**/prisma/migrations/**/*.sql
!**/migrations/**/*.sql

# Log (bisa memuat token/PII)
*.log
logs/
npm-debug.log*
yarn-debug.log*
yarn-error.log*
pnpm-debug.log*

# -----------------------------------------------------------------------------
# RUST (kontrak) — target/ besar & reproducible
# -----------------------------------------------------------------------------
target/
**/target/
*.wasm
!**/res/*.wasm
Cargo.lock.bak

# -----------------------------------------------------------------------------
# NODE / FRONTEND
# -----------------------------------------------------------------------------
node_modules/
**/node_modules/
.pnpm-store/
.pnpm-debug.log
.yarn/cache/
.yarn/install-state.gz
dist/
build/
.next/
out/
.turbo/
coverage/
*.tsbuildinfo
.vite/

# -----------------------------------------------------------------------------
# PYTHON (tooling/script bila ada)
# -----------------------------------------------------------------------------
__pycache__/
*.py[cod]
.venv/
venv/
.pytest_cache/
.mypy_cache/

# -----------------------------------------------------------------------------
# OS / EDITOR
# -----------------------------------------------------------------------------
.DS_Store
Thumbs.db
desktop.ini
*.swp
*.swo
*~
.idea/
.vscode/*
!.vscode/settings.json.example
!.vscode/extensions.json

# -----------------------------------------------------------------------------
# DOCKER / INFRA lokal
# -----------------------------------------------------------------------------
docker-compose.override.yml
*.pid
*.sock
postgres-data/
pgdata/

# -----------------------------------------------------------------------------
# TOOLING LOKAL (bukan bagian proyek)
# -----------------------------------------------------------------------------
.mimosa/
.zcode/
```

- **Pengecualian yang disengaja** (jangan diubah tanpa alasan): `!.env.example` (template), `!**/prisma/migrations/**/*.sql` dan `!**/migrations/**/*.sql` (migration = kode, bukan dump), `!**/res/*.wasm` (wasm contoh kecil bila ada), `!.vscode/extensions.json`.
- `*.wasm` di-ignore kecuali `res/` — artifact build tidak pernah di-commit; wasm rilis disimpan sebagai GitHub Release artifact ([ci-cd.md](./ci-cd.md) §9).
- Setiap pola baru → update tabel §4 + uji: `git check-ignore -v <file>`.

## 8. Jadwal rotasi secret

> Prinsip: **rotasi dulu, baru bersihkan jejak** (§6). Rotasi insidental **tidak** menunggu jadwal. Prosedur per-secret: [key-management.md](../security/key-management.md) §6 dan [cicd-security.md](../security/cicd-security.md) §9.

| Secret | Rotasi rutin | Pemicu insidental | Efek rotasi |
|---|---|---|---|
| Owner/DAO council key | tahunan (drill) | anggota keluar / dugaan bocor (CS-1) | proposal DAO `remove/add_member`; pause dulu bila darurat |
| `DEPLOY_SSH_KEY` (GitHub → VPS) | tahunan | anggota keluar / bocor | deploy key lama dicabut; uji `deploy-dev` |
| `JWT_SECRET` | tahunan | bocor / insiden | semua sesi invalid → user login ulang |
| `DATABASE_URL` / password DB | tahunan | bocor / anggota keluar | rolling restart; kredensial lama dicabut |
| `TELEGRAM_BOT_TOKEN` | saat perlu | bocor / bot dikompromi | `/revoke` BotFather → token baru |
| `BACKUP_S3_*` | tahunan | bocor | rotasi access key; uji upload + restore list |
| `IPFS_PINNING_KEY` | tahunan (fase lanjut) | bocor | rotasi di provider pinning |
| Signing key commit (SSH/GPG) | tahunan | bocor | public key baru di GitHub; kunci lama dihapus |
| GitHub PAT (bila ada) | ≤ 90 hari | bocor | revoke + buat baru (scope minimum) |

- Setiap rotasi → **verifikasi negatif**: kredensial lama harus **gagal**; `/api/health` 200 ([database-security.md](../security/database-security.md) §10).
- Catat di log rotasi (tanggal, aktor, secret yang dirotasi) — **tanpa** nilai secret.
- Rotasi tahunan didorong lewat reminder kalender/issue; tidak ada rotasi otomatis di MVP (⏳ open-by-design).

## 9. Konfigurasi gitleaks & allowlist

File konfigurasi: `.gitleaks.toml` (root) — **ada** (dibuat ronde 16; dipakai gate CI + scan lokal).

```toml
# .gitleaks.toml — PROPOSED
title = "NearSea secret scanning"
[extend]
useDefault = true          # pakai ruleset bawaan gitleaks (ed25519, AWS, JWT, dll.)

[allowlist]
description = "Pengecualian yang disengaja — tidak ada kredensial usable di sini"
paths = [
  '''docs/''',                     # dokumen boleh memuat CONTOH placeholder
  '''\.env\.example$''',           # template placeholder
  '''CHANGELOG\.md$''',
  '''pnpm-lock\.yaml$''',          # hash integritas, bukan secret
  '''Cargo\.lock$''',
]
# Placeholder yang jelas-jelas bukan secret asli:
regexes = [
  '''<generate-random-secret>''',
  '''<bot-token>''',
  '''<access-key>''',
  '''<secret-key>''',
  '''<pinning-provider-key>''',
  '''ed25519:7Kq3\.\.\.''',        # contoh publik di dokumentasi API
  '''0x[0-9a-fA-F]{40}''',         # alamat akun NEAR (publik, bukan secret)
]
```

- **Allowlist hanya untuk pola placeholder/dokumen**, bukan untuk menekan temuan nyata. Menambahkan allowlist wajib alasan di PR + review `security-owner`.
- Kunci allowlist sebaiknya **spesifik** (path/regex sempit); dilarang `allowlist = ["."]` atau menonaktifkan rule.
- CI memakai ruleset bawaan + file ini: `gitleaks detect --config .gitleaks.toml --redact`.
- Gitleaks memindai **seluruh history** di CI (`fetch-depth: 0` di [security.yml](../../.github/workflows/security.yml)) — bukan hanya diff.
- Temuan positif = **blok merge** (SEC-CICD-002); jangan menandai "false positive" tanpa perbaikan/allowlist beralasan.

## 10. Pemasangan pre-commit hook (perintah)

Selaras [code-standards.md](./code-standards.md) §14. Status **PROPOSED** (TASK-001/031).

```bash
# 1. Pasang framework pre-commit (sekali per mesin)
pipx install pre-commit          # disarankan; alternatif: pip install --user pre-commit

# 2. Dari root repo — pasang hook git
pre-commit install                       # hook pre-commit
pre-commit install --hook-type commit-msg  # commitlint (format conventional commit)

# 3. Uji semua hook pada seluruh file (sekali setelah pasang)
pre-commit run --all-files

# 4. Uji hanya file yang berubah (jalan otomatis saat commit)
pre-commit run

# 5. Perbarui versi hook (revisi pin) secara berkala
pre-commit autoupdate

# 6. (Opsional) gitleaks langsung tanpa pre-commit
gitleaks detect --config .gitleaks.toml --redact -v
```

- `pre-commit` versi lokal **melengkapi** CI; bypass (`git commit --no-verify`) hanya untuk darurat yang dicatat, dan CI tetap penjaga akhir.
- Bila hook gitleaks gagal karena file di-ignore → verifikasi dengan `git check-ignore -v <file>` (file mungkin belum masuk `.gitignore`, §1).
- Hook berjalan di Windows/macOS/Linux; gitleaks & cargo harus tersedia di PATH (dokumentasikan di README prasyarat).

## 11. Secret store (keputusan / status)

| Kebutuhan | Produk | Status |
|---|---|---|
| Secret runtime server (VPS) | **file `.env.server` di luar web-root, `chmod 600`**, diisi manual oleh operator | **DECIDED (interim MVP)** — model sederhana, satu VPS (ADR-009) |
| Secret CI | **GitHub Secrets** (repo/environment) | **DECIDED** — hanya `DEPLOY_SSH_KEY` (+ `TELEGRAM_*` bila notifikasi CI dipakai) |
| Secret store terkelola (Vault / Infisical / SOPS + age / AWS SM) | — | **PROPOSED / ⏳ open-by-design** — dipertimbangkan saat pindah multi-host / ada anggota kedua |
| Kunci dana (owner/DAO) | seed offline + Sputnik DAO (mainnet) | **DECIDED** ([key-management.md](../security/key-management.md)) |

- **Yang mengikat sekarang**: tidak ada kredensial usable di repo (source/contoh/test), `.env` tidak di-commit, secret dibaca dari env/secret store (§2).
- Bila kelak memakai secret store terkelola: `.env.server` diganti referensi (mis. `sops exec-env secrets.enc.yaml`), dan dokumen ini + [environments.md](../deployment/environments.md) diperbarui.
- Dilarang menyimpan secret di variabel GitHub **vars** (hanya untuk nilai non-rahasia seperti URL/host) — secret selalu di **Secrets**.

## 12. Verifikasi pembersihan history git

Dipakai **setelah** rotasi (§6) dan sebelum menganggap kebocoran selesai.

```bash
# 1. Cari string/secret di SELURUH history (bukan hanya HEAD)
git log --all --full-history -S '<secret-yang-bocor>' --oneline
git rev-list --all --objects | git cat-file --batch-check='%(objectname) %(objecttype) %(rest)' \
  | grep -i '\.env' | head

# 2. Jalankan gitleaks atas seluruh history
gitleaks detect --config .gitleaks.toml --redact -v

# 3. Bersihkan history (SETELAH rotasi!) — contoh git-filter-repo
git filter-repo --invert-paths --path .env --path-glob '*.pem'
#    alternatif per-string:
#    git filter-repo --replace-text replacements.txt

# 4. Paksa push + minta semua kolaborator re-clone (hash berubah)
git push --force --all
git push --force --tags

# 5. Verifikasi ulang (harapan: 0 temuan)
gitleaks detect --config .gitleaks.toml --redact -v
git log --all -S '<secret-yang-bocor>' --oneline   # harapan: kosong

# 6. Cabut akses/credential lama di provider; audit akses pasca-bersih.
```

- **Urutan wajib**: rotasi/revoke dulu (§6) → baru bersihkan history. Menghapus commit **tidak** membatalkan kebocoran.
- Karena history berubah (hash commit berubah), semua PR/branch terbuka harus di-rebase/re-clone; koordinasikan dengan tim + catat di [incident-response.md](../security/incident-response.md).
- Bila repo punya fork/mirror → history di sana tetap memuat secret sampai dihapus; anggap secret tetap bocor.
- Verifikasi akhir = **0 temuan gitleaks** atas seluruh history + bukti kredensial lama tidak valid.

## 13. Kredensial backup S3 (spesifik)

> `BACKUP_S3_*` dipakai cron `pg_dump` → object storage ([disaster-recovery.md](../deployment/disaster-recovery.md)); ini kredensial **tulis** ke bucket berisi data (walau minim-PII, tetap dilindungi).

| Aspek | Aturan |
|---|---|
| Penyimpanan | Hanya di `.env.server` (server, `chmod 600`) — **tidak** di repo, **tidak** di frontend, **tidak** di log |
| Izin kredensial | **Least privilege**: hanya `PutObject`/`GetObject`/`ListBucket` pada **satu bucket** backup; **tanpa** `DeleteBucket`, tanpa akses bucket lain |
| Kunci terpisah | Kredensial backup **berbeda** dari kredensial runtime app & dari DB — kompromi satu tidak membuka yang lain |
| Enkripsi | Bucket: SSE provider (AES-256) aktif; objek backup boleh di-enkripsi tambahan (`age`/gpg) sebelum upload (PROPOSED) |
| Retensi | **7 harian + 4 mingguan + 3 bulanan** ([database-security.md](../security/database-security.md) §4; SSOT: disaster-recovery.md §Jadwal); lifecycle rule hapus otomatis (bukan kredensial yang menghapus manual) |
| Rotasi | tahunan / insidental (§8); setelah rotasi → **uji upload + restore list** |
| Lokasi | Provider **terpisah** dari VPS (tidak satu akun/region dengan server produksi) agar VPS compromise ≠ backup compromise |
| Akses baca | Restore hanya dari mesin operator/break-glass, bukan dari container app |
| Verifikasi | Backup dianggap **INVALID** sampai restore drill lulus (SEC-DB-004) |

```bash
# Cron backup (server-side) — kredensial dari env, TIDAK ditulis di skrip
pg_dump "$DATABASE_URL" | gzip > "/var/backups/nearsea-$(date -u +%Y%m%dT%H%M%SZ).sql.gz"
aws --endpoint-url "$BACKUP_S3_ENDPOINT" s3 cp /var/backups/*.sql.gz "s3://$BACKUP_BUCKET/"
# verifikasi unggah + lifecycle; jangan echo kredensial ke log
```

- Dilarang menaruh `BACKUP_S3_SECRET` di command line yang terlihat di `ps`/history shell — pakai env/credential file dengan izin ketat.
- Bila kredensial backup bocor → rotasi access key segera + audit objek (apakah ada unduhan tidak wajar) + catat insiden.

## 14. Status

- Kebijakan & `.gitignore` — **DECIDED (ronde 14)**; file `.gitignore` sudah aktif.
- Kutipan `.gitignore`, jadwal rotasi, konfigurasi gitleaks, hook, history-clean, kredensial S3 (§7–§13) — **DECIDED (ronde 15)**.
- Secret store terkelola, enkripsi backup tambahan (`age`/gpg), rotasi otomatis — **PROPOSED / ⏳ open-by-design** (fase lanjut).
- Pre-commit hook lokal — PROPOSED, diaktifkan saat scaffold (TASK-001/031).
- Secret scanning CI — PROPOSED (aktif bersama CI, TASK-029).
