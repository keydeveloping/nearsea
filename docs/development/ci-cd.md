# CI/CD

> **Single source of truth untuk proses pipeline & deploy.** Aspek ancaman supply-chain
> ada di [cicd-security.md](../security/cicd-security.md) (dokumen itu merujuk ke sini untuk
> definisi pipeline). Aturan branch: [git-workflow.md](./git-workflow.md);
> versi & tag: [versioning-and-release.md](./versioning-and-release.md).

## 1. Peta branch → environment → deploy

| Branch | Environment | Trigger deploy | Approval | Artefak |
|---|---|---|---|---|
| `dev` | staging (RPC testnet, kontrak/subdomain dev terpisah) | otomatis saat push ⚠️ **belum aktif — manual sampai TASK-029** | tidak | web + kontrak dev |
| `testnet` | testnet publik | otomatis saat push (gate CI) ⚠️ **belum aktif — manual sampai TASK-029** | 1 (environment) | web + kontrak testnet |
| `mainnet` | produksi | **manual saja** (`workflow_dispatch`) | wajib (reviewer) | web + kontrak mainnet |

> **Status TASK-001:** trigger `push` untuk `dev`/`testnet` sengaja belum dipasang — environment belum di-provision (TASK-028) dan wiring SSH belum ada (TASK-029). Ketiga workflow deploy kini `workflow_dispatch` saja + guard variabel environment (§7).

## 2. Tahapan pipeline

```text
1. PR OPEN        → CI + Security jalan (lihat .github/workflows/)
2. CI             → cargo fmt/clippy/test (sandbox) + lint/typecheck/test/build FE
                    + cargo audit / npm audit + build wasm+ABI + verifikasi NEP-330
                    + upload artifact + code hash
3. SECURITY       → secret scanning (gitleaks) + dependency review + audit dependensi
4. MERGE          → hanya jika semua check hijau + approval cukup
5. RELEASE        → tag `*-vX.Y.Z` (di mainnet) memicu release.yml: versi manifest == tag,
                    build reproducible di container ter-pin, metadata NEP-330 == tag,
                    artifact + hash dilampirkan ke GitHub Release
6. DEPLOY         → workflow per-branch (SSH ke VPS → docker compose pull/up)
7. POST-DEPLOY    → verifikasi hash kontrak (NEP-330) + smoke test /api/health
8. ROLLBACK       → revert merge commit di branch target + deploy ulang (jika perlu)
```

## 3. Gate (wajib lulus sebelum merge/deploy)

| Gate | Berlaku | Gagal → |
|---|---|---|
| Format + clippy tanpa warning | kontrak | blok merge |
| Unit + sandbox test hijau | kontrak | blok merge |
| Lint + format + typecheck + test + build | FE | blok merge |
| `cargo audit` / `pnpm audit` (critical/high) | semua | blok merge |
| Secret scanning (gitleaks) | semua | blok merge |
| Metadata NEP-330 (versi + link tertanam) | kontrak | blok merge |
| Versi manifest == tag rilis | kontrak/web/indexer | blok rilis |
| Reproducible build + verifikasi hash | kontrak (testnet/mainnet) | blok deploy |
| Audit eksternal lulus | kontrak (mainnet, TASK-027) | blok deploy |

- **Advisory tanpa patch** dicatat **eksplisit** di `frontend/pnpm-workspace.yaml`
  (`auditConfig.ignoreCves`) sehingga terlihat saat review — bukan disenyapkan otomatis oleh flag CI.
  Entri di sana **permanen**: wajib ditinjau ulang dan dihapus begitu upstream merilis patch
  (pelacak: TASK-035). Advisory **baru** yang punya perbaikan tetap memerahkan CI. Gate inilah yang
  menangkap `tinypool` (critical) di run CI pertama — perbaikannya naik `vitest` 3 → 4 (vitest 4 tidak
  lagi memakai `tinypool`).
- **Sisi Rust** memakai mekanisme yang sama: pengecualian `cargo audit` ditulis **eksplisit** di
  [`.cargo/audit.toml`](../../.cargo/audit.toml) (`[advisories] ignore`), dengan alasan tertulis +
  pelacak task di komentarnya. `cargo audit` membacanya otomatis saat dijalankan dari root repo
  (lokasi yang dipakai job CI). **Satu entri aktif** (ronde 28): `RUSTSEC-2026-0285` (`rustls`),
  dijelaskan di bawah §3a.
- **Job yang bergantung pada baseline**: `Dependency review (PR)` butuh dependency graph branch target;
  selama branch target belum punya manifest, job melewati dirinya sendiri dengan catatan di job summary
  (kegagalan struktural ≠ temuan keamanan).

### 3a. Pengecualian `RUSTSEC-2026-0285` (TASK-039, ronde 28)

`rustls 0.23.43` — TLS 1.3 handshake messages incorrectly accepted across encryption level
boundaries; perbaikan `>=0.23.45`. Dikecualikan karena **perbaikannya tidak bisa dipakai**, bukan
karena tidak ada patch:

- **Jangkauan**: hanya rantai **test** — `nearsea-market` → `[dev-dependency]` `near-workspaces` →
  `near-sandbox` → `ureq` → `rustls`. Kontrak produksi (`nearsea-nft-collection`, `nearsea-factory`)
  tidak menyentuhnya; `dev` sebelum TASK-006 bahkan belum punya `rustls`.
- **Blocker**: `rustls 0.23.45` menuntut `aws-lc-rs ^1.18`, sedangkan `near-crypto` (via `near-sdk`,
  fitur `unit-testing`) mem-pin `aws-lc-rs = "=1.16.2"` secara exact. Rilis `near-crypto` stabil
  terbaru masih `0.37.4`.
- **Jalan keluar bersih sudah diuji, tidak ada**: (a) `near-workspaces` dengan fitur `native-tls`
  tidak cukup — `rustls` tetap masuk lewat jalur terpisah `near-sandbox` → `ureq`; (b) `near-sandbox`
  dan `ureq` sudah versi terbaru yang kompatibel; (c) `near-crypto` hanya punya prerelease
  `0.38.0-rc.3`.
- **Keputusan user (2026-10-09)**: terima + catat eksplisit. **Hapus entri ini** begitu upstream
  melonggarkan pin `aws-lc-rs` sehingga `rustls >=0.23.45` bisa dipakai (pelacak: TASK-039).

## 4. Secrets di CI

- Build **tidak butuh** secret produksi (FACT) — hanya nilai publik (`NEXT_PUBLIC_*`).
- Secret yang dipakai CI: `DEPLOY_SSH_KEY` (GitHub Secret). Tidak ada secret lain.
- Secret tidak pernah ditulis di file workflow; selalu `${{ secrets.* }}`.
- Rotasi saat anggota keluar / dugaan kompromi (lihat [secrets-and-gitignore.md](./secrets-and-gitignore.md)).
- Larangan: `pull_request_target` dengan checkout PR (jalur injeksi secret).

## 5. Verifikasi pasca-deploy (kontrak)

- Simpan **code hash (sha256)** wasm sebagai artifact CI.
- Setelah deploy: `near view <contract> contract_source_metadata` → bandingkan hash
  dengan artifact. Cocok = build terbukti (SEC-CONTRACT-006).
- Deploy kontrak dianggap **gagal** bila hash tidak cocok.

## 6. Rollback

- Web: deploy ulang commit/tag sebelumnya lewat workflow (environment sama).
- Kontrak: redeploy wasm versi sebelumnya ke akun yang sama (state persist); bila ada
  perubahan storage layout → jalankan migrate yang sesuai
  (lihat [deployment.md](../deployment/deployment.md)).
- Setiap rollback dicatat di [CHANGELOG.md](../../CHANGELOG.md) (bagian artefak terkait).

## 7. Workflow YAML (referensi file nyata)

Enam file di `.github/workflows/` adalah **file nyata** (perintah sudah dijalankan & diverifikasi lokal saat TASK-001; `release.yml` ditambahkan TASK-032). Cuplikan kunci:

**ci.yml — job kontrak (potongan):**

```yaml
contracts:
  name: Contracts — fmt, clippy, test
  runs-on: ubuntu-latest
  timeout-minutes: 12
  steps:
    - uses: actions/checkout@<sha> # v4
    - uses: dtolnay/rust-toolchain@<sha> # stable
      with:
        # Versi toolchain TIDAK ditulis di sini — dibaca dari rust-toolchain.toml (SSOT pin).
        components: rustfmt, clippy
        targets: wasm32-unknown-unknown
    - uses: actions/cache@<sha> # v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: cargo-${{ hashFiles('**/Cargo.lock') }}
    - run: cargo fmt --all -- --check
    - run: cargo clippy --all-targets -- -D warnings
    # cargo-near: versi + sha256 di-pin (bukan `cargo install` tanpa pin)
    - run: |
        curl -fsSL -o cargo-near.tar.gz \
          "https://github.com/near/cargo-near/releases/download/cargo-near-v0.22.0/cargo-near-x86_64-unknown-linux-gnu.tar.gz"
        echo "<sha256>  cargo-near.tar.gz" | sha256sum --check --strict
        tar -xzf cargo-near.tar.gz
        install -m 0755 cargo-near-x86_64-unknown-linux-gnu/cargo-near "$HOME/.cargo/bin/cargo-near"
    # Build wasm SEBELUM test: suite sandbox (`market/tests/slice_sandbox.rs`) men-deploy
    # wasm nyata dari `target/near/<crate>/` ke sandbox chain (TASK-006).
    - run: |
        for crate in contract market factory; do
          cargo near build non-reproducible-wasm --manifest-path "$crate/Cargo.toml"
        done
    # Fixture TEST di luar daftar crate produksi: koleksi pihak ketiga "nakal" (TC-003).
    - run: |
        cargo near build non-reproducible-wasm --no-abi \
          --manifest-path market/tests/fixtures/rogue-collection/Cargo.toml
    # Unit + sandbox (Linux/macOS saja — binary sandbox nearcore tidak ada untuk Windows).
    - run: cargo test --workspace
    # Verifikasi metadata NEP-330 (versi + link) — lihat §14.
    - run: |
        set -euo pipefail
        for crate in contract market factory; do
          pkg=$(awk -F'"' '/^\[package\]/{p=1} p && /^name[[:space:]]*=/{print $2; exit}' "$crate/Cargo.toml")
          wasm="target/near/${pkg//-/_}/${pkg//-/_}.wasm"
          grep -a -o -E '\{"version":"[^"]+","link":"[^"]+",' "$wasm" >/dev/null
        done
    - run: |
        mkdir -p artifacts
        sha256sum target/near/*/*.wasm | tee artifacts/code-hash.txt
    - uses: actions/upload-artifact@<sha> # v4
      with:
        name: contracts-wasm
        retention-days: 90
        path: |
          target/near/*/*.wasm
          artifacts/code-hash.txt
```

- **Glob artifact = `target/near/*/*.wasm`** — `cargo-near` menaruh hasil di sub-folder per crate (`target/near/<crate>/<crate>.wasm`), bukan langsung di `target/near/`.
- Build di gate PR kini **dengan ABI** (TASK-032): ABI dibangkitkan & disematkan, dan metadata NEP-330 (versi + link) diverifikasi tiap PR. Build **reproducible** (container Docker ter-pin by digest) tidak dijalankan di sini — itu milik [release.yml](../../.github/workflows/release.yml), karena butuh Docker dan toolchain ter-pin. Catatan Windows lokal: langkah ABI butuh linking native, jadi gate ABI hanya jalan di CI/Linux.

**deploy-*.yml — pola SSH deploy (potongan):**

```yaml
- name: Setup SSH deploy key
  run: |
    mkdir -p ~/.ssh
    echo "${{ secrets.DEPLOY_SSH_KEY }}" > ~/.ssh/deploy_key
    chmod 600 ~/.ssh/deploy_key
    ssh-keyscan -H "${{ vars.DEV_HOST }}" >> ~/.ssh/known_hosts 2>/dev/null

- name: Deploy via SSH (docker compose pull/up)
  run: |
    ssh -i ~/.ssh/deploy_key "${{ vars.DEV_SSH_USER }}@${{ vars.DEV_HOST }}" \
      "cd ${{ vars.DEV_APP_DIR }} && docker compose pull && docker compose up -d"
```

- Sumber kebenaran isi workflow tetap file di `.github/workflows/`; cuplikan di sini **ilustratif** — bila berbeda, file yang benar (dan tabel §3/§11 disinkronkan).
- Semua action pihak ketiga **di-pin ke commit SHA** (bukan tag) — [cicd-security.md](../security/cicd-security.md) §5.
- **Status deploy-*.yml (TASK-001):** ketiganya `workflow_dispatch` **saja** + langkah "Cek prasyarat environment" yang gagal bila `*_HOST`/`*_SSH_USER`/`*_APP_DIR`/`*_APP_URL` kosong. Alasannya: environment (VPS + subdomain) belum di-provision (TASK-028) dan wiring SSH belum ada (TASK-029) — auto-deploy `push: branches: [dev]`/`[testnet]` baru diaktifkan di TASK-029. Dengan begitu tidak ada kredensial yang dibutuhkan dan tidak ada job merah yang menyesatkan.

## 8. Caching, concurrency, timeout, retry

| Aspek | Konfigurasi | Catatan |
|---|---|---|
| **Concurrency (CI)** | `group: ci-${{ github.ref }}`, `cancel-in-progress: true` | Push baru membatalkan run lama di branch yang sama. |
| **Concurrency (deploy)** | `group: deploy-<env>`, `cancel-in-progress: false` | Deploy **tidak** dibatalkan — mencegah state setengah jalan. |
| **Cache cargo** | `~/.cargo/registry`, `~/.cargo/git`, `target`; key `cargo-${{ hashFiles('**/Cargo.lock') }}` | Key berubah saat lockfile berubah. |
| **Cache pnpm** | `actions/setup-node` `cache: pnpm` + `cache-dependency-path: frontend/pnpm-lock.yaml` | Install wajib `--frozen-lockfile`. |
| **Timeout job** | `timeout-minutes`: kontrak 12, FE 6, API 5, fuzz-smoke 4, e2e 15 | Selaras anggaran di [testing-strategy.md](../testing/testing-strategy.md) §Anggaran waktu CI. |
| **Retry** | **Tidak ada retry otomatis** untuk test (flaky = bug, testing-strategy §Flaky). Retry **hanya** untuk langkah jaringan non-deterministik (SSH/`docker pull`), maks 2× dengan backoff. | Test yang di-retry menyembunyikan kegagalan nyata. |
| **Timeout perintah jaringan** | SSH `ConnectTimeout=10`, `curl --max-time 15` | Mencegah job menggantung sampai timeout job. |

```yaml
# Pola retry terbatas untuk langkah jaringan (bukan test)
- name: Deploy via SSH (retry jaringan saja)
  run: |
    for i in 1 2 3; do
      ssh -o ConnectTimeout=10 -i ~/.ssh/deploy_key \
        "${{ vars.DEV_SSH_USER }}@${{ vars.DEV_HOST }}" \
        "cd ${{ vars.DEV_APP_DIR }} && docker compose pull && docker compose up -d" && break
      echo "percobaan $i gagal, ulang dalam $((i*10)) dtk"; sleep $((i*10))
    done
```

## 9. Retensi artifact

| Artifact | Retensi | Alasan |
|---|---|---|
| `contracts-wasm` + `code-hash.txt` | **90 hari** | Cukup untuk verifikasi hash & rollback rilis terakhir. |
| FE build output (opsional) | 14 hari | Debug cepat; image Docker di registry = sumber rollback. |
| Laporan coverage / test | 14 hari | Jejak PR. |
| Log deploy | 90 hari | Audit rilis (incident-response). |
| Release artifact (tag rilis) | **permanen** | Disimpan di GitHub Release + registry image, bukan hanya Actions artifact. |

- `retention-days` diset eksplisit di `actions/upload-artifact`; default 90 hari boleh dipertahankan untuk wasm.
- Artifact rilis final (tag) **wajib** dilampirkan ke GitHub Release agar tidak hilang saat retensi Actions habis.
- Artifact tidak boleh memuat secret; sebelum upload, pastikan tidak ada `.env`/kredensial terikut.

## 10. Gate migration database

Perubahan schema **wajib** lewat gate ini sebelum deploy (detail: [migrations.md](../database/migrations.md) §6).

```text
[ ] 1. prisma migrate deploy   → apply semua migration ke DB ephemeral (fresh)
[ ] 2. up → down → up          → schema akhir identik baseline (pg_dump --schema-only)
[ ] 3. prisma migrate status   → tidak ada migration pending / drift
[ ] 4. lint SQL                → migration.sql + down.sql ada; tidak ada edit file lama (git diff)
[ ] 5. test GRANT/REVOKE       → role nearsea_app: UPDATE/DELETE admin_audit GAGAL (SEC-DB-003)
```

- Gate berjalan sebagai job CI (`api-test` / `migration-test`, PROPOSED) — PR yang mengubah `prisma/migrations/**` **wajib** memicunya.
- **Urutan deploy**: migration dijalankan **sebelum** container app baru di-`up` (backward-compatible: migration aditif lebih dulu). Untuk migration destruktif → wajib expand-contract (migrations.md §4) dan approval DBA/owner.
- Migration pada `mainnet` → approval eksplisit user + backup terverifikasi sebelum apply (migrations.md §8).
- Migration yang gagal apply = pipeline merah; deploy dibatalkan, **tidak** ada retry otomatis.

## 11. Smoke test (langkah persis)

Dijalankan setelah setiap deploy (dev/testnet/mainnet), sebelum langkah dianggap sukses:

```bash
# 1. Aplikasi hidup + API sehat
curl -fsS --max-time 15 "$APP_URL/api/health"          # harapan: 200 {"status":"ok"}

# 2. Halaman utama merender (bukan 5xx)
curl -fsS -o /dev/null -w '%{http_code}\n' --max-time 15 "$APP_URL/"   # harapan: 200

# 3. Header keamanan ada (SEC-FE-001)
curl -fsSI --max-time 15 "$APP_URL/" | grep -Ei 'content-security-policy|strict-transport-security|x-content-type-options'

# 4. Kontrak ter-deploy & versinya benar (kontrak; testnet/mainnet)
near view "$MARKET_CONTRACT_ID" contract_source_metadata   # versi + hash
near view "$MARKET_CONTRACT_ID" get_version                # bila method tersedia

# 5. Verifikasi hash artifact == on-chain (kontrak) — §14
sha256sum target/near/*/*.wasm | diff - artifacts/code-hash.txt
```

- Gagal pada langkah mana pun → deploy **gagal** → jalankan penanganan kegagalan (§12), jangan tandai sukses.
- Smoke test = subset minimal; uji end-to-end penuh (Playwright) tetap dijalankan terpisah (pra-rilis/nightly).
- `/api/health` **bukan** bagian versi `/api/v1` ([api-security-architecture.md](../security/api-security-architecture.md) §6a).

## 12. Penanganan kegagalan deploy

| Titik gagal | Tindakan | Rollback? |
|---|---|---|
| CI/Security merah (pra-merge) | Jangan merge; perbaiki di branch | — |
| Migration gagal apply | Batalkan deploy; DB belum berubah / sudah di-`down`; perbaiki migration baru | tidak perlu bila belum apply |
| SSH/`docker compose pull` gagal | Retry jaringan terbatas (§8); masih gagal → tandai deploy gagal | tidak (belum ada perubahan) |
| `docker compose up` gagal / container crash-loop | Hentikan; kembali ke image tag sebelumnya (§16) | **ya** |
| Smoke test gagal | Jalankan rollback otomatis (§16); investigasi | **ya** |
| Verifikasi hash kontrak gagal | **Anggap deploy gagal**; redeploy wasm versi terverifikasi | **ya** |
| Kegagalan setelah deploy sukses (bug produksi) | Incident-response; rollback bila perlu | **ya** |

- Setiap kegagalan deploy → **notifikasi** (§13) + catat di [incident-response.md](../security/incident-response.md) bila berdampak user.
- Kegagalan berulang pada environment yang sama → hentikan promosi, perbaiki di `dev` (jangan menumpuk retry).
- Deploy **mainnet** yang gagal tidak pernah diulang otomatis — keputusan manual user.

## 13. Notifikasi pipeline

| Kejadian | Kanal | Isi |
|---|---|---|
| Deploy sukses (dev/testnet) | GitHub Actions summary + notifikasi GitHub | env, versi/tag, commit SHA, URL |
| Deploy sukses (mainnet) | + Telegram admin | env, tag, hash kontrak, aktor |
| Deploy **gagal** (semua env) | Telegram admin (PROPOSED) | env, langkah gagal, run URL |
| Smoke test gagal | Telegram admin | endpoint gagal, HTTP code |
| Security scan gagal (gitleaks/audit) | notifikasi GitHub + Telegram security | jenis temuan, PR/run |
| Alert runtime (5xx, pause, owner-tx) | Telegram admin | [monitoring.md](../deployment/monitoring.md) |

```yaml
# Langkah notifikasi Telegram pada kegagalan (PROPOSED — TASK-029)
- name: Notify failure (Telegram)
  if: failure()
  env:
    TG_TOKEN: ${{ secrets.TELEGRAM_BOT_TOKEN }}
    TG_CHAT: ${{ secrets.TELEGRAM_CHAT_ID }}
  run: |
    curl -fsS --max-time 10 "https://api.telegram.org/bot${TG_TOKEN}/sendMessage" \
      -d chat_id="${TG_CHAT}" \
      -d text="Deploy gagal: ${GITHUB_WORKFLOW} @ ${GITHUB_REF_NAME} (run ${GITHUB_RUN_ID})"
```

- Secret `TELEGRAM_BOT_TOKEN`/`TELEGRAM_CHAT_ID` untuk CI ditambahkan sebagai **GitHub Secret** (bukan disalin ke file) — PROPOSED; bila ditolak, pakai notifikasi GitHub native + step summary.
- Notifikasi **tidak** memuat detail sensitif (token, isi `.env`, stack trace penuh) — hanya referensi run.
- Alert Telegram runtime (5xx/pause) dimiliki [monitoring.md](../deployment/monitoring.md); jangan menduplikasi wiring-nya di sini.

## 14. Reproducible build — cara hash dihitung & dibandingkan

**Kontrak** (SEC-CONTRACT-006, NEP-330):

```text
A. BUILD (lingkungan terkunci)
   1. Build reproducible: `cargo near build reproducible-wasm` — dijalankan DI DALAM
      container Docker yang di-pin by digest di [package.metadata.near.reproducible_build]
      (image = toolchain rilis; rust-toolchain.toml repo tidak dipakai).
   2. Hasil: artifacts/<crate>/*.wasm (--out-dir menyalin artifact final ke sana)
   3. HITUNG hash:  find artifacts -name '*.wasm' | xargs sha256sum → artifacts/code-hash.txt

B. BUKTI metadata tertanam (NEP-330)
   4. `contract_source_metadata` (string JSON tertanam di wasm) memuat: `version`,
      `link` (dari `package.repository`), `standards` (nep330), `build_info`.
      Ia TIDAK memuat hash — hash artifact dibandingkan terpisah (langkah 6–7).
   5. Diverifikasi otomatis sebelum artifact dipublikasikan: version == tag rilis,
      link == URL repo. Gagal → rilis digagalkan (release.yml).

C. BANDINGKAN dengan yang berjalan (post-deploy)
   6. near view <contract> contract_source_metadata → cocokkan `version`/`link` dengan tag;
      near state <contract> → `code_hash` dibandingkan dengan artifacts/code-hash.txt.
      COCOK    → provenance sah, deploy sukses.
      TIDAK    → deploy GAGAL; jangan promosikan; investigasi (artifact swap — cicd-security §3).
```

- [ci.yml](../../.github/workflows/ci.yml) membangun wasm **dengan ABI** dan memverifikasi
  metadata NEP-330 (versi + link) tiap PR. **Build reproducible penuh** (mode Docker, image
  ter-pin by digest) berjalan di [release.yml](../../.github/workflows/release.yml) saat tag rilis
  dibuat — di sana hash artifact dibandingkan persis dan metadata yang tertanam dibuktikan == tag
  (TASK-032).
- Verifikasi ulang independen (opsional, gate M4): build di Docker pinned + SourceScan, bandingkan hash dengan metadata on-chain.
- **Web/indexer**: hash = digest artifact build (image digest Docker). Dibandingkan `git rev-parse HEAD` dengan SHA yang tercatat di artifact/image label; bukan NEP-330.
- Perubahan toolchain (versi rust/near-sdk) **mengubah hash** meski sumber sama → catat versi toolchain di metadata/summary agar verifikasi bisa direproduksi.

## 15. Environment protection (setup GitHub)

| Environment | Required reviewers | Wait timer | Deployment branches | Secrets/vars |
|---|---|---|---|---|
| `dev` | tidak | — | `dev` | `DEV_*` vars |
| `testnet` | **1 reviewer** | — | `testnet` | `TESTNET_*` vars |
| `mainnet` | **required reviewers (≥1, `release-owner`)** | (opsional 5 menit) | `mainnet` saja | `MAINNET_*` vars |

Langkah konfigurasi (Settings → Environments):

```text
1. Buat environment `dev`, `testnet`, `mainnet`.
2. mainnet: centang "Required reviewers" → tambahkan release-owner.
   testnet: tambahkan 1 reviewer.
3. "Deployment branches": batasi tiap environment ke branch-nya (mainnet → mainnet saja).
4. Simpan vars per environment: *_APP_URL, *_HOST, *_SSH_USER, *_APP_DIR.
5. Secrets: DEPLOY_SSH_KEY (repo-level), TELEGRAM_* (bila notifikasi CI dipakai).
6. Uji: jalankan deploy-mainnet dengan konfirmasi salah → guard menolak (§16 workflow).
```

- Environment `mainnet` **wajib** punya required reviewers (deploy-mainnet.yml) — ini lapisan approval terakhir selain aturan tanya-user (git-workflow §3).
- Jangan menaruh IP/hostname nyata di dokumen; pakai vars environment (environments.md).
- Deployment branch restriction mencegah workflow deploy-mainnet dijalankan dari branch lain meski dipicu manual.

## 16. Otomasi rollback

Rollback web/indexer (image tag sebelumnya):

```bash
# Manual (break-glass, di VPS) — kembalikan ke tag rilis sebelumnya
ssh "$SSH_USER@$HOST" "cd $APP_DIR && IMAGE_TAG=web-v0.2.0 docker compose up -d web"
curl -fsS --max-time 15 "https://<env-url>/api/health"
```

```yaml
# .github/workflows/rollback.yml — PROPOSED (TASK-029)
on:
  workflow_dispatch:
    inputs:
      environment: { type: choice, options: [dev, testnet, mainnet], required: true }
      image_tag:   { description: 'Tag rilis sebelumnya (mis. web-v0.2.0)', required: true }
      confirm:     { description: 'Ketik ROLLBACK untuk konfirmasi', required: true }
jobs:
  rollback:
    environment: ${{ github.event.inputs.environment }}   # mainnet → butuh reviewer
    steps:
      - run: test "${{ github.event.inputs.confirm }}" = "ROLLBACK"
      - run: |
          ssh ... "cd $APP_DIR && IMAGE_TAG=${{ github.event.inputs.image_tag }} docker compose up -d"
      - run: curl -fsS "$APP_URL/api/health"
```

Aturan rollback:

- **Web/indexer**: deploy ulang **tag rilis sebelumnya** (image immutable) — bukan `git revert` untuk rollback cepat; `git revert` dipakai untuk memperbaiki branch (git-workflow §15).
- **Kontrak**: redeploy wasm versi sebelumnya ke akun yang sama (state persist); bila storage layout berubah → jalankan migrate yang sesuai. Rollback kontrak mainnet = keputusan eksplisit user.
- Rollback **otomatis** dipicu bila smoke test (§11) gagal pada deploy dev/testnet; mainnet = manual (tidak pernah otomatis).
- Setiap rollback dicatat di [CHANGELOG.md](../../CHANGELOG.md) (bagian artefak) + [incident-response.md](../security/incident-response.md) bila berdampak user.
- Setelah rollback: perbaiki di `dev`, lalu promosikan ulang lewat alur normal (jangan menambal langsung di branch atas).

## 17. Status

- Pipeline & workflow: **CI + Security AKTIF (TASK-001)** — `ci.yml` (fmt/clippy/test + build wasm + lint/format/typecheck/test/build FE) dan `security.yml` (gitleaks + audit + dependency review) sudah berisi perintah nyata dan dijalankan terhadap workspace yang ada. Deploy workflow masih **PROPOSED** (diaktifkan di TASK-029, bersama TASK-028).
- Strategi merge yang memengaruhi promosi: [git-workflow.md](./git-workflow.md) §9.
- Caching/timeout/retensi/gate migration/smoke test/notifikasi/rollback (§8–§16) — **DECIDED (ronde 15)**; nilai operasional (retensi, wait timer) boleh disetel saat scaffold.
- Reproducible build mode + verifikasi hash otomatis (§14) — **AKTIF (TASK-032)** di
  [release.yml](../../.github/workflows/release.yml) (dipicu tag rilis; bisa diuji-kering lewat
  `workflow_dispatch`). [ci.yml](../../.github/workflows/ci.yml) tetap memakai build cepat
  (host runner) untuk gate PR, kini **dengan ABI** + verifikasi metadata NEP-330.
- **Belum ada di CI (sengaja, jangan ditambahkan sebagai required check sebelum job-nya ada):** `API — test` (butuh API + DB, TASK-018), `Fuzz smoke` (butuh target `cargo-fuzz`, belum ada), `E2E — golden path` (TASK-008/010).
- **Urutan job kontrak (TASK-006, ronde 24):** `fmt` → `clippy` → build wasm (3 crate + fixture test) → `cargo test --workspace`. Build wasm **sebelum** test karena suite sandbox men-deploy artefak nyata dari `target/near/<crate>/`. Suite sandbox hanya berjalan di Linux/macOS (runner CI = `ubuntu-latest`); di Windows ia terkompilasi menjadi nol test lewat `#![cfg(unix)]`, dan dev-dependency `near-workspaces` di-scope `[target.'cfg(unix)'.dev-dependencies]` supaya gate lokal Windows tidak berubah.
