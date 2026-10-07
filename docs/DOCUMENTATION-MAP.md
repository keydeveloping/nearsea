# Documentation Map — Peta Sinkronisasi Dokumen

> Diperbarui setiap kali struktur dokumen berubah. Dirujuk dari [AGENTS.md](../AGENTS.md) § Documentation Sync Rules.
> Tujuan: semua dokumen tetap **sync dan saling berhubungan** — tidak ada nilai yang bertentangan antar dokumen.

---

## 1. Single Source of Truth (siapa memegang fakta apa)

| Fakta | Sumber kebenaran | Dokumen yang hanya boleh MERUJUK |
|---|---|---|
| Nama produk | [README.md](../README.md) | semua dokumen lain |
| Scope fitur MVP & Non-Goals | [01-PRD.md](./01-PRD.md) §8 & §4 | 00-overview, milestones, backlog, features/* |
| Business rules (fee 2%, royalti, larangan) | [01-PRD.md](./01-PRD.md) §13 | payments.md, marketplace.md, test-cases |
| Keputusan arsitektur + alasannya | [decisions/ADR-*.md](./decisions/ADR-001.md) | architecture/*, features/*, README |
| Tech stack final | [architecture/tech-stack.md](./architecture/tech-stack.md) | README, system-architecture, AGENTS |
| Alur & method kontrak | [features/marketplace.md](./features/marketplace.md) + [RESEARCH.md](../RESEARCH.md) | 02-requirements, user-flows, test-cases |
| Reference implementasi kontrak (signature, init args, layout state, prefix storage, konstanta gas, view shapes, pemetaan method→event→error) | [contracts/nft-collection.md](./contracts/nft-collection.md) · [contracts/market.md](./contracts/market.md) · [contracts/factory.md](./contracts/factory.md) | features/*, security/smart-contract-* (perilaku tetap milik features/) |
| Katalog event NEP-297 + payload | [api/webhooks.md](./api/webhooks.md) | features/*, security/*, indexer docs |
| Schema DB (report/profil/audit MVP + proyeksi fase 2) | [database/database-schema.md](./database/database-schema.md) | data-model, api/* |
| Kontrak API REST (report dsb.) | [api/endpoints.md](./api/endpoints.md) | features/* terkait |
| Roles & permissions | [security/permissions.md](./security/permissions.md) | 02-requirements, features/* |
| Prioritas task | [tasks/backlog.md](../tasks/backlog.md) (SSOT prioritas) · Milestone | [tasks/milestones.md](../tasks/milestones.md) (SSOT milestone) | 01-PRD §17, implementation-plan |
| Riset teknis NEAR | [RESEARCH.md](../RESEARCH.md) | semua (jangan tulis ulang fakta riset di docs lain) |
| Security requirements (SEC-*) | [security/security-requirements.md](./security/security-requirements.md) | semua dokumen keamanan merujuk ID, tidak menyalin |
| Keputusan keamanan ADR-010..015 | [decisions/](./decisions/ADR-010-security-boundary.md) (folder decisions/) | architecture/*, features/* |
| Model branch & alur promosi git | [development/git-workflow.md](./development/git-workflow.md) | AGENTS.md, ci-cd, versioning-and-release |
| Skema versi & rilis | [development/versioning-and-release.md](./development/versioning-and-release.md) | CHANGELOG.md, ci-cd, README |
| Definisi pipeline CI/CD | [development/ci-cd.md](./development/ci-cd.md) | cicd-security.md, deployment.md, environments.md |
| Taksonomi error & kebijakan notifikasi | [development/error-handling.md](./development/error-handling.md) | frontend-architecture, api/*, features/* (rujuk kode, tidak menyalin) |
| Jaminan concurrency/race order | [development/concurrency-and-races.md](./development/concurrency-and-races.md) | order-protocol-security, threat-model, test-cases |
| Kebijakan secret & higienitas file | [development/secrets-and-gitignore.md](./development/secrets-and-gitignore.md) | .gitignore, key-management, cicd-security, environments |
| Rencana scaling | [architecture/scaling.md](./architecture/scaling.md) | infrastructure, backend-architecture, tech-stack |

---

## 2. Sync Trigger Matrix (perubahan apa → cek apa)

| # | Jika mengubah… | Wajib cek & update |
|---|---|---|
| T1 | Nama produk / branding | README, 00-overview, ADR terkait, semua file yang menyebut nama lama (grep) |
| T2 | Scope MVP (fitur masuk/keluar) | 01-PRD §8+§4, 02-product-requirements, 03-user-flows, features/ terkait, backlog, milestones, ADR |
| T3 | Business rule (fee, royalti, expire, batas) | 01-PRD §13, features/payments.md, features/marketplace.md, test-cases, threat-model |
| T4 | Keputusan arsitektur | ADR (buat baru / update status), system-architecture, tech-stack, README (tech stack), backlog |
| T5 | Method/argumen kontrak (on-chain API) | features/marketplace.md, features/payments.md, test-cases, security, threat-model |
| T6 | Schema/data model DB | database-schema, data-model, migrations.md (log), api/endpoints |
| T7 | Endpoint REST baru/berubah | api/endpoints, api-overview, features/ pemakainya |
| T8 | Halaman/flow UI baru/berubah | 04-ux-ui-spec, 03-user-flows, frontend-architecture (routing), test-cases (e2e) |
| T9 | Role/permission | security/permissions, 02-requirements (ADMIN), threat-model |
| T10 | Tooling/testing/deploy proses | testing-strategy, deployment/*, environments |
| T11 | Temuan riset NEAR baru (standard, tool, harga gas) | RESEARCH.md (tambah/ubah bagian), lalu T4 bila memengaruhi keputusan |
| T12 | Status pekerjaan (task selesai/milestone) | backlog, milestones, 00-overview (Status), implementation-plan |
| T13 | Security requirement (SEC-*) berubah status | security-requirements.md (register), security-roadmap, task terkait di backlog, ADR jika jadi keputusan |
| T14 | Branch model / proses promosi / proteksi branch | development/git-workflow.md, AGENTS.md (Git Rules), ci-cd.md, cicd-security.md |
| T15 | Versi/skema rilis/tag berubah | development/versioning-and-release.md, CHANGELOG.md, README, ci-cd.md |
| T16 | Pipeline/gate CI/CD berubah | development/ci-cd.md, cicd-security.md, deployment.md, .github/workflows/* |
| T17 | Kode error / pesan notifikasi berubah | development/error-handling.md (registry), features/* (kasus per fitur), api/endpoints.md |
| T18 | Environment baru/berubah (dev/testnet/mainnet) atau env var | deployment/environments.md, .env.example, ci-cd.md, scaling.md |
| T19 | Kebijakan secret / file sensitif / .gitignore | development/secrets-and-gitignore.md, .gitignore, key-management.md, cicd-security.md |
| T20 | Kapasitas/scaling/arsitektur infra berubah | architecture/scaling.md, infrastructure.md, backend-architecture.md |

## Dependency chain dokumentasi (urutan keputusan)

```text
PRD → Product Requirements → User Flows → Architecture → Security Requirements
   → Threat Model → Protocol Design (order/signature/auth) → Database/API Design
   → Testing Strategy → Implementation Plan → Implementation
```
Keputusan keamanan (ADR-010..015) mengikat architecture & protocol design di chain ini.

---

## 3. Checklist Crosscheck (jalankan SEBELUM dinyatakan selesai)

```text
[ ] 1. Faktanya diubah di dokumen PEMILIKnya dulu (bukan di dokumen perujuk)
[ ] 2. Baris terkait di Sync Trigger Matrix di atas sudah dilalui
[ ] 3. Grep istilah/angka lama di seluruh repo → 0 hasil tersisa
      contoh: grep -ri "fee 5%" . ; grep -ri "nama-lama" .
[ ] 4. Semua link markdown dalam file yang diedit masih valid (path benar)
[ ] 5. docs/00-project-overview.md § Status diperbarui
[ ] 6. Perubahan keputusan besar → ADR dibuat/di-update (status: Accepted/Superseded)
[ ] 7. Task baru muncul? → backlog; task jadi tidak relevan? → beri catatan, jangan hapus diam-diam
[ ] 8. Tidak ada dua dokumen yang menyatakan nilai berbeda untuk fakta yang sama
```

---

## 4. Inventaris Dokumen (indeks lengkap + tautan)

> Setiap file punya tautan markdown agar seluruh dokumen saling terhubung dari satu halaman ini.

**Root**

- [README.md](../README.md) — pintu masuk: nama, deskripsi, stack ringkas, link semua docs
- [AGENTS.md](../AGENTS.md) — aturan kerja agent (RULES), termasuk aturan sinkronisasi ini
- [CONTEXT.md](../CONTEXT.md) — glosarium istilah domain (bukan spesifikasi)
- [RESEARCH.md](../RESEARCH.md) — riset NEAR: standar, tutorial, market-contract, peta fitur OpenSea
- [CHANGELOG.md](../CHANGELOG.md) — catatan perubahan (Keep a Changelog, per-artefak)
- [LICENSE](../LICENSE) — lisensi MIT (diputuskan ronde 17; ADR-016)
- [rust-toolchain.toml](../rust-toolchain.toml) — pin toolchain Rust (SSOT versi Rust; 1.93.1)
- [rustfmt.toml](../rustfmt.toml) — konfigurasi `cargo fmt` (edition; opsi nightly sengaja tidak dipasang)
- [.nvmrc](../.nvmrc) — pin versi Node (SSOT versi Node; 24 LTS)
- [Cargo.toml](../Cargo.toml) — workspace kontrak (anggota `contract/`/`market/`/`factory/`) + profil rilis
- [Cargo.lock](../Cargo.lock) — lockfile workspace (WAJIB di-commit; SEC-CICD-001)
- [.gitleaks.toml](../.gitleaks.toml) — konfigurasi secret scan (dipakai CI + lokal)
- [.gitignore](../.gitignore) — daftar file sensitif/generated yang tidak masuk git (ketat)
- [.env.example](../.env.example) — template variabel environment (placeholder saja)
- [.gitattributes](../.gitattributes) — normalisasi line ending & tanda file biner
- [.editorconfig](../.editorconfig) — konsistensi indentasi/encoding lintas editor
- `.github/` — [CODEOWNERS](../.github/CODEOWNERS) (reviewer otomatis per area), [dependabot.yml](../.github/dependabot.yml) (PR dependency → `dev`), dan `workflows/`: [ci.yml](../.github/workflows/ci.yml), [security.yml](../.github/workflows/security.yml), [deploy-dev.yml](../.github/workflows/deploy-dev.yml), [deploy-testnet.yml](../.github/workflows/deploy-testnet.yml), [deploy-mainnet.yml](../.github/workflows/deploy-mainnet.yml)

**Kode** (dibuat TASK-001; bukan dokumen — terdaftar agar tidak jadi folder yatim)

- `contract/` — kontrak NFT koleksi (TASK-002..003); `market/` — kontrak market (TASK-004..005); `factory/` — kontrak factory (TASK-012). Struktur internal: [contracts/](./contracts/nft-collection.md)
- `frontend/` — Next.js App Router (TASK-007..008). Struktur internal: [frontend-architecture.md](./architecture/frontend-architecture.md) §1

**docs/development/** (proses kerja)

- [git-workflow.md](./development/git-workflow.md) — 3 branch, alur promosi, aturan merge
- [versioning-and-release.md](./development/versioning-and-release.md) — SemVer per-artefak, tag, checklist rilis
- [ci-cd.md](./development/ci-cd.md) — pipeline, gate, deploy, rollback
- [code-standards.md](./development/code-standards.md) — konvensi kode & kebijakan komentar
- [error-handling.md](./development/error-handling.md) — taksonomi error & kebijakan notifikasi
- [concurrency-and-races.md](./development/concurrency-and-races.md) — jaminan race (20 pembeli 1 NFT)
- [secrets-and-gitignore.md](./development/secrets-and-gitignore.md) — kebijakan secret & higienitas file

**docs/agents/** (konfigurasi konsumsi skill engineering — bukan spesifikasi produk)

- [domain.md](./agents/domain.md) — cara skill membaca domain docs: lokasi `CONTEXT.md`, rumah ADR (`docs/decisions/`, bukan `docs/adr/`), aturan kosakata & penandaan konflik ADR
- [issue-tracker.md](./agents/issue-tracker.md) — tracker markdown lokal: `tasks/backlog.md` (SSOT) + `.scratch/<fitur>/issues/`; konvensi wayfinding
- [triage-labels.md](./agents/triage-labels.md) — lima peran triage → label repo; pemisahan "progres" vs "triage"

**docs/research/** (bukti riset — bukan keputusan)

- [near-nft-market-2026.md](./research/near-nft-market-2026.md) — realita pasar NFT NEAR 2026 (marketplace yang mati, ukuran pasar, perang royalti) — **bertentangan dengan premis 01-PRD §2**

**docs/ — produk**

- [00-project-overview.md](./00-project-overview.md) — ringkasan 1 halaman + status proyek terkini
- [01-PRD.md](./01-PRD.md) — produk: goals, non-goals, features, business rules
- [02-product-requirements.md](./02-product-requirements.md) — behavior spec detail per kemampuan
- [03-user-flows.md](./03-user-flows.md) — perjalanan user per flow
- [04-ux-ui-spec.md](./04-ux-ui-spec.md) — halaman, komponen, state, responsive

**docs/architecture/**

- [system-architecture.md](./architecture/system-architecture.md) — komponen & alur end-to-end
- [frontend-architecture.md](./architecture/frontend-architecture.md) — struktur FE, routing, state
- [backend-architecture.md](./architecture/backend-architecture.md) — API/report service & auth
- [infrastructure.md](./architecture/infrastructure.md) — VPS, RPC, backup, ownership
- [scaling.md](./architecture/scaling.md) — rencana horizontal & vertical scaling
- [tech-stack.md](./architecture/tech-stack.md) — pilihan teknologi final

**docs/database/** (indexer/report — fase 2 + report MVP)

- [database-schema.md](./database/database-schema.md) — kelas tabel & kolom
- [data-model.md](./database/data-model.md) — entitas & relasi
- [migrations.md](./database/migrations.md) — alur Prisma migrate
- [seed-data.md](./database/seed-data.md) — data seed testnet

**docs/api/**

- [api-overview.md](./api/api-overview.md) — prinsip & batas API
- [authentication.md](./api/authentication.md) — NEP-413 + fallback
- [endpoints.md](./api/endpoints.md) — kontrak endpoint
- [webhooks.md](./api/webhooks.md) — event & payload

**docs/features/**

- [auth.md](./features/auth.md) — connect & login
- [users.md](./features/users.md) — profil & settings
- [marketplace.md](./features/marketplace.md) — listing/buy/offer/private/bundle
- [payments.md](./features/payments.md) — settlement, fee, royalti
- [launchpad.md](./features/launchpad.md) — create collection + mint per phase + allowlist
- [report.md](./features/report.md) — report/moderasi/badge/blocklist
- [notifications.md](./features/notifications.md) — notifikasi in-app

**docs/contracts/** (reference implementasi kontrak)

- [nft-collection.md](./contracts/nft-collection.md) — NEP-171/177/178/181/199 + nft_mint launchpad + royalti + layout
- [market.md](./contracts/market.md) — listing/offer/bundle + views + konstanta gas + prefix storage
- [factory.md](./contracts/factory.md) — deploy koleksi + wiring + allowlist upload

**docs/security/**

- [security.md](./security/security.md) — gambaran & checklist
- [permissions.md](./security/permissions.md) — role & alias
- [threat-model.md](./security/threat-model.md) — ancaman & mitigasi
- [access-control-matrix.md](./security/access-control-matrix.md) — matriks akses
- [asset-inventory.md](./security/asset-inventory.md) — aset yang dilindungi
- [trust-boundaries.md](./security/trust-boundaries.md) — batas kepercayaan
- [catastrophic-failure-scenarios.md](./security/catastrophic-failure-scenarios.md) — skenario terburuk
- [wallet-authentication.md](./security/wallet-authentication.md) — auth wallet
- [signature-architecture.md](./security/signature-architecture.md) — arsitektur tanda tangan
- [order-protocol-security.md](./security/order-protocol-security.md) — lifecycle order & transisi
- [smart-contract-security-architecture.md](./security/smart-contract-security-architecture.md) — keamanan kontrak
- [smart-contract-invariants.md](./security/smart-contract-invariants.md) — INV-001..030
- [key-management.md](./security/key-management.md) — pengelolaan kunci
- [api-security-architecture.md](./security/api-security-architecture.md) — keamanan API
- [metadata-security.md](./security/metadata-security.md) — keamanan metadata/IPFS
- [indexer-security.md](./security/indexer-security.md) — keamanan indexer
- [frontend-security.md](./security/frontend-security.md) — CSP & keamanan FE
- [database-security.md](./security/database-security.md) — keamanan DB
- [fraud-and-abuse.md](./security/fraud-and-abuse.md) — fraud & penyalahgunaan
- [admin-security.md](./security/admin-security.md) — keamanan admin
- [infrastructure-security.md](./security/infrastructure-security.md) — keamanan infra
- [cicd-security.md](./security/cicd-security.md) — keamanan pipeline
- [incident-response.md](./security/incident-response.md) — respons insiden
- [security-requirements.md](./security/security-requirements.md) — register SEC-*
- [security-roadmap.md](./security/security-roadmap.md) — peta jalan keamanan
- [security-gap-analysis.md](./security/security-gap-analysis.md) — analisis gap (G1..G14)

**docs/testing/**

- [testing-strategy.md](./testing/testing-strategy.md) — strategi & DoD
- [test-cases.md](./testing/test-cases.md) — TC-*
- [acceptance-criteria.md](./testing/acceptance-criteria.md) — AC per fitur

**docs/deployment/**

- [environments.md](./deployment/environments.md) — environment & env vars
- [deployment.md](./deployment/deployment.md) — langkah rilis
- [monitoring.md](./deployment/monitoring.md) — alert & observability
- [disaster-recovery.md](./deployment/disaster-recovery.md) — DR & backup

**docs/decisions/**

- [ADR-001.md](./decisions/ADR-001.md) — log keputusan ronde 1–13 + indeks semua ADR
- [ADR-002-listing-two-tx.md](./decisions/ADR-002-listing-two-tx.md) — pola listing 2-tx + dual verification
- [ADR-003-single-market-contract.md](./decisions/ADR-003-single-market-contract.md) — satu kontrak market gabungan
- [ADR-004-mvp-data-layer.md](./decisions/ADR-004-mvp-data-layer.md) — data layer MVP (RPC + NearBlocks, tanpa indexer)
- [ADR-005-platform-fee.md](./decisions/ADR-005-platform-fee.md) — fee 2% dipotong on-chain
- [ADR-006-open-collections-factory.md](./decisions/ADR-006-open-collections-factory.md) — factory open + badge + report
- [ADR-007-approval-listing-model.md](./decisions/ADR-007-approval-listing-model.md) — listing approval non-custodial + auto-stale
- [ADR-008-launchpad-phases.md](./decisions/ADR-008-launchpad-phases.md) — launchpad berphase + allowlist on-chain
- [ADR-009-self-hosted-vps.md](./decisions/ADR-009-self-hosted-vps.md) — infra VPS self-hosted
- [ADR-010-security-boundary.md](./decisions/ADR-010-security-boundary.md) — chain-centric boundary
- [ADR-011-wallet-authentication.md](./decisions/ADR-011-wallet-authentication.md) — NEP-413 auth
- [ADR-012-order-settlement-authority.md](./decisions/ADR-012-order-settlement-authority.md) — orderbook on-chain
- [ADR-013-key-management.md](./decisions/ADR-013-key-management.md) — key management (Sputnik DAO)
- [ADR-014-metadata-isolation.md](./decisions/ADR-014-metadata-isolation.md) — metadata isolation
- [ADR-015-indexer-consistency.md](./decisions/ADR-015-indexer-consistency.md) — indexer Neardata
- [ADR-016-project-intent-and-scope-cut.md](./decisions/ADR-016-project-intent-and-scope-cut.md) — **niat proyek (pembelajaran/portofolio) + M1 dipotong jadi vertical slice**

**tasks/**

- [backlog.md](../tasks/backlog.md) — semua task + prioritas
- [milestones.md](../tasks/milestones.md) — M0–M4
- [implementation-plan.md](../tasks/implementation-plan.md) — urutan kerja per fase

---

## 5. Riwayat Sinkronisasi

> Catatan tiap kali crosscheck besar dilakukan (bukan tiap edit kecil):

| Tanggal | Pemicu | Dokumen yang disinkronkan |
|---|---|---|
| 2026-10-01 | Ronde 1–3 tanya-jawab | 01-PRD, 02-requirements, 04-ux-ui-spec, features/{marketplace,payments}, architecture/{system,tech-stack,backend}, backlog, milestones, implementation-plan, ADR-001, README, 00-overview |
| 2026-10-01 | Ronde 4–6 tanya-jawab | 01-PRD §13, features/{marketplace,notifications}, architecture/{tech-stack,backend,infrastructure}, deployment/{deployment,disaster-recovery}, 02-requirements, backlog (TASK-012/015), milestones, implementation-plan, ADR-001, 00-overview |
| 2026-10-01 | Ronde 7–12 + pengisian penuh docs | hampir semua dokumen: README, 00-overview, 01-PRD (§1–11,13), 02-requirements (semua behavior), 03-user-flows (semua flow), 04-ux, architecture/{frontend,infrastructure,tech-stack}, api/{overview,authentication,endpoints,webhooks}, features/{auth,users,marketplace,payments,notifications}, security/{security,permissions,threat-model}, testing/{strategy,test-cases}, deployment/{deployment,monitoring,disaster-recovery}, database/seed-data, backlog (TASK-020..025), milestones, implementation-plan, ADR-001 (ADR-007..009), AGENTS |
| 2026-10-01 | Ronde 13 + deep check 42 file | 02-requirements (hapus duplikat), testing-strategy (hapus duplikat + skenario baru), 00-overview, features/{marketplace,notifications}, api/{overview,webhooks}, architecture/{system,backend,infrastructure,tech-stack}, 01-PRD (§12-13), database/{data-model,schema,migrations,seed-data}, deployment/{deployment,disaster-recovery}, security/{security,threat-model}, testing/acceptance-criteria (isi penuh), README, implementation-plan, ADR-001 (ronde 13) |
| 2026-10-01 | Security research pre-implementation (master prompt) | +22 docs baru di security/, +ADR-010..015, threat-model rewrite, permissions pointer, disaster-recovery (+security recovery), 01-PRD §10 (koreksi escrow), RESEARCH.md §12, backlog (TASK-026/027), AGENTS.md |
| 2026-10-01 | Riset & koreksi NEAR-compliance (NEP-199, finality, NEP-413 Final, Sputnik DAO, Lake deprecated→Neardata, NEP-330) | RESEARCH.md (standar+§10.5), AGENTS/README/tech-stack/00-overview (NEP list), indexer-security (finality+tooling), wallet-authentication+ADR-011 (NEP-413), key-management+ADR-013 (Sputnik DAO), smart-contract-arch+invariants (migrate, NEP-330, tooling), infrastructure (RPC list), security-requirements (status), gap-analysis (G1-G16), ADR-015 |
| 2026-10-01 | Crosscheck penuh 2 (71 file) | bersihkan 11 baris RESEARCH REQUIRED kedaluwarsa (signature-architecture, contract-arch, indexer-security, asset-inventory, security-requirements SEC-KEY-002, security-roadmap Phase B, catastrophic-scenarios ×3, incident-response ×2, trust-boundaries, key-management); log ADR-001 +ADR-010..015; verifikasi: 0 TODO asli, 0 NEP-list lama, 0 Vercel/Supabase sisa |
| 2026-10-01 | **Review subagent 71 file (11 agent) → AUDIT-TEMUAN.md → eksekusi semua perbaikan** | ~45 file: P0×6 (system-architecture 2-tx, api/authentication NEP-413, api/endpoints profil/launchpad/bundle, ADR-011/013 internal, implementation-plan fase); gap cakupan (schema launchpad/bundle/audit/nonce, data-model, users+profil, flows baru, threat-model, INV-023..027, testing×3); desain kontrak (guardian pause, nft_revoke_token, kanonisasi method, bundle rollback); stale/factual (README fee OpenSea, RESEARCH renumber, tech-stack, environments, DR, CS-9, fraud INV-023, CSP FE, fastnear endpoint, NEP-177); sistemik (backlog TASK-028..030, milestones M4 gate, monitoring pause-alert, deployment NEP-330+DAO, SEC-AUTH-006, ADR log) |
| 2026-10-02 | **Verifikasi ulang 12 agent (4 gelombang) atas 72 file + perbaikan ronde-2** | ~40 file: G7 sinkron ×5 (incident-response, CS-2, gap-analysis, RESEARCH, 00-overview); payments per-token (INV-027); **INV-025 bundle di-redesign jujur** (pre-validasi + kompensasi G7 — NEAR tidak punya rollback lintas-receipt) + INV-028..030 (bundle/launchpad/min-price); order-protocol lifecycle PARTIAL + agregasi payout bundle; nonce auth (validasi dulu, konsumsi belakangan); kanonisasi `offer`→`buy`/`buy_bundle` (marketplace, system-arch, INV-023); guardian `pause_callers` propagasi (threat-model, PRD, security, contract-arch, matrix, INV-022); PRD §8 duplikat hapus; user-flows settle order + bundle max; 04-ux self-buy; webhooks duplikat hapus + EVENT_JSON; auth template Network (api+wallet); endpoints template + step-up; database-schema kelas (accounts/sessions/blocklisted/allocation/★fase2); seed-data testnet+profil; data-model link; metadata SSRF/data-uri; FE CSP style-src/form-action; indexer urutan failover; DB path; fraud G11; admin "faktor"; infra A2; requirements 012/002 guardian + rujukan nama file; roadmap 4 auditor; ADR-001 log pasca-audit + ADR-003 + ADR-012/013/015/010; env vars backup; deployment link RESEARCH + timelock; monitoring heading; DR status; backlog Spec presisi + P0 def; milestones done-when M1; plan TASK-012/022/028-030 |
| 2026-10-02 | **Crosscheck ronde-3 (per file, verifikasi otomatis)** | DOCUMENTATION-MAP §4 diubah dari pohon teks → **indeks bertaut lengkap** (70 file, 0 orphan — sebelumnya 22 file tanpa tautan markdown); tech-stack status IPFS "TODO" → "⏳ open-by-design" (selaras teks). Verifikasi repo: 0 tautan rusak, 0 orphan, 0 heading duplikat, 0 heading nomor tak berurutan, 123/123 ID (INV/TC/TASK/SEC) terdefinisi, fee 2% & royalti cap 10% konsisten di 20+ file, 0 istilah usang (Vercel/Supabase/NEAR Lake/2,5%/mainnet.fastnear.com) |
| 2026-10-02 | **Ronde 15 Fase 2–5 — ekspansi semua file ke engineering spec** | **Fase 2** (architecture/database/api): system-architecture (C4 + sequence + gas budget), backend-architecture (middleware + Docker + observability), frontend-architecture (route table + query keys + perf budget), infrastructure (env matrix + firewall + DNS + cost), tech-stack (version pins + rationale), scaling (kapasitas + SLO + PgBouncer), database-schema (DDL nyata + enum + ERD + partisi + REVOKE), data-model (ERD formal + writer ownership + PayoutSplit), migrations (up/down + expand-contract + backfill), seed-data (dataset eksak + assertion), api-overview (versioning + CORS + ETag), authentication (worked JSON + pseudocode + JWT claims + test vectors), endpoints (skema per endpoint + enum + rate limit), webhooks (payload per event + HMAC + DLQ). **Fase 3** (features/deployment/testing): marketplace (skema argumen per method + event + storage + gas + edge case), payments (algoritma payout + contoh angka + merge bundle), notifications (payload + localStorage + dedup), auth/users (wallet + settings + identicon), deployment (runbook + rollback + smoke test), environments (nilai per env + bootstrap), monitoring (SLO + alert rule + Telegram + `/api/health` kanonik), disaster-recovery (restore runbook + RPO/RTO), testing-strategy (coverage + fixture + fuzz harness), test-cases (TC-019..TC-042), acceptance-criteria (AC-ID 83). **Fase 4** (security, 26 file): security (index + threat→control), permissions (kemampuan per fase), threat-model (L/I + attack tree AT-1..3), access-control-matrix (RACI + break-glass), asset-inventory (custodian + CIA), trust-boundaries (attack tree + asersi AS-1..24), signature-architecture (byte-level + revokasi), wallet-authentication (§6-§12 payload/pseudocode/session store/ml-dsa), order-protocol-security (contoh numerik + gas + attack tree), smart-contract-* (akses per method + peta INV→test→SEC), key-management (§5-§10 ceremony/rotasi/Shamir/timeline), api-security (inventaris + token bucket), metadata-security (sha256 + limit + allowlist), indexer-security (rekonsiliasi + lag + events_raw + rollback), frontend-security (CSP final + SRI + CSP-report), database-security (GRANT/REVOKE + TLS + RPO/RTO), fraud-and-abuse (heuristik H1-H7 + appeal + SLA), admin-security (runbook allowlist + 2-admin), infrastructure-security (nftables + sshd + Docker hardening), cicd-security (SHA pin + OIDC vs SSH + branch protection), incident-response (roster + template + S1 runbook + postmortem), security-requirements (kolom Owner/Target + TC), security-roadmap (exit criteria + coverage matrix), security-gap-analysis (risk scoring + owner/date), catastrophic-scenarios (peta drill). **Fase 5** (produk/development/tasks/root): 00-overview (glosarium + konteks + traceability + scope matrix), 01-PRD (KR + FR-ID + traceability + risiko + contoh penjualan), 02-requirements (spec engineering per capability), 03-user-flows (step-ID + Mermaid + exception table), 04-ux-ui-spec (wireframe 9 halaman + token + a11y + copy EN), development/* (merge strategy + CHANGELOG format + workflow snippet + lint + i18n + retry + off-chain races + secret store), tasks/* (kolom Status/Estimate/Done-when/Milestone + mapping + exit criteria), README (prasyarat + quickstart + license), AGENTS (perintah build/test + review checklist + escalation). **Konsistensi silang**: `/healthz` → `/api/health`; kode error baru didaftarkan (AUTH_* lengkap, INVALID_PRICE); `require linear history` → tidak (selaras squash merge); ADR-010..015 diperluas setara ADR-002..009. **Verifikasi akhir**: 0 tautan rusak, 0 orphan, 0 fence ganjil, 0 file tanpa newline, 172/172 ID terdefinisi, ~167.7k kata di 89 file |
| 2026-10-02 | **Ronde 14 — git/versioning/CI-CD/scaling/error/concurrency (permintaan user)** | **File baru**: `.gitignore` (ketat), `.env.example`, `.gitattributes`, `.editorconfig`, `CHANGELOG.md`, `.github/workflows/{ci,security,deploy-dev,deploy-testnet,deploy-mainnet}.yml`, `docs/development/{git-workflow,versioning-and-release,ci-cd,code-standards,error-handling,concurrency-and-races,secrets-and-gitignore}.md`, `docs/architecture/scaling.md`. **Update**: AGENTS.md (Git Rules 3-branch + aturan wajib tanya user sebelum merge testnet/mainnet, File & Secret Hygiene, kebijakan komentar, NOT-DO diperluas), DOCUMENTATION-MAP (SSOT + trigger T14..T20 + indeks), environments (env dev + mapping branch), cicd-security (rujuk ci-cd + secret scan + 3 branch), infrastructure (pointer scaling + RPC rate limit), order-protocol-security (bagian race), security-requirements (SEC-ORDER-006 race, SEC-CICD-002 secret scan, SEC-CICD-003 proteksi 3 branch), test-cases + testing-strategy (race/double-buy), backlog + implementation-plan (TASK-031..034), 00-overview (status) |
| 2026-10-02 | **Ronde 14b — aturan kualitas kode (permintaan user)** | AGENTS.md (Testing Rules: **semua endpoint API wajib punya test** happy+error; bagian baru **Behavioral Guidelines**: Think Before Coding, Simplicity First, Surgical Changes, Goal-Driven Execution; bagian baru **Server-Side Fetch & SSRF**), `development/error-handling.md` (§8 baru: **satu modul error terpusat** — impor, bukan ad-hoc; §9 Status), `development/code-standards.md` (ikuti pola yang ada; error via modul terpusat), `testing/testing-strategy.md` (endpoint API wajib test + DoD dipertegas) |
| 2026-10-02 | **Ronde 16 — full crosscheck kesiapan bangun + penutupan semua gap** | **Audit 3 subagent** menemukan: ADR-002..009 tanpa file (sudah dibuat ronde 15), ~20 inkonsistensi baru, dan gap kesiapan Fase 1. **Keputusan A1–A11 diterapkan** (ADR-001 ronde 16): A1 basis royalti bundle (listing aktif > harga mint > proporsional; validasi Σ+fee ≤ harga saat create_bundle); A2 `expires_at` = u64 nanodetik (marketplace, webhooks, 02-requirements); A3 nama event final (`factory_collection_created`, `market_bundle_create/cancel`, `market_pause/unpause`, `fee_update`, `treasury_update`, `treasury_withdraw`) + kolom pemitik di katalog; A4 SEC-CONTRACT-002/009/010 → DECIDED; A5 onboarding & leaderboard → M3 (PRD selaras backlog); A6 kosakata aksi admin = 9 aksi (wallet-authentication §3, endpoints, database-schema); A7 lockout = fail-bucket, nonce TIDAK diblokir; A8 retensi 7/4/3 + RTO <4 jam (database-security, secrets, security, asset-inventory, DR); A9 payout event = object map + nama field katalog (webhooks 3 payload dikoreksi + matematika fee benar); A10 DDL `sessions` kanonik + `refresh_token_hash`/`family_id`; A11 gas listing ≈10–20 Tgas total (satu keluarga angka). **Perbaikan**: DDL `approval_request` dibuat; registry error +5 kode (`INVALID_ROYALTY`, `CONFLICT_PHASE_OVERLAP`, `CONFLICT_PRE_VALIDATE_FAILED`, `CONFLICT_BUNDLE_TOO_MANY/ITEM_INVALID`); peta panic §4 selaras; marker ➕ usang dihapus; TV namespace = SSOT wallet-authentication §8 (authentication.md berhenti duplikasi); CORS headers disatukan; rate limit GET profile 60/IP/menit; MVP search di 02-requirements dikoreksi ke RPC langsung (ADR-004); RESEARCH.md +4 "CATATAN PROYEK" (callback listing, 0.01 Ⓝ, nama method, msg vs arg); database-security `nearsea_<env>`; testing-strategy catatan SSOT peta INV. **File baru**: `docs/contracts/{nft-collection,market,factory}.md` (reference implementasi — signature, init args, layout, prefix storage, konstanta, view shapes), `docs/features/launchpad.md`, `docs/features/report.md`, `.gitleaks.toml`, `rust-toolchain.toml`, `.nvmrc`. **Test**: TC-043..TC-052 (bundle guard, cancel listing/offer/bundle, withdraw_fees, forged callback, fallback verify, body cap, CORS, health); peta INV-013/028 + SEC-CONTRACT-003/012 diperbarui. **Prosedur baru**: menambah SEC requirement (register) + pembaruan threat-model. **SSOT baru**: contracts/* = reference implementasi; katalog event = webhooks.md. Verifikasi: 0 tautan rusak, 0 orphan, 0 fence ganjil, 0 newline hilang, ID lengkap |
| 2026-10-07 | **Ronde 17 — grilling + riset pasar + pemotongan scope** | **Riset pasar** (3 agent + verifikasi langsung): `docs/research/near-nft-market-2026.md` BARU — Paras pivot ke narrativeprotocol.com (FACT, cek langsung), MITTE tutup 2025-04-10, Mintbase paused, Few and Far jadi blog, hanya HotCraft hidup (~$742/mgg); perang royalti sudah kalah. **Keputusan user**: niat proyek = pembelajaran/portofolio; scope dipotong. **ADR-016** BARU: niat proyek + M1 = vertical slice (8 task) + M1+ (16 task pindah) + M4 opsional. **Update**: 01-PRD §2 premis dikoreksi dgn bukti; milestones (M1 baru, M1+ baru, M4 ditandai opsional); backlog (16 task M1→M1+); implementation-plan (Fase 1/2 slice, Fase 2b M1+); contracts/market.md §Catatan review (7 temuan C1-C3/H1-H4 + status pasca-cut); error-handling §9 katalog i18n 10→~60 kunci + CHAIN_REVERT tidak lagi menjanjikan refund; 02-requirements (A2 ns, A11 gas, label langkah listing dikoreksi); deployment + security-roadmap (Phase E/F ditandai hanya jalur produk). **CONTEXT.md** BARU (glosarium; MVP/order ambigu dicatat). Verifikasi: 0 tautan rusak, 0 orphan, ID lengkap |
| 2026-10-07 | **Ronde 17b — tutup blocker slice M1** | Verifikasi 1 agent: slice TIDAK self-contained (3 blocker). **Perbaikan**: B1 `TASK-008 Depends` 007,008b -> 007 (branding tidak memblokir M1); B2 `set_phases` minimal (satu fase publik) dipindah ke TASK-002 + catatan di contracts/nft-collection.md (mint tanpa fase = tidak bisa mint); B3 TASK-006 rescope ke subset slice + daftar TC. **C3 SELESAI**: pending_purchases + recover_stuck_purchase (permissionless) + INV-031 + TC-054 (contracts/market.md §3b). **H4 SELESAI**: definisi stale dua kasus (ownership mismatch ATAU approval invalid) + TC-053 (§3a) + INV-016 diperbarui + event market_stale_detected reason + market_purchase_recovered di webhooks. **Keputusan ditutup**: OQ-001 treasury (testnet=owner), OQ-002 IPFS (Pinata), OQ-003 domain (localhost+placeholder), OQ-008 event bundle; OQ-005/006 dibatalkan (bukan produk); LICENSE MIT + file LICENSE dibuat. **Stale-after-cut dibersihkan**: implementation-plan (pemetaan fase, exit criteria Fase 1/2, sisa non-blokir), milestones (demo M1 = slice 5 langkah, M1+ demo terpisah). Verifikasi: 0 tautan rusak, 186/186 ID |
| 2026-10-07 | **Ronde 17c — discoverability: konvensi repo vs struktur tooling** | Pertanyaan user: struktur repo tidak mengikuti default skill (Matt Pocock: `CONTEXT.md` + `docs/adr/`). **Keputusan: TIDAK restrukturisasi** — memindahkan `docs/decisions/` → `docs/adr/` akan memutus 361 rujukan `ADR-0…` + 65 rujukan `decisions/` di 97 file tanpa manfaat fungsional (melanggar "Surgical Changes"). **Yang diperbaiki = discoverability**: AGENTS.md bagian baru **"Konvensi Repo"** (tabel pemetaan path default tooling → konvensi repo: `CONTEXT.md`, `docs/decisions/ADR-NNN-*.md`, `tasks/backlog.md`, `docs/features/*`, `docs/contracts/*`) + 2 aturan (jangan tambah struktur baru tanpa justifikasi; daftarkan file baru di §4 peta ini). **README.md**: pohon repo disinkronkan (ADR-001..016, `contracts/`, `research/`, `CONTEXT.md`, `LICENSE`, `rust-toolchain.toml`, `.nvmrc`, `.gitleaks.toml`); tabel Documentation + bagian Repository Structure + Prerequisites (versi mengikuti file pin, bukan angka lepas). **AGENTS.md**: "Important Architecture Decisions" ADR-001..015 → ..016. **AUDIT-TEMUAN.md**: ditandai arsip historis (rentang ADR-nya = keadaan saat audit). Verifikasi: 0 tautan rusak |
| 2026-10-07 | **Ronde 17c-lanjutan — konfigurasi skill (`docs/agents/`)** | Melengkapi jalur integrasi yang benar (bukan sekadar tabel di AGENTS.md): **3 file baru** di `docs/agents/` — `domain.md` (single-context; rumah ADR = `docs/decisions/`; aturan kosakata glosarium + penandaan konflik ADR; tabel pemisahan `features/*` vs `contracts/*`), `issue-tracker.md` (tracker markdown lokal: SSOT `tasks/backlog.md` + `.scratch/<fitur>/issues/`; konvensi wayfinding; aturan promosi kesimpulan ke backlog/ADR), `triage-labels.md` (5 peran → label; **pemisahan tegas kolom `Status` = progres vs triage**). **AGENTS.md**: blok **`## Agent Skills`** (Issue tracker / Triage labels / Domain docs) — format yang dibaca `/grill-with-docs`, `/to-spec`, `/to-tickets`, `/triage`, `/wayfinder`; tabel Konvensi Repo +baris `docs/agents/`. **`.gitignore`**: `.scratch/` **DITINJAU & sengaja tidak di-ignore** (tidak memuat kredensial; ikut di-commit) + aturan bahwa `.scratch/` = state turunan yang kesimpulannya wajib dipromosikan ke SSOT. **README.md**: `docs/agents/` masuk pohon repo. Verifikasi: 100 file, 1068 tautan, 0 rusak |
| 2026-10-07 | **Ronde 17e — `/to-tickets` slice M1** | Menjalankan skill `/to-tickets` (Matt Pocock) dengan **scope M0+M1 saja** (bukan M0–M4): tiket untuk M2–M4 akan basi sebelum dikerjakan, dan beberapa task M1+ belum bisa punya AC (TASK-010 terblokir C1/C2/C3). **Output: `.scratch/m1-slice/`** — `spec.md` (peta tiket + graf ketergantungan + demo M1 + aturan kerja) + **10 tiket** di `issues/`: `01` scaffold+CI (TASK-001), `02` proteksi branch+secret scan (TASK-031), `03` versioning (TASK-032), `04` NFT core+`set_phases` (TASK-002), `05` royalti (TASK-003), `06` listing 2-tx (TASK-004), `07` buy+settlement+refund (TASK-005), `08` suite sandbox slice (TASK-006), `09` FE wallet (TASK-007), `10` FE browse/list/buy (TASK-008). Frontier = tiket `01`. **`/to-spec` sengaja dilewati** — spec sudah ada di `docs/`; membuat spec baru akan melanggar aturan SSOT repo ini. **Penyimpangan sadar dari resep tracer-bullet didokumentasikan** di `spec.md` §Catatan: tiket mengikuti graf `Depends` yang sudah jadi SSOT (memotong ulang akan memutus jejak `TASK-xxx`), dan skill mengizinkan alternatif "verifiable on its own". **`.scratch/` TIDAK didaftarkan di inventaris §4** — itu state kerja turunan, bukan dokumen rujukan; pointer ditambahkan di `tasks/backlog.md` agar ditemukan. Verifikasi: 10/10 tiket punya heading+blocker+status+AC (≥6 AC masing-masing), semua nomor blocker resolve, 0 tautan rusak |
| 2026-10-07 | **Ronde 17d — `git init` + 3 branch + perbaikan mojibake `tasks/`** | **Repo diinisialisasi** (permintaan user): `git init -b mainnet`, identitas lokal `keydeveloping`, `core.autocrlf=false`; **3 branch permanen** dibuat (`mainnet` default, `testnet`, `dev`) — semua menunjuk commit baseline; commit baseline `chore: baseline dokumentasi & konfigurasi repo` = 113 file. **Temuan & perbaikan: mojibake di 2 file `tasks/`** — `backlog.md` (186 marker) + `implementation-plan.md` (36 marker) memuat sisa korupsi encoding PowerShell dari sesi sebelumnya: teks UTF-8 dibaca sebagai CP1252 lalu disimpan ulang sebagai UTF-8, sehingga em dash, en dash, panah, tanda bagian, titik tengah, ≤, …, ⭐, ⏳, ⚠️ semua rusak (sebagian dua tingkat). Diperbaiki dengan decoder UTF-8 ber-validasi (iteratif, hanya sekuens yang ter-decode bersih) — **194 sekuens** dipulihkan. **Bukti verifikasi**: (a) 0 marker mojibake tersisa; (b) kerangka ASCII (teks dengan semua non-ASCII dibuang) **identik** sebelum/sesudah → tidak ada satu kata pun yang berubah; (c) rekonstruksi dari HEAD memakai **tabel penggantian eksplisit 15 baris** (diturunkan dari enumerasi run non-ASCII) cocok byte-per-byte dengan hasil; (d) file dibaca ulang penuh dan terbaca benar; (e) himpunan karakter non-ASCII hasil hanya yang sah (`— – … · ≤ → § ⭐ ⚠️ ⏳`). **Higienitas**: `.zcodeignore` ditambahkan ke ignore; 113 file ter-stage, 0 nama sensitif, 0 temuan scan kredensial; line ending dinormalisasi ke LF (6 file yang melanggar `.editorconfig`). **Catatan TASK-031 tetap `todo`** — 3 branch + `.gitignore` selesai, tapi proteksi branch + gitleaks CI + CODEOWNERS baru bisa setelah remote GitHub ada. Verifikasi: 0 tautan rusak, 0 fence ganjil, 0 file tanpa newline |
| 2026-10-07 | **Ronde 18b — remote GitHub + bukti gate CI (TASK-001 `done`)** | User memberi remote `github.com/keydeveloping/nearsea` (**private**). 3 branch permanen (`mainnet`/`testnet`/`dev`) di-push membentuk struktur repo; **default branch di-set `mainnet`** (GitHub awalnya memilih `dev`; spec [git-workflow.md](./development/git-workflow.md) §1 menuntut `mainnet`). Branch `chore/scaffold-repo` → **PR #1** → **squash-merge ke `dev`** (commit `223015b`), branch fitur dihapus. **CI + Security hijau di `dev`** (bukti: [CI 37627825466](https://github.com/keydeveloping/nearsea/actions/runs/37627825466), [Security 37627825490](https://github.com/keydeveloping/nearsea/actions/runs/37627825490)) → Done-when TASK-001 terpenuhi. **Tiga bug nyata ditemukan run CI (tidak terlihat lokal)** dan diperbaiki dalam 2 commit lanjutan: (1) `Cannot find name 'LayoutProps'` — `app/layout.tsx` memakai tipe hasil generate Next.js (`.next/types`); lokal tertutup `.next/` sisa build, CI menjalankan typecheck sebelum build → script `typecheck` kini `next typegen && tsc --noEmit`; (2) `tinypool` (critical, prototype pollution → RCE) lewat vitest 3 → **vitest 4** (tidak lagi memakai tinypool); (3) gitleaks 403 "Resource not accessible by integration" → job butuh `pull-requests: read`. **Juga**: `cargo audit` lewat `rustsec/audit-check` gagal karena instalasi tanpa `--locked` menarik dependency yang menuntut rustc 1.96 → diganti langkah eksplisit `cargo install cargo-audit --locked`; `dependency-review` di-skip terarah selama branch target belum punya dependency graph (kegagalan struktural ≠ temuan). **Sisa 1 advisory tanpa patch** (`braces <=3.0.3`, high, ReDoS, lewat `eslint-config-next`) — pengecualian ditulis **eksplisit** di `frontend/pnpm-workspace.yaml` (`auditConfig.ignoreCves`, bukan flag senyap) + pelacak **TASK-035**. **Dokumen disinkronkan**: backlog (TASK-001 → `done`, +TASK-035), milestones (M0 tertutup), 00-overview (Status), security-requirements (SEC-CICD-002 → READY-FOR-IMPLEMENTATION; 001/003 tetap BLOCKED dengan alasan baru), ci-cd (§3 gate audit + pengecualian + job bergantung-baseline), tech-stack + 01-PRD §15.1 (vitest 4), `.scratch/m1-slice/issues/01` (status `resolved` + bukti). **Catatan**: Dependabot sudah membuka 5 PR bump GitHub Actions ke `dev` (semua check hijau; bump mayor pada workflow → butuh review manusia, belum di-merge) + PR `docker` gagal karena belum ada Dockerfile (TASK-028). |
