# CI/CD Security

> Pipeline: GitHub → Actions (test + build) → SSH deploy ke VPS. Permukaan supply chain diidentifikasi eksplisit.
> **Definisi pipeline & gate ada di [development/ci-cd.md](../development/ci-cd.md)** — dokumen ini fokus ke aspek keamanannya.
> Aturan branch: [development/git-workflow.md](../development/git-workflow.md). Higienitas secret: [development/secrets-and-gitignore.md](../development/secrets-and-gitignore.md).

## 1. Pipeline (target — PROPOSED, diimplementasi saat scaffold Fase 1)

```text
PULL REQUEST   : branch protection (3 branch: dev/testnet/mainnet) → approval + checks hijau
CI JOB         : cargo fmt/clippy/test (sandbox) + npm audit/cargo audit + build wasm + build FE
SECURITY JOB   : secret scanning (gitleaks) + dependency review
ARTIFACT       : wasm + FE bundle; SIMPAN code hash (sha256) di build summary
DEPLOY         : dev/testnet = auto (gate CI); mainnet = manual-trigger + approval → SSH ke VPS → docker compose pull/up
POST-DEPLOY    : verifikasi code hash kontrak on-chain == hash artifact (SEC-CONTRACT-006); smoke test endpoint
```

## 2. Kontrol per area

| Area | Kontrol |
|---|---|
| Repo | branch protection di `mainnet`/`testnet`/`dev` (force push & delete diblokir, juga untuk admin); signed commits RECOMMENDED |
| PR | wajib test hijau; perubahan file workflow = review wajib; label breaking-change wajib update docs (AGENTS.md sync rules) |
| Dependencies | lockfile committed; `cargo audit` + `npm audit` di CI (gagal build pada critical known CVE); Dependabot on |
| Build | build di GitHub runner tanpa akses secret produksi (build tidak butuh secret — FACT); no `pull_request_target` dengan checkout PR |
| Secrets | GitHub Secrets: SSH deploy key, (tidak ada secret lain yang dibutuhkan CI); rotation saat keluar anggota tim |
| **Secret scanning** | **gitleaks di CI (SEC-CICD-002)** — memindai PR/push; pre-commit hook opsional lokal |
| Deploy credential | SSH key khusus deploy (bukan key personal), bisa di-revoke sendiri; restrict command (forced command) — RESEARCH REQUIRED per setup |
| Production access | Tidak ada SSH interaktif otomatis; hanya workflow deploy + break-glass manual (di-audit via auth.log) |
| Release approval | Mainnet: release = 2-orang (PROPOSED) + checklist deployment.md |

## 3. Supply-chain attack surface (eksplisit)

| Vektor | Dampak | Mitigasi |
|---|---|---|
| Dependency malicious (npm/cargo) | kode jalan di CI/produksi | lockfile, audit gate, minimal deps, review Dependabot |
| Compromised GitHub account | push langsung / ubah workflow | 2FA GitHub wajib (FACT kemampuan), branch protection, review |
| Malicious workflow modification | steal secrets / backdoor deploy | workflow files perubahan = review wajib (path filter) |
| Build artifact swap | deploy kontrak palsu | post-deploy hash verification (SEC-CONTRACT-006) |
| SSH key leak dari GH | akses VPS | dedicated deploy key, revoke cepat, fail2ban, no root |
| Secret ter-commit | kebocoran kredensial permanen | `.gitignore` ketat + gitleaks + rotasi segera (secrets-and-gitignore §6) |

## 4. Status

- Semua PROPOSED — jadi prasyarat TASK-001 "Scaffold workspace repo" di [tasks/backlog.md](../../tasks/backlog.md) untuk item CI; verifikasi hash jadi SEC-CONTRACT-006; secret scanning jadi SEC-CICD-002.
- Kerangka workflow sudah disiapkan di `.github/workflows/` (lihat [development/ci-cd.md](../development/ci-cd.md)).

## 5. Action pinning (SHA) — PROPOSED

Semua third-party action di-pin ke **commit SHA penuh** (bukan tag bergerak seperti `@v4`), agar tag bisa dibalik tanpa mengubah pipeline. Tag ditulis sebagai komentar untuk keterbacaan.

| Workflow | Action | Pin (PROPOSED — ganti tag → SHA saat scaffold) |
|---|---|---|
| [ci.yml](../../.github/workflows/ci.yml) | `actions/checkout` | `@<sha> # v4` |
| ci.yml | `dtolnay/rust-toolchain` | `@<sha> # stable` |
| ci.yml | `actions/cache` | `@<sha> # v4` |
| ci.yml | `actions/upload-artifact` | `@<sha> # v4` |
| ci.yml | `pnpm/action-setup` | `@<sha> # v4` |
| ci.yml | `actions/setup-node` | `@<sha> # v4` |
| [security.yml](../../.github/workflows/security.yml) | `gitleaks/gitleaks-action` | `@<sha> # v2` |
| security.yml | `actions/dependency-review-action` | `@<sha> # v4` |
| deploy-*.yml | `actions/checkout` | `@<sha> # v4` |

- `<sha>` = commit SHA yang dipilih saat scaffold (TASK-001/029); Dependabot mengusulkan bump SHA via PR.
- Action buatan sendiri (composite lokal) tidak perlu pin eksternal.
- Larangan: action yang mengeksekusi kode arbitrer dari PR (`pull_request_target` + checkout PR) — sudah dilarang §2.

## 6. OIDC vs SSH (keputusan)

**Keputusan: SSH deploy key (bukan OIDC).** Status: **PROPOSED** (dikonfirmasi saat TASK-029).

| Aspek | SSH deploy key (DIPILIH) | GitHub OIDC → cloud |
|---|---|---|
| Target | VPS self-hosted (ADR-009) | cloud provider (AWS/GCP/Azure) |
| Kesesuaian | langsung cocok dengan VPS | butuh provider identitas yang mendukung |
| Secret tersimpan | 1 private key di GitHub Secrets (`DEPLOY_SSH_KEY`) | tanpa long-lived secret |
| Risiko | key bocor → akses VPS | salah konfigurasi trust policy |
| Alasan | VPS tidak punya endpoint OIDC native; OIDC tidak mengurangi permukaan berarti untuk satu VPS | — |

- Mitigasi SSH: key khusus deploy, forced command, `AllowUsers deploy`, fail2ban, no root (infrastructure-security §7–8).
- OIDC tetap dicatat sebagai **alternatif** bila kelak pindah ke cloud; bukan jalur MVP.
- GitHub Secrets hanya memuat `DEPLOY_SSH_KEY` (FACT — tidak ada secret lain; build tidak butuh secret).

## 7. Provenance / SLSA (catatan)

- **Level saat ini: SLSA Build L1 (PROPOSED)** — build di GitHub Actions, ada catatan sumber (repo + commit + workflow). Belum L2+ (belum ada build service terisolasi dengan provenance tersigned).
- **Kontrak** sudah lebih kuat dari L1: reproducible build NEP-330 + `contract_source_metadata` + verifikasi hash (SEC-CONTRACT-006) → integritas artifact terbukti secara kriptografis, terlepas dari SLSA level.
- **FE/API**: artifact dibangun di CI; provenance = run ID + commit SHA yang tercatat di artifact summary. Peningkatan ke SLSA L3 (`actions/attest-build-provenance`) dicatat sebagai **⏳ open-by-design** (fase lanjut, bukan gate MVP).
- Larangan: mengklaim SLSA level yang belum diverifikasi; dokumen ini hanya menyatakan target PROPOSED.

## 8. Branch protection (konfigurasi persis) + required checks

Selaras [git-workflow.md](../development/git-workflow.md) §8. Status PROPOSED (TASK-001/031).

| Setting | `mainnet` | `testnet` | `dev` |
|---|---|---|---|
| Require pull request before merging | ya | ya | ya |
| Required approvals | **2** | 1 | 1 |
| Dismiss stale approvals on new commits | ya | ya | ya |
| Require review from Code Owners | ya (`.github/CODEOWNERS`) | ya | tidak |
| Require status checks to pass | ya | ya | ya |
| Require branches up to date | ya | ya | tidak |
| Require conversation resolution | ya | ya | ya |
| Require signed commits | RECOMMENDED | RECOMMENDED | tidak |
| Block force pushes (incl. admins) | ya | ya | ya |
| Block deletions | ya | ya | ya |
| Restrict who can push | ya (via PR saja) | ya | tidak |
| Require linear history | tidak | tidak | tidak |

> **Catatan (sinkron ronde 15)**: strategi merge final = **squash** untuk `feat/*`→`dev`, dan **merge commit (`--no-ff`)** untuk promosi `dev`→`testnet`→`mainnet` + hotfix/back-merge (lihat [git-workflow.md](../development/git-workflow.md) §9). Karena promosi memakai merge commit, *require linear history* **tidak** diaktifkan di branch mana pun. Nilai lama "ya" di baris ini sudah digantikan.

**Required status checks (nama job persis — samakan dengan workflow):**

| Check | Workflow | Sumber |
|---|---|---|
| `Contracts — fmt, clippy, test` | ci.yml | [ci.yml](../../.github/workflows/ci.yml) |
| `Frontend — lint, typecheck, build` | ci.yml | ci.yml |
| `Secret scanning (gitleaks)` | security.yml | [security.yml](../../.github/workflows/security.yml) |
| `Dependency audit (cargo + npm)` | security.yml | security.yml |
| `Dependency review (PR)` | security.yml | security.yml (hanya PR) |

- Perubahan `.github/workflows/**` → review wajib (path filter) + CODEOWNERS.
- Environment `mainnet` WAJIB punya required reviewers (deploy-mainnet.yml).

## 9. Runbook rotasi secret CI/CD

```text
Pemicu: anggota keluar, dugaan kompromi, atau rotasi berkala (tahunan).

A. DEPLOY_SSH_KEY
1. Buat keypair baru di mesin lokal (offline): ssh-keygen -t ed25519 -C nearsea-deploy.
2. Pasang public key di VPS ~deploy/.ssh/authorized_keys dengan forced command (command="...").
3. Update GitHub Secret DEPLOY_SSH_KEY (Settings → Secrets → Actions).
4. Uji: jalankan workflow deploy-dev (bukan mainnet) → sukses.
5. Hapus public key LAMA dari authorized_keys; verifikasi key lama gagal.
6. Catat tanggal/aktor di log insiden.

B. GITHUB_TOKEN / token bot Telegram (bila dipakai CI alert)
1. Revoke token lama (BotFather /revoke atau settings GitHub).
2. Set nilai baru; uji kirim alert.
3. Hapus nilai lama dari semua tempat.

C. Kredensial object storage backup (bila dipakai CI)
1. Rotasi access key di provider; update secret; uji upload + restore list.

VERIFIKASI AKHIR
- Deploy dev/testnet sukses; tidak ada secret lama yang masih valid.
- Tidak ada kredensial baru ter-commit (gitleaks hijau).
- Status SEC-CICD-002 diperbarui bila relevan.
```

- Aturan: **rotasi dulu, baru hapus** (secrets-and-gitignore §6); menghapus commit tidak membatalkan kebocoran.
- Rotation saat keluar anggota = wajib (§2).

## 10. Tautan file workflow (sumber kebenaran)

- [.github/workflows/ci.yml](../../.github/workflows/ci.yml) — build/test/artifact + code hash
- [.github/workflows/security.yml](../../.github/workflows/security.yml) — gitleaks + audit + dependency review
- [.github/workflows/deploy-dev.yml](../../.github/workflows/deploy-dev.yml) — auto-deploy staging
- [.github/workflows/deploy-testnet.yml](../../.github/workflows/deploy-testnet.yml) — auto-deploy testnet (gate CI)
- [.github/workflows/deploy-mainnet.yml](../../.github/workflows/deploy-mainnet.yml) — manual + approval + gate
- Definisi pipeline/gate: [development/ci-cd.md](../development/ci-cd.md) (SSOT proses)
