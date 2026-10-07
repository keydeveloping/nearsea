# Security Requirements Register

> Register keamanan dengan ID stabil. **Status yang valid (enum, pilih satu)**: `PROPOSED` / `RESEARCHING` / `DECIDED` / `BLOCKED` / `READY-FOR-IMPLEMENTATION`.
> **Tidak pernah ada status IMPLEMENTED** sampai ada implementasi terverifikasi (Fase C+).
> **Fase** menunjukkan kapan requirement ini berlaku: `MVP` / `fase 2` / `mainnet gate`.
> Kolom **Ref** = dokumen kontrol pemilik requirement. Kolom **Verifikasi** menyebut test/ID bila sudah ada.
> Kolom **Owner** = peran/tim pemilik (contract/backend/frontend/infra/indexer/security). Kolom **Target** = milestone penutup (M1/M2/M3/M4) — lihat [tasks/milestones.md](../../tasks/milestones.md).

## Auth (API)

| ID | Requirement | Threat | Ref | Verifikasi | Priority | Fase | Owner | Target | Status |
|---|---|---|---|---|---|---|---|---|---|
| SEC-AUTH-001 | Challenge nonce sekali pakai, TTL ≤5 menit, consumed atomically (validasi dulu, konsumsi belakangan) | replay | wallet-authentication §3 | TC-027, TC-028 | P0 | MVP | backend | M1 | READY-FOR-IMPLEMENTATION |
| SEC-AUTH-002 | Message signature memuat domain + network + scope + expires — format utama **NEP-413** (status Final; recipient=domain, nonce 32B), custom challenge sebagai fallback | wrong-domain/wrong-chain signature | wallet-authentication §3 | TC-028 | P0 | MVP | backend | M1 | DECIDED |
| SEC-AUTH-003 | Session token scoped + TTL pendek + revocable | session abuse | api-security §4 | TC-029, TC-030 | P1 | MVP | backend | M1 | PROPOSED |
| SEC-AUTH-004 | Verifikasi public key terikat account via RPC (`view_access_key_list`) — skema-agnostik (ed25519 / secp256k1 / ml-dsa-65-hash) dan menerima semua tipe akun (named/implicit/0x) | identity spoof | wallet-authentication §1 | TC-028 | P1 | MVP | backend | M1 | READY-FOR-IMPLEMENTATION |
| SEC-AUTH-005 | Rate limit & lockout nonce (api-security §1) | brute/spam | wallet-authentication §5 | TC-027 | P1 | MVP | backend | M1 | PROPOSED |
| SEC-AUTH-006 | Refresh token rotasi setiap pakai; revoke via denylist server; **scope admin tanpa silent refresh** | session abuse | wallet-authentication §3, admin-security §2 | TC-029, TC-030 | P1 | MVP | backend | M1 | READY-FOR-IMPLEMENTATION |

## Signature / Order / Contract

| ID | Requirement | Threat | Ref | Verifikasi | Priority | Fase | Owner | Target | Status |
|---|---|---|---|---|---|---|---|---|---|
| SEC-SIGN-001 | Tidak ada off-chain order signature; order = state on-chain | order replay via signature model lain | ADR-012 | review arsitektur | P0 | MVP | contract | M1 | DECIDED |
| SEC-ORDER-001 | Settlement gagal → refund 100% + state kembali (atomic resolve) | dana buyer hangus | INV-001/002 | TC-003 | P0 | MVP | contract | M1 | DECIDED |
| SEC-ORDER-002 | Escrow hanya refund ke buyer_id (hardcoded) | drain escrow | INV-005 | TC-004 | P0 | MVP | contract | M1 | DECIDED |
| SEC-ORDER-003 | FE re-verify harga/ownership via view sebelum sign | transaksi berdasarkan cache basi | order-protocol-security §4 | TC-022, TC-040 | P0 | MVP | frontend | M1 | PROPOSED |
| SEC-ORDER-004 | Re-verify ownership+approval di settle (dual verification) | stale/executed listing | INV-016 | TC-006 | P0 | MVP | contract | M1 | DECIDED |
| SEC-ORDER-005 | Unique order key; no partial fill di MVP | overfill/double-fill | INV-007 | TC-016, TC-017 | P0 | MVP | contract | M1 | DECIDED |
| SEC-ORDER-006 | Race/konkurensi: `buy` bersamaan → 1 pemenang, sisanya revert + refund otomatis (optimistic removal + unique key); double-submit ditolak | double-spend / dua pembayar | concurrency-and-races | TC-016/017 | P0 | MVP | contract | M1 | DECIDED |
| SEC-CONTRACT-001 | `assert_one_yocto` pada semua mutasi seller/owner | function-call key abuse | AGENTS rules | TC-002, TC-012 | P0 | MVP | contract | M1 | DECIDED |
| SEC-CONTRACT-002 | `#[init]` + PanicOnDefault; init hanya owner valid | init takeover | smart-contract-security-architecture §9 | TC-001 | P0 | MVP | contract | M1 | DECIDED |
| SEC-CONTRACT-003 | Callback `#[private]` + predecessor checks; `nft_on_approve` memvalidasi payload NEP-178 | malicious callback | INV-013 | TC-048 | P0 | MVP | contract | M1 | DECIDED |
| SEC-CONTRACT-004 | `MAX_FEE_BPS` constant (≤500 = cap), `fee_bps` default 200 (2%) ≤ cap — perubahan hanya via governance ADR-013 | owner drains via fee | INV-004 | TC-005, TC-015 | P0 | MVP | contract | M1 | DECIDED |
| SEC-CONTRACT-005 | Payout ≤10 penerima, sum ≤ harga−fee, sisa ≤1 yocto, checked math | accounting bug | INV-002/003 | TC-003, TC-005, TC-015 | P0 | MVP | contract | M1 | DECIDED |
| SEC-CONTRACT-006 | Verifikasi build: **reproducible build (NEP-330)** — publish `contract_source_metadata`, verifikasi reproduksi via Docker/SourceScan + hash CI | artifact swap | smart-contract-security-architecture §2b | `near view … contract_source_metadata` + build ulang | P1 | mainnet gate | infra | M4 | DECIDED |
| SEC-CONTRACT-007 | Pausable: blokir mutasi, izinkan cancel/withdraw; teruji | emergency | INV-022 | TC-012 | P0 | MVP | contract | M1 | DECIDED |
| SEC-CONTRACT-008 | Storage layout dokumentasi per release; persistent prefix stabil | upgrade state korup | smart-contract-security-architecture §2 | review release | P1 | MVP | contract | M1 | PROPOSED |
| SEC-CONTRACT-009 | Launchpad: validasi fase/allowlist/alokasi/exact deposit | mint abuse | INV-017/018 | TC-007, TC-021 | P0 | MVP | contract | M1 | DECIDED |
| SEC-CONTRACT-010 | Batas statis semua loop (10 payout, maks 10 token/bundle) | gas DoS | INV-021 | TC-014 | P1 | MVP | contract | M1 | DECIDED |
| SEC-CONTRACT-011 | Storage growth dibayar user (NEP-145 semua map tumbuh) | storage exhaustion | INV-020 | TC-020 | P0 | MVP | contract | M1 | DECIDED |
| SEC-CONTRACT-012 | Mainnet: owner actions finansial via **Sputnik DAO V2 council 2-of-3 + timelock**; pause via guardian `pause_callers` terpisah dari owner-DAO — **register utama** (SEC-KEY-002 = rujukan silang) | owner key abuse | key-management §3, ADR-013 | TC-047 (owner-only) + setup + drill (SEC-IR-001) | P1 | mainnet gate | security | M4 | DECIDED |

## API / Metadata / Indexer / FE

| ID | Requirement | Threat | Ref | Verifikasi | Priority | Fase | Owner | Target | Status |
|---|---|---|---|---|---|---|---|---|---|
| SEC-API-001 | Rate limit semua endpoint + body size cap | resource exhaustion | api-security §1 | TC-027 | P1 | MVP | backend | M1 | PROPOSED |
| SEC-API-002 | AuthZ test per endpoint (BOLA matrix) | broken authorization | api-security §2 | TC-031..TC-039 | P1 | MVP | backend | M1 | PROPOSED |
| SEC-META-001 | FE: media hanya `<img>` + gateway allowlist + CSP | XSS/SVG | metadata-security §2 | TC-040 + CSP report | P0 | MVP | frontend | M1 | PROPOSED |
| SEC-META-002 | Fetcher server-side: protocol allowlist + SSRF guard + limits, proses terisolasi | SSRF | metadata-security §3 | unit + review | P1 | fase 2 | backend | M2 | PROPOSED |
| SEC-INDEX-001 | Indexer tidak pernah otoritas ownership/settlement | indexer poisoning | indexer-security §3 | review arsitektur | P0 | fase 2 | indexer | M2 | PROPOSED |
| SEC-INDEX-002 | Ingest hanya blok final + dedup (receipt_id,event_index); sumber stream = **Neardata** (NEAR Lake DEPRECATED 2026-03 — FACT) | duplicate/inconsistent | indexer-security §1-2 | TC-024 | P1 | fase 2 | indexer | M2 | DECIDED |
| SEC-FE-001 | CSP strict + security headers + tanpa third-party script | script injection | frontend-security §2 | TC-040 + header scan | P0 | MVP | frontend | M1 | PROPOSED |
| SEC-FE-002 | (rujukan silang SEC-ORDER-003 di lapisan FE) preview=on-chain | UI/cache mismatch | frontend-security §3 | TC-022 | P0 | MVP | frontend | M1 | PROPOSED |

## Admin / Infra / IR

| ID | Requirement | Threat | Ref | Verifikasi | Priority | Fase | Owner | Target | Status |
|---|---|---|---|---|---|---|---|---|---|
| SEC-ADMIN-001 | Admin = allowlist DB + signature scope admin | privilege escalation | admin-security §1 | TC-034, TC-026 | P0 | MVP | backend | M1 | PROPOSED |
| SEC-ADMIN-002 | `admin_audit` append-only (REVOKE update/delete) | audit tampering | admin-security §5 | TC-025 | P1 | MVP | backend | M1 | PROPOSED |
| SEC-ADMIN-003 | Step-up signature per aksi destruktif | session abuse | admin-security §3 | TC-025, TC-026 | P1 | MVP | backend | M1 | PROPOSED |
| SEC-INFRA-001 | VPS hardening checklist (SSH key-only, fw, unattended-upgrades) | server compromise | infrastructure-security §2 | review + scan | P0 | MVP | infra | M1 | PROPOSED |
| SEC-INFRA-002 | DB tidak expose publik; app role non-superuser | DB compromise | infrastructure-security §2, database-security §2 | review | P0 | MVP | infra | M1 | PROPOSED |
| SEC-DB-001 | Backup harian → storage terpisah | data loss | disaster-recovery | cron log | P0 | MVP | infra | M1 | PROPOSED |
| SEC-DB-002 | Tidak ada secret di DB; minim PII | leak | database-security §1 | review | P0 | MVP | backend | M1 | DECIDED |
| SEC-DB-003 | `admin_audit` immutable | tampering | admin-security §5 | TC-025 | P1 | MVP | backend | M1 | PROPOSED |
| SEC-DB-004 | Restore drill sukses terdokumentasi | backup palsu | disaster-recovery | drill | P0 | MVP | infra | M4 | BLOCKED |
| SEC-CICD-001 | Branch protection + audit gates + lockfile | supply chain | cicd-security §2 | CI config review | P0 | MVP | infra | M1 | BLOCKED |
| SEC-CICD-002 | Secret scanning (gitleaks) di CI + `.gitignore` ketat; tidak ada kredensial usable di repo | secret leak | secrets-and-gitignore | gitleaks + review | P0 | MVP | infra | M1 | READY-FOR-IMPLEMENTATION |
| SEC-CICD-003 | Proteksi 3 branch (`dev`/`testnet`/`mainnet`); merge/deploy ke `testnet`/`mainnet` wajib approval + tanya user | rilis tak sah | git-workflow §3/§8 | review konfigurasi branch | P0 | MVP | infra | M1 | BLOCKED |
| SEC-IR-001 | Playbook IR + drill pause & restore sebelum mainnet | unprepared incident | incident-response §4 | drill evidence | P0 | mainnet gate | security | M4 | PROPOSED |
| SEC-IR-002 | Kontak darurat + akses kedua (VPS/DM) | unprepared incident | incident-response §4 | dokumentasi | P1 | MVP | security | M1 | PROPOSED |
| SEC-IR-003 | Kanal komunikasi status resmi siap | unprepared incident | incident-response §4 | dokumentasi | P1 | MVP | security | M1 | PROPOSED |
| SEC-KEY-001 | Owner key tidak pernah di app/CI/git | key theft | key-management §2 | review | P0 | MVP | security | M1 | DECIDED |
| SEC-KEY-002 | Mainnet: ganti single-key → **Sputnik DAO V2 council 2-of-3** + timelock; pause via guardian `pause_callers` — **rujukan silang ke SEC-CONTRACT-012 (register utama)** | key theft | key-management §3, §3a | setup DAO + drill | P0 | mainnet gate | security | M4 | DECIDED |

## Catatan register

- **Enum status**: hanya 5 nilai di atas. Fase (`MVP` / `fase 2` / `mainnet gate`) dipisahkan ke kolom sendiri — bukan bagian status. `BLOCKED` (SEC-DB-004) berarti butuh prasyarat infra jalan.
- **`RESEARCHING`** tersedia untuk requirement yang butuh riset lanjut; saat ini belum ada yang memakainya.
- **Kolom Owner** = peran pemilik (bukan orang); MVP = satu orang memegang banyak peran. **Kolom Target** = milestone penutup (M1 MVP / M2 fase 2 / M3 ekosistem / M4 mainnet gate) — lihat [tasks/milestones.md](../../tasks/milestones.md). Bila requirement lintas-milestone, dipakai milestone gate terakhir.
- **Verifikasi ↔ test**: sel Verifikasi yang punya test konkret kini menunjuk ID TC-* di [testing/test-cases.md](../testing/test-cases.md); yang belum punya test tetap deskriptif (`review`, `drill`, `scan`) sampai test dibuat.
- **Rujukan silang**: SEC-CONTRACT-012 = register utama untuk Sputnik DAO + guardian; SEC-KEY-002 merujuk ke sana (bukan duplikat). SEC-FE-002 merujuk SEC-ORDER-003.
- **Rujukan dari dokumen kontrol**: requirement yang belum dikutip di dokumen kontrol lain (`SEC-CONTRACT-009/010/012`, `SEC-FE-001`, `SEC-ADMIN-001/002`, `SEC-INFRA-001/002`, `SEC-DB-002`, `SEC-KEY-001`) dikutip di dokumen pemiliknya pada kolom **Ref** — jangan menghapus ID ini karena tidak muncul di dokumen lain.
- **Naik status sebelum task**: requirement P0 yang masih `PROPOSED` (mis. SEC-ORDER-003) wajib naik ke `DECIDED`/`READY-FOR-IMPLEMENTATION` sebelum task terkait dimulai. (Ronde 16: SEC-CONTRACT-002/009/010 naik ke DECIDED — desain lengkap di contract-architecture + ADR-002/003/008; DECIDED berarti desain terkunci, bukan sudah diimplement.)
- **Ronde 18 (TASK-001 scaffold) — status SEC-CICD**: `SEC-CICD-002` → **READY-FOR-IMPLEMENTATION** (gitleaks + `.gitignore` ketat + allowlist `.gitleaks.toml` + job CI sudah ada dan **hijau di `dev`**; verifikasi = run CI [37627825490](https://github.com/keydeveloping/nearsea/actions/runs/37627825490)). `SEC-CICD-001` & `SEC-CICD-003` → **BLOCKED**: bagian audit-gate + lockfile sudah terpenuhi, tetapi **branch protection belum dikonfigurasi** (butuh setting repo GitHub; dikerjakan di TASK-031 — remote sudah ada sejak ronde 18). Tidak ada status `IMPLEMENTED` — enum tetap 5 nilai.

## Prosedur: menambah SEC requirement baru (ronde 16)

```text
1. CEK DUPLIKAT  — baca register di atas; bila sudah ada yang mencakup, PERLUAS baris itu
                   (jangan buat ID baru untuk threat yang sama).
2. TENTUKAN ID   — lanjutkan nomor per namespace (contoh: namespace SEC-CONTRACT, ambil nomor
                   berikutnya yang belum dipakai); ID = stabil & tidak pernah dipakai ulang.
3. ISI 10 KOLOM  — ID, Requirement (satu kalimat tegas), Threat, Ref (dokumen kontrol pemilik),
                   Verifikasi (TC-* bila ada, else deskriptif), Priority (P0/P1), Fase
                   (MVP/fase 2/mainnet gate), Owner (peran), Target (milestone), Status (enum).
4. STATUS AWAL   — PROPOSED (desain belum di-lock) atau RESEARCHING (butuh riset).
5. SINKRON       — dokumen kontrol di Ref WAJIB benar-benar memuat kontrolnya (bukan janji);
                   update DOCUMENTATION-MAP bila dokumen baru; log di riwayat sinkronisasi.
6. NAIK STATUS   — PROPOSED → DECIDED hanya setelah desain lengkap di dokumen kontrol;
                   DECIDED → READY-FOR-IMPLEMENTATION setelah rencana test jelas (TC dibuat).
7. RETIRE        — requirement tak relevan TIDAK dihapus; coret (~~) + alasan (pola gap-analysis).
```
