# Feature — Report & Moderasi (Report, Badge Verified, Blocklist, Appeal)

> Moderasi **display-layer**: takedown hanya menyembunyikan dari discovery — chain tidak pernah disentuh (ADR-006). Satu-satunya backend MVP (PostgreSQL di VPS — ADR-009).

## Objective

Sistem lapor dan moderasi: user melaporkan koleksi/token bermasalah → antrean review admin → keputusan (`hide_target` | `ignore`) yang hanya berdampak pada **layer tampilan**; ditambah kurasi badge verified dan blocklist display-layer. Kepemilikan & transaksi on-chain tidak pernah berubah (✅ DIPUTUSKAN — [ADR-006](../decisions/ADR-006-open-collections-factory.md), ronde 3 + revisi ronde 5).

## Preconditions

| Aktor | Syarat |
|---|---|
| Reporter | Login wallet dengan session scope `report` ([authentication.md](../api/authentication.md)). |
| Admin | Akun ada di **allowlist DB** `admin_allowlist` (**≤ 3 akun di MVP**) + session scope `admin` (TTL 15 menit, **tanpa silent refresh**) + **step-up signature** per aksi destruktif (SEC-ADMIN-003). |
| Target | Koleksi/token valid — `contract_account_id` (+ `token_id` opsional); tak ada → `404 NOT_FOUND_COLLECTION`/`NOT_FOUND_TOKEN`. |

## Flow — Report (user)

```text
1. User klik "Report" pada koleksi/token → Report Modal terbuka
2. Pilih alasan (enum — ⏳ open-by-design; kandidat: scam / impersonation /
   plagiarism / nsfw / other) + teks bebas opsional (panjang maks = ⏳ open-by-design)
3. POST /api/v1/reports  { contract_account_id, token_id?, reason }
   (Auth: session scope `report`)
4. Server validasi target + rate limit + idempotency:
   - rate limit 5/akun/hari           → 429 RATE_LIMITED + Retry-After
   - unik (account_id, target, hari kalender) → 409 CONFLICT_REPORT_EXISTS
5. 201 → report berstatus `open`, masuk antrean review admin
```

- State machine report: `open → resolved (hidden | ignored)`.
- Report **append-only** — reporter tidak bisa mengubah/menarik isi report.
- Idempotency ditegakkan constraint DB unik `(account, target, hari kalender)` — bukan sekadar header ([api-security-architecture.md](../security/api-security-architecture.md) §1).

## Flow — Admin (keputusan report)

```text
1. Admin login wallet → cek allowlist DB (bukan sekadar scope JWT — SEC-ADMIN-001)
   → session scope `admin` (15 menit, tanpa silent refresh; habis → re-login + step-up ulang)
2. Buka antrean: GET /api/v1/admin/reports
3. Pilih keputusan → PATCH /api/v1/admin/reports/:id
   { action: "hide_target" | "ignore", reason(wajib), step_up { action: "report_decide", ... } }
4. Server verifikasi step-up signature (nonce segar, belum expire, action/target cocok)
5. Efek:
   - hide_target → target (KOLEKSI/TOKEN yang dilaporkan — bukan reportnya)
     disembunyikan dari discovery/listing FE; chain TIDAK tersentuh
   - ignore → report berstatus ignored; tanpa efek display
6. Baris admin_audit (actor, action, target, reason) tercatat — append-only (SEC-ADMIN-002)
```

- Jika report sudah diputuskan admin lain → `409 CONFLICT_ALREADY_DECIDED`.
- Tidak ada event on-chain — jejak audit sepenuhnya di DB (`admin_audit`).

## Flow — Badge Verified (kurasi admin)

1. Admin menilai koleksi (kurasi manual) → `PATCH /api/v1/admin/collections/:id/verified` `{ verified: true|false, reason, step_up { action: "verified_set", ... } }`.
2. Badge = flag DB display-layer pada koleksi; dirender di discovery/detail (AC-BADGE-1/2).
3. **Kriteria pemberian badge = ⏳ open-by-design** (target keputusan: Fase 2).
4. Bukan admin / tanpa step-up → ditolak `403`/`401` (AC-BADGE-3).

## Blocklist (display-layer)

| Aspek | Aturan |
|---|---|
| Target | `target_type`: `collection` \| `token` (`account` = ⏳ open-by-design — [database-schema.md](../database/database-schema.md) § Enumerasi) |
| Tambah | `POST /api/v1/admin/blocklist` `{ target_type, target_id, reason }` — step-up `blocklist_add` |
| Hapus | `DELETE /api/v1/admin/blocklist/:id` — step-up `blocklist_remove` |
| Lihat | `GET /api/v1/admin/blocklist` — cukup session admin (read-only) |
| Efek | Hanya display-layer — target disembunyikan dari discovery; chain & settlement tidak tersentuh |
| Duplikat | `UNIQUE (target_type, target_id)` → `409 CONFLICT_ALREADY_BLOCKED` |

## Appeal (banding)

Alur banding false-positive (trigger → submit → antrean → keputusan `upheld`/`overturned`, aksi kanonik `appeal_decide`) **dimiliki [fraud-and-abuse.md](../security/fraud-and-abuse.md) §7**. SLA: respons pertama ≤ 3 hari kerja, keputusan ≤ 7 hari kerja (PROPOSED — tabel klasifikasi §9). Banding tidak pernah menyentuh settlement on-chain.

## UI

- **Report Modal** — komponen `Report Modal` (varian form/pending/success/error; alasan = enum): [04-ux-ui-spec.md](../04-ux-ui-spec.md) § Kontrak Komponen. Tombol Report di detail koleksi/token.
- **`/admin`** — panel admin (guard: URL langsung + 403 di luar allowlist): tab **Reports** (antrean + aksi Hide/Ignore), **Blocklist**, **Verified**; aksi destruktif selalu minta konfirmasi step-up ("Confirm with your wallet"). Wireframe: [04-ux-ui-spec.md](../04-ux-ui-spec.md) § Halaman 9.
- Copy i18n kunci `admin.*` + `errors.*` ada di [02-product-requirements.md](../02-product-requirements.md) § ADMIN / MODERATION.

## API

> Blok Moderasi dimiliki [endpoints.md](../api/endpoints.md) § Moderasi — tabel ini hanya ringkasan.

| Endpoint | Auth | Aksi step-up |
|---|---|---|
| `POST /api/v1/reports` | Bearer scope `report` | — (rate 5/akun/hari) |
| `GET /api/v1/admin/reports` | `admin` + allowlist DB | — |
| `PATCH /api/v1/admin/reports/:id` | `admin` + allowlist DB | `report_decide` |
| `PATCH /api/v1/admin/collections/:id/verified` | `admin` + allowlist DB | `verified_set` |
| `GET /api/v1/admin/blocklist` | `admin` + allowlist DB | — |
| `POST /api/v1/admin/blocklist` | `admin` + allowlist DB | `blocklist_add` |
| `DELETE /api/v1/admin/blocklist/:id` | `admin` + allowlist DB | `blocklist_remove` |

- Kosakata aksi step-up = **9 nilai kanonik** (SSOT: [admin-security.md](../security/admin-security.md) §11; template signature: [wallet-authentication.md](../security/wallet-authentication.md) §3). Keputusan `report_decide` dipakai untuk menerima/menolak laporan. Semua contoh di repo sudah diselaraskan ke daftar ini (ronde 16).
- Bentuk transport step-up (header vs body) = ⏳ open-by-design ([endpoints.md](../api/endpoints.md) § Endpoint MVP — Admin).

## Error Cases

> Kode = registry [error-handling.md](../development/error-handling.md) §3.

| Kasus | Kode | HTTP |
|---|---|---|
| Aksi destruktif tanpa step-up signature | `ADMIN_STEPUP_REQUIRED` | 403 |
| Step-up signature tidak valid / expired / nonce terpakai | `ADMIN_STEPUP_INVALID` | 401 |
| Sudah report target sama di hari kalender sama | `CONFLICT_REPORT_EXISTS` | 409 |
| Report sudah diputuskan admin lain | `CONFLICT_ALREADY_DECIDED` | 409 |
| > 5 report/akun/hari (atau > 30 aksi admin/menit) | `RATE_LIMITED` (+ `Retry-After`) | 429 |
| Akun tidak ada di allowlist admin DB | `FORBIDDEN_ADMIN` | 403 |
| Alasan/target tidak valid | `INVALID_REASON` / `INVALID_TARGET` | 400 |
| Target koleksi/token tidak ditemukan | `NOT_FOUND_COLLECTION` / `NOT_FOUND_TOKEN` | 404 |

## Security

| Kontrol | Isi | Rujukan |
|---|---|---|
| SEC-ADMIN-001 | Admin = allowlist DB + signature scope `admin` (bukan hardcode; scope JWT saja tidak cukup) — ✅ DECIDED arah, rincian PROPOSED | [security-requirements.md](../security/security-requirements.md); [admin-security.md](../security/admin-security.md) §1 |
| SEC-ADMIN-002 | `admin_audit` append-only — `REVOKE UPDATE, DELETE` di DB | [admin-security.md](../security/admin-security.md) §5 |
| SEC-ADMIN-003 | Step-up signature per aksi destruktif; kunci diverifikasi ulang saat aksi | [admin-security.md](../security/admin-security.md) §3 |
| Allowlist ≤ 3 (MVP) | Kompensasi 1-faktor: allowlist kecil + audit penuh; TOTP = PROPOSED mainnet | [admin-security.md](../security/admin-security.md) §2 |
| Sesi admin | TTL 15 menit, tanpa silent refresh; perubahan allowlist → revoke sesi terkait | [authentication.md](../api/authentication.md) § Sesi |
| Chain tak tersentuh | Takedown/badge/blocklist/appeal hanya layer tampilan; kepemilikan & transaksi on-chain tetap (AC-REPORT-6) | [ADR-006](../decisions/ADR-006-open-collections-factory.md) |
| Rate limit | `POST /reports` 5/akun/hari; `POST/PATCH/DELETE /admin/*` 30/menit per admin | [api-security-architecture.md](../security/api-security-architecture.md) §1 |

## Acceptance Criteria

- Definisi "selesai" fitur ini = AC lolos di [acceptance-criteria.md](../testing/acceptance-criteria.md):
  - § Report System — **AC-REPORT-1..6** (report masuk antrean, idempoten per hari, rate limit, 404 target, auth/scope, takedown display-layer saja).
  - § Admin Panel — **AC-ADMIN-1..6** (allowlist, 403 di luar allowlist, step-up wajib, audit append-only, sesi 15 menit tanpa silent refresh, `ignore` tanpa efek display).
  - § Verified Badge — **AC-BADGE-1..3** (badge tampil/sembunyi, set verified butuh admin + step-up).
