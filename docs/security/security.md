# Security

> Keamanan menyeluruh. Checklist kontrak NEAR ada di [RESEARCH.md](../../RESEARCH.md) bagian 3 — WAJIB dibaca setiap agent yang menyentuh kontrak.
> Dokumen ini = **peta masuk** (index + parameter + severity + recovery + test mapping). Fakta detail dimiliki dokumen pemiliknya (single source of truth — [DOCUMENTATION-MAP.md](../DOCUMENTATION-MAP.md) §1).

## Peta dokumen keamanan (index)

> Setiap bullet keamanan di repo ini punya **dokumen pemilik** dan **SEC-ID** di register. Tabel berikut memetakan keduanya agar tidak ada kontrol yatim. Status enum SEC: `PROPOSED` / `RESEARCHING` / `DECIDED` / `BLOCKED` / `READY-FOR-IMPLEMENTATION` ([security-requirements.md](./security-requirements.md)).

| Dokumen | Cakupan | SEC-ID utama | Fase |
|---|---|---|---|
| [security-requirements.md](./security-requirements.md) | Register kanonik semua requirement (SSOT) | semua `SEC-*` | MVP→mainnet |
| [threat-model.md](./threat-model.md) | Ancaman STRIDE + Web3, skoring, attack tree | SEC-AUTH/ORDER/CONTRACT/FE/INDEX | MVP→fase 2 |
| [asset-inventory.md](./asset-inventory.md) | Aset yang dilindungi + CIA + retensi | SEC-KEY-001/002, SEC-DB-002 | MVP+ |
| [trust-boundaries.md](./trust-boundaries.md) | Batas kepercayaan + asersi testable | lintas dokumen | MVP+ |
| [permissions.md](./permissions.md) · [access-control-matrix.md](./access-control-matrix.md) | Role, operasi privileged, SoD | SEC-ADMIN-001/002/003 | MVP+ |
| [key-management.md](./key-management.md) | Owner/deploy key, Sputnik DAO | SEC-KEY-001/002, SEC-CONTRACT-012 | MVP→mainnet |
| [wallet-authentication.md](./wallet-authentication.md) | NEP-413, nonce, session (pemilik template §3) | SEC-AUTH-001..006 | MVP |
| [signature-architecture.md](./signature-architecture.md) | Taksonomi S1–S4, replay, revocation | SEC-SIGN-001, SEC-AUTH-001 | MVP+ |
| [order-protocol-security.md](./order-protocol-security.md) | Lifecycle order, race, refund | SEC-ORDER-001..006 | MVP |
| [smart-contract-security-architecture.md](./smart-contract-security-architecture.md) | Init, upgrade, pause, fee, approval | SEC-CONTRACT-001..012 | MVP→mainnet |
| [smart-contract-invariants.md](./smart-contract-invariants.md) | INV-001..030 (target test & audit) | turunan `SEC-CONTRACT/ORDER` | MVP |
| [api-security-architecture.md](./api-security-architecture.md) | BOLA, authz, injection, rate limit | SEC-API-001/002 | MVP |
| [metadata-security.md](./metadata-security.md) | XSS/SVG, gateway, fetcher/SSRF | SEC-META-001/002 | MVP→fase 2 |
| [indexer-security.md](./indexer-security.md) | Finality, dedup, otoritas | SEC-INDEX-001/002 | fase 2 |
| [frontend-security.md](./frontend-security.md) | CSP, supply chain, preview=on-chain | SEC-FE-001/002, SEC-META-001 | MVP |
| [database-security.md](./database-security.md) | Klasifikasi data, constraint, backup | SEC-DB-001..004 | MVP+ |
| [admin-security.md](./admin-security.md) | Allowlist, step-up, audit, break-glass | SEC-ADMIN-001..003 | MVP+ |
| [infrastructure-security.md](./infrastructure-security.md) | Hardening VPS, segmentation, secret | SEC-INFRA-001/002 | MVP+ |
| [cicd-security.md](./cicd-security.md) | Branch protection, secret scan, supply chain | SEC-CICD-001..003 | MVP+ |
| [fraud-and-abuse.md](./fraud-and-abuse.md) | Wash/Sybil/impersonation (display-only) | SEC-INDEX-001 (batas) | MVP→fase 2 |
| [catastrophic-failure-scenarios.md](./catastrophic-failure-scenarios.md) | CS-1..CS-10 + prioritas | SEC-IR-001 | MVP→mainnet |
| [incident-response.md](./incident-response.md) | Sev, playbook, drill, G7 | SEC-IR-001..003 | MVP→mainnet |
| [security-roadmap.md](./security-roadmap.md) | Phase A–F | semua | semua |
| [security-gap-analysis.md](./security-gap-analysis.md) | G1..G16 + keputusan | — | riset |

**Bullet → pemilik** (setiap poin di dokumen ini tidak mendefinisikan ulang; ia merujuk):

| Bullet di dokumen ini | Dokumen pemilik | SEC-ID |
|---|---|---|
| `assert_one_yocto`, callback `#[private]`, predecessor, dual verification, payout validation | smart-contract-security-architecture.md · smart-contract-invariants.md | SEC-CONTRACT-001/003/005 |
| Storage pre-deposit NEP-145 | smart-contract-invariants.md (INV-020) · database-security.md | SEC-CONTRACT-011 |
| `overflow-checks` + checked math | smart-contract-invariants.md (INV-001..004) | SEC-CONTRACT-005 |
| Pausable + guardian `pause_callers` | key-management.md §3 · smart-contract-security-architecture.md §4 | SEC-CONTRACT-007/012 |
| Rate limit API + input validation + CORS | api-security-architecture.md | SEC-API-001 |
| Guard session + allowlist admin | admin-security.md · wallet-authentication.md | SEC-ADMIN-001, SEC-AUTH-003 |
| Full-access key hanya di VPS | key-management.md | SEC-KEY-001 |
| Sputnik DAO V2 2-of-3 + timelock | key-management.md §3 · ADR-013 | SEC-KEY-002, SEC-CONTRACT-012 |
| Audit eksternal sebelum mainnet | security-roadmap.md (Phase E) | TASK-027 |

## On-chain (draft dari riset)

- `assert_one_yocto()` pada semua mutasi seller (remove/update listing, withdraw).
- Callback (`resolve_purchase`, `process_listing`, dst.) selalu `#[private]`.
- Otorisasi via `predecessor_account_id` (bukan signer) untuk pemanggil langsung.
- Validasi payout: ≤ 10 penerima, total ≤ harga, sisa 0..1 yocto.
- Dual verification saat listing (ownership + approval) — cegah listing NFT milik orang lain.
- Storage pre-deposit (NEP-145) — kontrak tak pernah menanggung storage user.
- `overflow-checks = true` pada profile.release; aritmetika nilai pakai checked ops.
- **Pausable (ronde 13)** di market & factory — kill switch: MVP owner-only; **mainnet: guardian `pause_callers` terpisah dari owner-DAO** (key-management §3, ADR-013). Saat pause: mutasi baru ditolak, penarikan escrow/refund tetap terbuka.

## Off-chain (API/report — default, bisa diperketat)

- Rate limit per IP + per akun pada endpoint tulis (report: mis. 5/hari/akun).
- Input validation ketat (panjang string, enum alasan report); CORS terbatas ke domain app.
- Payload report dibatasi ukuran; tidak ada upload file di v1.
- Admin endpoint: guard session signature + allowlist admin (lihat [admin-security.md](./admin-security.md)).

## Secret management

- Full-access key deploy: **hanya di VPS** (`~/.near-keys/`, permission 600) + `.env` server-side.
- Dilarang keras: secret di git, di frontend, atau di log. `.env.production` tidak masuk repo (contoh: `.env.example`).
- **Mainnet**: model berubah ke Sputnik DAO V2 council 2-of-3 + guardian pause — [key-management.md](./key-management.md) & ADR-013 (DECIDED).

## Audit

- Sebelum mainnet (M4): review manual menyeluruh + checklist [docs.near.org/security/checklist](https://docs.near.org/smart-contracts/security/checklist) + threat-model direview ulang + **audit eksternal** (TASK-027).

## Model ancaman → kontrol (ringkas)

> Matriks cepat: ancaman tingkat tinggi → kontrol yang menjawabnya → SEC-ID. Detail skoring & attack tree ada di [threat-model.md](./threat-model.md).

| # | Ancaman | Kontrol utama | Layer | SEC-ID |
|---|---|---|---|---|
| T-01 | Replay signature API | nonce 32B sekali pakai + TTL 5m, konsumsi atomik | API/DB | SEC-AUTH-001 |
| T-02 | Wrong-domain / wrong-chain signature | `recipient`=domain (NEP-413) + `network` di dalam message | API | SEC-AUTH-002 |
| T-03 | Identity spoof (key bukan milik akun) | `view_access_key_list` skema-agnostik | API/RPC | SEC-AUTH-004 |
| T-04 | Session theft / abuse | JWT TTL 15m + rotasi refresh; admin tanpa silent refresh | API | SEC-AUTH-003/006 |
| T-05 | Owner key compromise | key hanya di VPS 600; mainnet Sputnik DAO 2-of-3 + timelock | Kunci/governance | SEC-KEY-001/002 |
| T-06 | Upgrade ke kode jahat | reproducible build NEP-330 + timelock + hash publish | Kontrak | SEC-CONTRACT-006/012 |
| T-07 | Accounting/payout salah | INV-001..004, checked math, ≤10 penerima, sisa ≤1 yocto | Kontrak | SEC-CONTRACT-005 |
| T-08 | Drain escrow offer | refund hardcoded `buyer_id`, entry dihapus atomik | Kontrak | SEC-ORDER-002 |
| T-09 | Malicious NFT contract | permissionless + payout divalidasi + gas caps + optimistic revert | Kontrak | SEC-CONTRACT-003/005/010 |
| T-10 | Reentrancy/callback manipulation | `#[private]` + validasi promise result | Kontrak | SEC-CONTRACT-003 |
| T-11 | Init takeover | `#[init]` + `PanicOnDefault` | Kontrak | SEC-CONTRACT-002 |
| T-12 | Race double-buy | unique key + optimistic removal | Kontrak | SEC-ORDER-005/006 |
| T-13 | XSS via metadata/SVG | render `<img>` saja, CSP strict, tanpa `innerHTML` | FE | SEC-META-001, SEC-FE-001 |
| T-14 | SSRF (fase 2 fetcher) | guard IP privat + allowlist protokol + isolasi proses | Fetcher | SEC-META-002 |
| T-15 | Supply chain FE/CI | lockfile, audit gate, gitleaks, branch protection | CI/CD | SEC-CICD-001/002/003 |
| T-16 | Indexer poisoning | ingest final-only, dedup `(receipt_id,event_index)`, non-otoritatif | Indexer | SEC-INDEX-001/002 |
| T-17 | Admin privilege escalation | allowlist DB + step-up + scope terpisah | API/DB | SEC-ADMIN-001/003 |
| T-18 | Audit tampering | `admin_audit` append-only (REVOKE UPDATE/DELETE) | DB | SEC-ADMIN-002, SEC-DB-003 |
| T-19 | DNS hijack / mirror phishing | registrar lock + 2FA + cron `dig` + edukasi | Infra | SEC-IR-002 |
| T-20 | DoS/gas exhaustion | batas statis loop + rate limit + body cap | Kontrak/API | SEC-CONTRACT-010, SEC-API-001 |

## Aktor & jalur otorisasi

> Ringkasan per-aktor. Register formal + matriks operasi privileged ada di [access-control-matrix.md](./access-control-matrix.md); role ringkas di [permissions.md](./permissions.md).

| Aktor | Kredensial | Bisa apa | Enforcement | Audit |
|---|---|---|---|---|
| Guest | — | browse/search (read-only) | API publik (rate-limited) | — |
| User (USER) | access key wallet (ed25519/secp256k1/ml-dsa-65) | tx atas asetnya sendiri; API `report`/`profile` via NEP-413 | Kontrak (`predecessor`+ownership) + API session | tx on-chain; log API |
| Collection Owner (CREATOR) | = User (owner koleksi) | mint sesuai phase, kelola phase/allowlist miliknya | Kontrak (owner koleksi + storage pre-deposit) | event launchpad |
| Admin off-chain (ADMIN) | signature wallet + allowlist DB + scope `admin` | moderasi display-layer (report/verified/blocklist) | API + DB allowlist; **tanpa kekuatan on-chain** | `admin_audit` |
| SUPER_ADMIN | allowlist + (mainnet) 2-admin approval | kelola allowlist admin | API/DB; MVP manual SQL | `admin_audit` manual |
| Platform Owner (PLATFORM_OWNER) | owner key (MVP) / Sputnik DAO 2-of-3 (mainnet) | pause, `fee_bps`, treasury, upgrade, withdraw | Kontrak (Owner pattern) | event `market_*` on-chain |
| Guardian (SECURITY) | guardian key (mainnet) | **hanya `pause`** (`pause_callers`) | Kontrak (daftar terpisah dari owner) | event `market_pause` |
| FINANCE (mainnet) | DAO proposal | tarik treasury | Kontrak (DAO) | event `treasury_withdraw` |
| Indexer (fase 2) | RPC read-only | ingest event → proyeksi | DB role read-only | lag/reconciliation log |
| Attacker (untrusted) | — | apa pun yang diizinkan protokol terbuka | dibatasi oleh kontrol di atas | deteksi via alert/monitoring |

## Parameter keamanan konkret

> Nilai-nilai ini **dimiliki** dokumen lain (lihat kolom Owner) — tabel ini hanya konsolidasi agar tidak ada nilai tersembunyi. Dilarang mengubah angka di sini tanpa mengubah pemiliknya lebih dulu.

| Parameter | Nilai | Owner dokumen | SEC/INV |
|---|---|---|---|
| Fee platform | `fee_bps=200` (2%) | 01-PRD §13 · smart-contract-security-architecture §5 | SEC-CONTRACT-004 |
| Cap fee | `MAX_FEE_BPS=500` (5%, immutable) | smart-contract-invariants.md (INV-004) | SEC-CONTRACT-004 |
| Cap royalti | 10% per token (bundle tanpa cap agregat) | smart-contract-invariants.md (INV-027) | SEC-CONTRACT-005 |
| Penerima payout | ≤10 unik setelah merge | smart-contract-invariants.md (INV-021) | SEC-CONTRACT-005 |
| Sisa pembulatan | ≤1 yocto | smart-contract-invariants.md (INV-002) | SEC-CONTRACT-005 |
| Harga minimum | 0.01 Ⓝ (listing & offer) | smart-contract-invariants.md (INV-030) | — |
| Durasi offer | default 7 hari | smart-contract-invariants.md (INV-024) | — |
| Offer aktif | maks 1 per buyer per token | smart-contract-invariants.md (INV-024) | — |
| Bundle | maks 10 token | smart-contract-invariants.md (INV-021) | SEC-CONTRACT-010 |
| Nonce auth | 32 byte, TTL 5 menit, sekali pakai | wallet-authentication.md §3 | SEC-AUTH-001 |
| Session access | TTL 15 menit (900 s) | api/authentication.md | SEC-AUTH-003 |
| Refresh | TTL ≤12 jam (43200 s), rotasi tiap pakai | api/authentication.md | SEC-AUTH-006 |
| Session admin | 15 menit, **tanpa silent refresh** | admin-security.md §2 | SEC-AUTH-006 |
| Rate limit nonce | 10/IP/menit + 30/akun/jam | api-security-architecture.md §1 | SEC-AUTH-005 |
| Rate limit verify | 10/IP/menit; 5 gagal/menit → lockout | wallet-authentication.md §5 | SEC-AUTH-005 |
| Rate limit report | 5/akun/hari | api-security-architecture.md §1 | SEC-API-001 |
| Rate limit profil | GET 60/IP/menit; PATCH 10/akun/jam | api-security-architecture.md §1 | SEC-API-001 |
| Rate limit discovery | 60/IP/menit | api-security-architecture.md §1 | SEC-API-001 |
| Body size cap API | mis. 16 KB | api-security-architecture.md §2 | SEC-API-001 |
| Gas `nft_transfer_payout` | 15 Tgas | smart-contract-security-architecture §8 | — |
| Gas resolve callback | 115 Tgas | smart-contract-security-architecture §8 | — |
| Batas gas tx | 300 Tgas (FACT) | smart-contract-invariants.md (INV-021) | SEC-CONTRACT-010 |
| Timelock mainnet | 24 jam (fee/treasury/upgrade) | key-management.md §3 | SEC-CONTRACT-012 |
| Council DAO | 2-of-3 | key-management.md §3 | SEC-KEY-002 |
| Backup retensi | 7 harian + 4 mingguan + 3 bulanan | disaster-recovery.md §Jadwal (SSOT) · database-security §4 | SEC-DB-001 |
| Media on-chain | URL + hash saja (≤4 MB/call, 1 Ⓝ/100 KB) | AGENTS.md · RESEARCH.md §3 | — |

## Severity & eskalasi

> Model severity kanonik dimiliki [incident-response.md](./incident-response.md) §1; tabel ini menambah pemetaan ke kemungkinan/keseriusan threat.

| Sev | Definisi | Contoh | Target respons | Kanal |
|---|---|---|---|---|
| S1 | Dana/aset user berisiko AKTIF | exploit kontrak, owner key dicuri | segera (menit): pause/break-glass | Telegram + status page |
| S2 | Kontrol rusak, dana belum hilang | key leak terdeteksi, CI compromised, auth bypass | jam: rotasi/patch | Telegram |
| S3 | Degraded / data off-chain rusak | API down, indexer korup, spam massal | hari: perbaikan normal | Telegram (ringan) |

Pemetaan likelihood×impact (detail di threat-model.md): **Kritis** = potensi kehilangan dana/aset (S1); **Tinggi** = kontrol kunci rusak (S2); **Sedang** = degradasi/data off-chain (S3).

## Pemulihan (recovery) per kelas insiden

> Langkah ringkas; playbook lengkap + 9 field per skenario di [catastrophic-failure-scenarios.md](./catastrophic-failure-scenarios.md) dan [incident-response.md](./incident-response.md).

```text
KELAS: OWNER KEY / UPGRADE (CS-1, CS-9)            [S1]
1. Pause via guardian `pause_callers` (mainnet) atau owner key bila masih aman.
2. JANGAN pakai seed backup yang mungkin ikut bocor — buat key FRESH.
3. Deploy kontrak baru + migrasi listing; rotasi semua kunci/secret.
4. Playbook mass-revoke: user `nft_revoke_all` per token + market `remove_stale_listing` (G6).
5. Escrow offer: user `cancel_offer` (refund otomatis ke buyer hardcoded).

KELAS: ACCOUNTING/PAYOUT (CS-3)                    [S1]
1. Pause sampai patch; identifikasi tx terdampak dari event `market_sale`.
2. Perbaiki + test regresi INV-001..004 sebelum unpause.
3. Kompensasi kasus terverifikasi via G7 (insurance fund 0.1% fee, proposal DAO).

KELAS: DRAIN ESCROW (CS-4)                          [S1]
1. Pause; audit jalur refund/accept.
2. Refund manual + kompensasi G7; test regresi accept/refund sebelum unpause.

KELAS: FE / CI-CD / DNS (CS-6, CS-7, CS-8)          [S1–S2]
1. Rollback FE ke build terverifikasi; cabut token deploy + rotasi secret CI.
2. Restore DNS dari registrar (lock/restore), umumkan peringatan.
3. Audit provenance, rebuild, verifikasi hash, baru deploy.

KELAS: API / DB / INDEXER (S2–S3)
1. Take offline / tandai degraded; simpan bukti (log/tx/snapshot) SEBELUM dibersihkan.
2. Patch + rotasi kredensial; restore backup (SEC-DB-004) atau rebuild dari events_raw.
```

## Pemetaan test keamanan

> Setiap kontrol P0 harus punya test. Kolom Test memakai ID dari [testing/test-cases.md](../testing/test-cases.md) atau kelas test di [testing/testing-strategy.md](../testing/testing-strategy.md). Test yang belum ada → ditambahkan saat task implementasi.

| Kontrol / SEC-ID | Jenis test | Test ID / kelas |
|---|---|---|
| SEC-ORDER-001/002/004/005/006 (settlement, refund, stale, race) | sandbox | TC-002..006, TC-016..019, TC-042 |
| SEC-CONTRACT-005 (payout ≤10, sisa ≤1 yocto) | unit matriks royalti + fuzz | TC-003, TC-015 |
| SEC-CONTRACT-004 (`MAX_FEE_BPS`) | unit + review | property: `update_fee_bps > 500` revert |
| SEC-CONTRACT-007/012 (pause, guardian) | sandbox + drill | TC-012 + SEC-IR-001 drill |
| SEC-CONTRACT-009 (launchpad) | sandbox | TC-007, TC-021 |
| SEC-CONTRACT-011 (storage NEP-145) | sandbox | TC-020 |
| SEC-AUTH-001/002/004/006 (nonce, NEP-413, key, session) | api + vector | TC-027..TC-030, TC-028 vector |
| SEC-ADMIN-001/003 (allowlist, step-up) | api | TC-025, TC-026, TC-034..TC-039 |
| SEC-API-001/002 (rate limit, authz) | load + integration | TC-027, TC-033, authz matrix |
| SEC-META-001, SEC-FE-001 (XSS/CSP) | e2e + CSP report | header scan + TC-040 |
| SEC-INDEX-001/002 (indexer) | review + test | fase 2 (TC discovery ★) |
| SEC-DB-001..004 (DB) | test DB perms + drill | restore drill (SEC-DB-004) |
| SEC-CICD-001/002 (supply chain) | CI config review | gitleaks + branch protection |
| SEC-IR-001..003 (IR) | drill + dokumentasi | drill pause/unpause + restore |
