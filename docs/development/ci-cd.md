# CI/CD

> **Single source of truth untuk proses pipeline & deploy.** Aspek ancaman supply-chain
> ada di [cicd-security.md](../security/cicd-security.md) (dokumen itu merujuk ke sini untuk
> definisi pipeline). Aturan branch: [git-workflow.md](./git-workflow.md);
> versi & tag: [versioning-and-release.md](./versioning-and-release.md).

## 1. Peta branch → environment → deploy

| Branch | Environment | Trigger deploy | Approval | Artefak |
|---|---|---|---|---|
| `dev` | staging (RPC testnet, kontrak/subdomain dev terpisah) | otomatis saat push | tidak | web + kontrak dev |
| `testnet` | testnet publik | otomatis saat push (gate CI) | 1 (environment) | web + kontrak testnet |
| `mainnet` | produksi | **manual saja** (`workflow_dispatch`) | wajib (reviewer) | web + kontrak mainnet |

## 2. Tahapan pipeline

```text
1. PR OPEN        → CI + Security jalan (lihat .github/workflows/)
2. CI             → cargo fmt/clippy/test (sandbox) + lint/typecheck/test/build FE
                    + cargo audit / npm audit + build wasm + upload artifact + code hash
3. SECURITY       → secret scanning (gitleaks) + dependency review + audit dependensi
4. MERGE          → hanya jika semua check hijau + approval cukup
5. DEPLOY         → workflow per-branch (SSH ke VPS → docker compose pull/up)
6. POST-DEPLOY    → verifikasi hash kontrak (NEP-330) + smoke test /api/health
7. ROLLBACK       → revert merge commit di branch target + deploy ulang (jika perlu)
```

## 3. Gate (wajib lulus sebelum merge/deploy)

| Gate | Berlaku | Gagal → |
|---|---|---|
| Format + clippy tanpa warning | kontrak | blok merge |
| Unit + sandbox test hijau | kontrak | blok merge |
| Lint + typecheck + build | FE | blok merge |
| `cargo audit` / `npm audit` (critical/high) | semua | blok merge |
| Secret scanning (gitleaks) | semua | blok merge |
| Reproducible build + verifikasi hash | kontrak (testnet/mainnet) | blok deploy |
| Audit eksternal lulus | kontrak (mainnet, TASK-027) | blok deploy |

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

Lima file di `.github/workflows/` adalah **kerangka final** (isi perintah sudah nyata; job build/test akan hijau begitu kode ada — TASK-001). Cuplikan kunci:

**ci.yml — job kontrak (potongan):**

```yaml
contracts:
  name: Contracts — fmt, clippy, test
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
        targets: wasm32-unknown-unknown
    - uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: cargo-${{ hashFiles('**/Cargo.lock') }}
    - run: cargo fmt --all -- --check
    - run: cargo clippy --all-targets -- -D warnings
    - run: cargo test --workspace
    - run: cargo near build non-reproducible-wasm
    - run: |
        mkdir -p artifacts
        sha256sum target/near/*.wasm | tee artifacts/code-hash.txt
    - uses: actions/upload-artifact@v4
      with:
        name: contracts-wasm
        path: |
          target/near/*.wasm
          artifacts/code-hash.txt
```

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
sha256sum target/near/*.wasm | diff - artifacts/code-hash.txt
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
   1. Build reproducible: `cargo near build` (mode reproducible) di runner dengan toolchain
      di-pin (versi rust + wasm-opt + near-sdk identik dengan rilis).
   2. Hasil: target/near/<contract>.wasm
   3. HITUNG hash:  sha256sum target/near/<contract>.wasm  → simpan sebagai artifact
                    (mis. artifacts/code-hash.txt)

B. PUBLISH metadata on-chain (NEP-330)
   4. `contract_source_metadata` memuat: version, source URL + commit, build hash.
   5. Deploy wasm; metadata ikut terpasang.

C. BANDINGKAN (post-deploy, otomatis di workflow)
   6. near view <contract> contract_source_metadata  → ambil `build_hash` / hash yang dipublikasikan
   7. Bandingkan dengan artifacts/code-hash.txt dari CI.
      COCOK    → provenance sah, deploy sukses.
      TIDAK    → deploy GAGAL; jangan promosikan; investigasi (artifact swap — cicd-security §3).
```

- Skeleton [ci.yml](../../.github/workflows/ci.yml) saat ini memakai `cargo near build non-reproducible-wasm` (cukup untuk uji); **mode reproducible** diaktifkan saat rilis kontrak pertama (TASK-032) dan hasil hash dibandingkan persis.
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

- Pipeline & workflow: **PROPOSED** — diimplementasi saat scaffold repo (TASK-001) dan infra CI/CD (TASK-029). File workflow di `.github/workflows/` sudah disiapkan sebagai kerangka.
- Strategi merge yang memengaruhi promosi: [git-workflow.md](./git-workflow.md) §9.
- Caching/timeout/retensi/gate migration/smoke test/notifikasi/rollback (§8–§16) — **DECIDED (ronde 15)**; nilai operasional (retensi, wait timer) boleh disetel saat scaffold.
- Reproducible build mode + verifikasi hash otomatis (§14) — **PROPOSED** (aktif saat rilis kontrak pertama, TASK-032).
