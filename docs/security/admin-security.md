# Admin Security

> Keamanan lapisan administrasi off-chain (panel /admin + aksi privileged). On-chain privileged ops ada di access-control-matrix.md (owner key).

## 1. Privileged authentication

- Login: signature wallet atas challenge (domain + scope `admin` + nonce) — wallet-authentication.md.
- **Allowlist admin di DB** (ronde 11) — bukan hardcode; perubahan allowlist = aksi SUPER_ADMIN dengan audit.
- Faktor autentikasi: signature wallet = possession factor (**FACT**). Faktor kedua (mis. TOTP) = PROPOSED mainnet; MVP diterima 1 faktor dengan allowlist kecil (≤3 akun) + semua aksi ber-audit.

## 2. Session policy (admin lebih ketat dari user)

- TTL token admin: 15 menit, TANPA silent refresh (re-login manual).
- IP binding longgar (log perubahan IP antar request → warning).
- Logout = revoke server-side.

## 3. Step-up authorization

- Aksi destruktif (laporan, badge verified, blocklist, allowlist admin) → **re-sign challenge spesifik aksi** (`Action: <salah satu dari 9 aksi kanonik — §11>`, `Target: ...`, `Nonce: ...`) baru dieksekusi. (SEC-ADMIN-003)
- Aksi read-only antrean → session cukup.

## 4. Approval workflows

- MVP: keputusan admin tunggal + audit trail.
- Mainnet PROPOSED: hide berdampak luas (koleksi volume tinggi) butuh 2 admin approval; perubahan allowlist admin butuh 2 approval.

## 5. Audit logging (wajib, append-only)

```text
admin_audit(id, actor_account, action, target_type, target_id, reason, decision, created_at)
-- REVOKE UPDATE/DELETE dari app role (SEC-DB-003)
```

- Setiap keputusan wajib `reason` bebas-text (akuntabilitas).
- Log akses antrean juga (read audit ringan).

## 6. Emergency / break-glass

- On-chain emergency = owner key (pause) — jalur tersendiri (key-management.md, incident-response.md).
- Off-chain emergency: SUPER_ADMIN manual SQL (MVP) — setiap penggunaan wajib entri `admin_audit` manual + postmortem.
- Kehilangan akses admin (semua allowlist hilang): recovery via direct DB access (SSH) — **skenario ini terdaftar di playbook IR** (incident-response.md §2 "Loss of off-chain admin access").

## 7. Catastrophic admin ops (identifikasi)

1. Menambah admin jahat ke allowlist → mitigasi: approval 2-admin (mainnet), audit.
2. Mass-hide seluruh marketplace → mitigasi: opsi per-item, tak ada "hide all".
3. Menghapus/mengedit audit trail → mitigasi: REVOKE DB + backup audit.
4. Owner key (on-chain) disalahgunakan → di luar admin panel; lihat catastrophic-failure-scenarios.md.

## 8. Runbook allowlist admin (seed / onboarding / offboarding)

Allowlist hidup di tabel `admin_allowlist` (bukan hardcode, ronde 11). Perubahan = aksi SUPER_ADMIN ber-audit.

```sql
-- Skema minimum (tipe eksplisit).
CREATE TABLE admin_allowlist (
  id             bigserial PRIMARY KEY,
  account_id     text        NOT NULL UNIQUE,     -- NEAR account (named/implicit/0x)
  role           text        NOT NULL,            -- ADMIN | SUPPORT | SUPER_ADMIN
  status         text        NOT NULL DEFAULT 'active', -- active | suspended | revoked
  added_by       text        NOT NULL,
  added_at       timestamptz NOT NULL DEFAULT now(),
  revoked_by     text,
  revoked_at     timestamptz,
  note           text
);
```

**Seed awal (MVP, manual via SSH — bukan panel):**

```text
1. Tentukan akun admin (named account khusus, mis. nearsea-admin.testnet), BUKAN wallet pribadi.
2. Verifikasi kepemilikan: login signature sekali + cek `view_access_key_list` (SEC-AUTH-004).
3. INSERT baris role=ADMIN, status=active, added_by=<PLATFORM_OWNER>, note='seed MVP'.
4. Tulis entri `admin_audit` manual: action='allowlist_seed', target=akun, reason.
5. Uji: akun bisa login scope admin; akun di luar allowlist ditolak (TC-026).
```

**Onboarding (mainnet, 2-admin approval):**

```text
1. Calon admin membuat akun + kunci (generation offline; key-management.md §4).
2. Calon login scope admin → signature terverifikasi (SEC-AUTH-004).
3. SUPER_ADMIN ajukan `allowlist_add`; approver kedua menyetujui (lihat §10).
4. INSERT status=active + admin_audit(action='allowlist_add').
5. Verifikasi: `SELECT` allowlist; akun berhasil aksi read-only antrean.
6. Cabut akses yang tidak perlu (tidak ada role default); catat tanggal/aktor.
```

**Offboarding (anggota keluar / dugaan kompromi):**

```text
1. UPDATE status='revoked', revoked_by, revoked_at.
2. Revoke SEMUA sesi akun tersebut (session-revocation store §9) + denylist jti.
3. Cabut akses lain: SSH key VPS, GitHub deploy key, Telegram admin, akses DB, DAO council bila ada.
4. admin_audit(action='allowlist_revoke', reason='offboarding'|'kompromi').
5. Rotasi secret yang pernah diakses akun (secrets-and-gitignore §6) bila alasan = kompromi.
6. Verifikasi: login scope admin gagal (403 FORBIDDEN_ADMIN).
```

## 9. Session-revocation store (desain)

TTL admin 15 menit TANPA silent refresh (§2) → revokasi harus instan dan server-side.

```sql
-- Denylist berbasis jti; dibaca setiap verifikasi token.
CREATE TABLE session_revocations (
  jti          text        PRIMARY KEY,     -- id token (uuid) dari access/refresh
  account_id   text        NOT NULL,
  scope        text        NOT NULL,
  reason       text        NOT NULL,        -- logout | offboarding | kompromi | rotasi
  revoked_at   timestamptz NOT NULL DEFAULT now(),
  expires_at   timestamptz NOT NULL         -- = exp token; baris boleh dibersihkan setelahnya
);
CREATE INDEX idx_revocations_account ON session_revocations (account_id);
```

- **Aturan**: setiap request ber-scope admin memeriksa `jti` di denylist (cache in-memory TTL 60 detik + fallback DB).
- Revokasi massal per akun: `INSERT … SELECT` semua sesi aktif akun → semua token langsung invalid.
- Baris `expires_at < now()` dibersihkan harian (bukan arsip permanen); jejak keputusan tetap di `admin_audit`.
- `logout` = revoke satu `jti`; offboarding/kompromi = revoke seluruh akun.

## 10. Urutan approval 2-admin (persis)

Berlaku untuk aksi berisiko (hide koleksi volume tinggi, perubahan allowlist) — PROPOSED mainnet.

```text
PRE  : approver A ≠ approver B (dua akun allowlist berbeda, tidak boleh sama).
1. A membuka aksi → server membuat `approval_request(id, action, target, requester=A,
   status='pending', expires_at=now()+24 jam, nonce)`.
2. A menandatangani step-up untuk `approval_request.id` → tercatat `approval_vote(A, approve)`.
3. B (berbeda akun) membuka request yang sama → server memverifikasi B ∉ {A}.
4. B menandatangani step-up untuk id yang sama → `approval_vote(B, approve)`.
5. Server mengecek: jumlah approve ≥ 2 DAN approver unik DAN belum expired.
6. EKSEKUSI aksi sekali (idempoten via approval_request.id); tandai status='executed'.
7. admin_audit(action=<aksi>, reason, approvers=[A,B], request_id).
```

- Satu orang tidak bisa memenuhi kedua suara (dicek `requester ≠ approver`).
- Request kedaluwarsa (24 jam) → batal; harus diajukan ulang.
- Tolak: satu `reject` → status='rejected', aksi tidak jalan.
- MVP (≤3 admin): keputusan tunggal + audit; urutan 2-admin ini **PROPOSED mainnet**.

## 11. Skema audit dengan tipe eksplisit

> **SSOT DDL = [database-schema.md](../database/database-schema.md) §DDL audit.** Skema di bawah adalah ringkasan untuk pembaca dokumen ini — jangan mengubah kolom di sini.

```sql
-- Ringkasan; DDL lengkap + approval_request ada di database-schema.md.
admin_audit(
  id            bigint GENERATED ALWAYS AS IDENTITY,  -- BUKAN bigserial (aturan repo)
  actor_account text        NOT NULL,   -- akun admin pelaku
  action        text        NOT NULL,   -- 9 aksi kanonik: report_decide | verified_set |
                                        -- blocklist_add | blocklist_remove | allowlist_add |
                                        -- allowlist_revoke | appeal_decide | approval_execute | break_glass
  target_type   text,                   -- report | collection | token | account | blocklist | allowlist
  target_id     text,
  reason        text        NOT NULL,   -- wajib, bebas-text (akuntabilitas)
  decision      text,                   -- upheld | overturned | hidden | ignored | verified | ...
  created_at    timestamptz NOT NULL
);
-- Kaitan ke approval 2-admin: table approval_request(action, target_id, requested_by, approved_by)
-- di database-schema.md; audit entry untuk approval_execute menyimpan approval_request.id di `reason`
-- atau payload decision — TIDAK sebagai kolom tambahan (jaga skema kanonik).

CREATE INDEX idx_admin_audit_actor  ON admin_audit (actor_account, created_at DESC);
CREATE INDEX idx_admin_audit_target ON admin_audit (target_type, target_id);

-- Append-only: REVOKE UPDATE/DELETE dari app role (SEC-DB-003).
REVOKE UPDATE, DELETE, TRUNCATE ON admin_audit FROM nearsea_app;
GRANT  INSERT, SELECT ON admin_audit TO nearsea_app;
```

- Tidak ada kolom yang boleh NULL untuk `actor_account`, `action`, `reason`, `created_at`.
- Read audit ringan (akses antrean) = baris terpisah dengan `action='queue_read'` bila diaktifkan (opsional).

## 12. Status

- Allowlist DB + step-up + audit — MVP (SEC-ADMIN-001/002/003).
- TOTP (faktor kedua) — **PROPOSED mainnet**; MVP menerima 1 faktor dengan allowlist ≤3 akun + audit penuh. Aktivasi TOTP menaikkan status SEC-ADMIN-001 terkait sebelum mainnet.
- Approval 2-admin + panel kelola allowlist — **PROPOSED mainnet** (MVP: manual SQL/script + log manual).
