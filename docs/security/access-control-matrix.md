# Access Control Matrix

> Register formal role & operasi privileged. Melengkapi `permissions.md` (ringkas). Prinsip prompt: tidak ada "admin" catch-all — setiap operasi punya aktor, auth, approval, audit.

## Roles

| Role | Scope | Ada di fase |
|---|---|---|
| USER | wallet terhubung, aksi atas asetnya sendiri | MVP |
| CREATOR | USER + deploy koleksi via factory + kelola phases/allowlist miliknya | MVP |
| PLATFORM_OWNER (on-chain) | pemegang owner key market/factory: pause, fee_bps, treasury | MVP |
| ADMIN (off-chain) | moderasi display-layer: reports, verified, blocklist — via allowlist DB + signature | MVP |
| SUPPORT (off-chain) | baca report/kontak user, tanpa aksi destruktif | PROPOSED (fase 2+) |
| FINANCE | tarik treasury / rekonsiliasi fee | PROPOSED (mainnet) |
| SECURITY | pause + break-glass (mainnet: guardian `pause_callers`, terpisah dari owner-DAO) | PROPOSED (mainnet) |
| SUPER_ADMIN | kelola allowlist admin, kebijakan | PROPOSED (mainnet; MVP = manual/script) |

## Kredensial & tipe kunci per role

> Setiap role memakai **satu kredensial utama**; tidak ada role dengan dua jenis kredensial otoritatif. Skema kunci NEAR (ed25519/secp256k1/ml-dsa-65) berlaku untuk kredensial wallet; verifikasi ownership skema-agnostik via `view_access_key_list` ([wallet-authentication.md](./wallet-authentication.md) §1, SEC-AUTH-004).

| Role | Kredensial utama | Tipe kunci / mekanisme | AuthN | Masa hidup | Bisa di-revoke |
|---|---|---|---|---|---|
| Guest | — | — | — | — | — |
| USER | access key wallet | ed25519 / secp256k1 / ml-dsa-65 (full-access atau function-call) | tx signature (protokol) | per tx | user hapus key |
| USER (API) | signature wallet + session | NEP-413 (utama) / custom fallback | nonce 32B TTL 5m + JWT 15m | nonce sekali; JWT 15m; refresh ≤12 jam | logout/denylist |
| CREATOR | = access key wallet (owner koleksi) | sama dengan USER | tx signature | per tx | user |
| ADMIN | wallet signature + **allowlist DB** + scope `admin` | ed25519/secp256k1/ml-dsa-65 | login NEP-413 + step-up per aksi destruktif | token 15m, **tanpa silent refresh** | cabut dari allowlist + revoke sesi |
| SUPER_ADMIN | allowlist (MVP: SQL/SSH) | kredensial DB/SSH + wallet (mainnet panel) | MVP: akses SSH + audit manual | sampai rotasi | rotasi SSH/DB |
| SUPPORT | session `scope:support` | wallet signature | NEP-413 | 15m | revoke sesi |
| PLATFORM_OWNER (MVP) | full-access owner key | ed25519 (via near-cli) | tx signature | sampai rotasi | rotasi owner |
| PLATFORM_OWNER (mainnet) | Sputnik DAO V2 council | 2-of-3 member keys | proposal DAO | per proposal + timelock | ganti member via proposal |
| SECURITY / guardian | guardian key | ed25519 (daftar `pause_callers`) | tx signature | sampai rotasi | ganti `pause_callers` |
| FINANCE (mainnet) | DAO proposal | member key DAO | proposal | per proposal | ganti member |

**Catatan**: full-access key vs function-call key untuk `signMessage` API → ⏳ open-by-design ([api/authentication.md](../api/authentication.md)). MVP: kredensial on-chain dan off-chain **terpisah** — owner key tidak pernah dipakai login API.

## RACI — operasi privileged

> R = Responsible (mengeksekusi), A = Accountable (final), C = Consulted, I = Informed. MVP sering menggabungkan R/A pada satu orang (risiko diterima testnet — [key-management.md](./key-management.md) §2).

| Operasi | USER | CREATOR | ADMIN | SUPER_ADMIN | SECURITY | FINANCE | PLATFORM_OWNER/DAO |
|---|---|---|---|---|---|---|---|
| Pause kontrak | I | I | I | I | **R/A** (guardian) | I | C (MVP: R/A) |
| Unpause | I | I | I | I | C | I | **R/A** |
| Update `fee_bps` | I | I | — | C | C | C | **R/A** |
| Update treasury | I | I | — | C | C | C | **R/A** |
| Rekonsiliasi fee treasury (off-chain) | I | I | — | I | C | **R** | **A** |
| Upgrade kontrak | I | I | — | C | C | C | **R/A** |
| Moderasi report/verified/blocklist | I | C | **R/A** | I | I | — | — |
| Kelola allowlist admin | I | I | C | **R/A** | C | — | — |
| Break-glass SQL (IR) | I | I | I | **R/A** | C | I | C |
| Drill pause/unpause & restore | I | I | I | C | **R** | C | **A** |

## Privileged operations (on-chain)

| Operasi | Aktor | AuthN | Guard | Approval | Audit | Emergency |
|---|---|---|---|---|---|---|
| `pause` | MVP: PLATFORM_OWNER; mainnet: guardian SECURITY (`pause_callers` terpisah — key-management §3) | owner key / guardian key | — | tunggal (MVP); mainnet: pause tunggal (reaksi cepat) | event `market_pause` | **ini kontrol daruratnya** |
| `unpause` | PLATFORM_OWNER (DAO proposal di mainnet) | sama | — | mainnet timelock | event `market_unpause` | — |
| `update_fee_bps` | PLATFORM_OWNER | MVP: owner key; mainnet: DAO proposal | `≤ MAX_FEE_BPS` (immutable — INV-004) | MVP tunggal; mainnet timelock 24 jam | event `fee_update` | — |
| `update_treasury` | PLATFORM_OWNER | MVP: owner key; mainnet: DAO proposal | valid account | MVP tunggal; mainnet timelock | event `treasury_update` | — |
| Upgrade kontrak (redeploy) | PLATFORM_OWNER | MVP: owner key; mainnet: DAO proposal 2-of-3 | release checklist (deployment.md) + reproducible build NEP-330 | mainnet: timelock + publish wasm hash | code hash on-chain | — |
| ~~Withdraw treasury~~ | — | — | **Dihapus ronde 23** — fee ditransfer langsung ke treasury saat settlement; tidak ada akumulasi di kontrak ([contracts/market.md](../contracts/market.md) §4a) | — | — | — |
| Set phase / allowlist koleksi | CREATOR | signature tx creator (owner koleksi) | harus owner koleksi + storage pre-deposit | — | event launchpad | — |

## Privileged operations (off-chain)

| Operasi | Aktor | AuthN | AuthZ | Audit | Catatan |
|---|---|---|---|---|---|
| Decide report (hide/ignore) | ADMIN | signature wallet + session `scope:admin` | allowlist DB | `admin_audit` (schema § di bawah) | step-up: re-sign per keputusan hide (SEC-ADMIN-003) |
| Set verified badge | ADMIN | sama | allowlist | sama | takedown badge = keputusan sensitive |
| Edit blocklist tampilan | ADMIN | sama | allowlist | sama | chain tidak tersentuh |
| Kelola allowlist admin | SUPER_ADMIN | MVP: manual SQL/script (log manual); mainnet: panel + 2-admin approval | — | DB audit | MVP risk accepted + dikompensasi VPS hardening |
| Baca data user | SUPPORT | session `scope:support` | read-only | query log | PROPOSED fase 2+ |

### Skema `admin_audit` (kanonik)

> **SSOT skema = [admin-security.md](./admin-security.md) §5** — tabel di bawah hanya salinan rujukan; perubahan wajib dilakukan di admin-security.md lebih dulu. Semua operasi off-chain privileged WAJIB menulis baris ke tabel ini (append-only).

```sql
admin_audit(
  id,            -- PK
  actor_account, -- wallet/account pelaku (bukan nama tampilan)
  action,        -- 9 aksi kanonik (SSOT: admin-security §11): report_decide | verified_set |
                 -- blocklist_add | blocklist_remove | allowlist_add | allowlist_revoke |
                 -- appeal_decide | approval_execute | break_glass
  target_type,   -- report | collection | token | account
  target_id,     -- id target
  reason,        -- bebas-text, WAJIB (akuntabilitas)
  decision,      -- outcome (mis. hidden | ignored | granted | revoked)
  created_at     -- now()
)
-- REVOKE UPDATE/DELETE dari app role (SEC-DB-003)
```

> **Rekonsiliasi**: definisi lama yang lebih ringkas (`id, actor, action, target, reason, at`) **digantikan** oleh skema di atas agar konsisten dengan [admin-security.md](./admin-security.md) §5 dan [database-security.md](./database-security.md) §3. Tidak ada kolom `target` tunggal — dipisah `target_type` + `target_id`.

## Break-glass (prosedur darurat)

> Break-glass = jalur darurat **di luar alur normal**. Setiap penggunaan WAJIB: (1) tercatat di `admin_audit`, (2) postmortem ([incident-response.md](./incident-response.md)), (3) rotasi kredensial terkait.

```text
TRIGGER yang sah:
  - Owner key compromise (CS-1) / upgrade jahat (CS-9)
  - Semua akses admin hilang ("Loss of off-chain admin access")
  - Exploit aktif butuh pause tanpa menunggu DAO

LANGKAH (on-chain emergency — pause):
  1. Guardian `pause_callers` (mainnet) / owner key (MVP) memanggil `pause` SEGERA (single-key, tanpa timelock).
  2. Verifikasi event `market_pause` on-chain; konfirmasi mutasi baru ditolak, cancel/withdraw tetap terbuka.
  3. Bila key diduga bocor: JANGAN pakai seed backup — buat key FRESH, deploy akun/kontrak baru.
  4. Unpause HANYA via DAO 2-of-3 + timelock setelah patch terverifikasi.

LANGKAH (off-chain — akses admin hilang):
  1. SSH ke VPS (key kedua; SEC-IR-002) dengan user OS terpisah dari app.
  2. Rebuild allowlist via SQL manual — INSERT baris admin + TULIS entri `admin_audit` manual
     (action=add_admin, reason="break-glass recovery", decision=granted).
  3. Cabut sesi lama (revoke `sessions`) + rotasi `JWT_SECRET`.
  4. Rotasi SSH/DB credential yang mungkin terekspos; verifikasi hanya akun berwenang yang masuk.

LARANGAN:
  - Break-glass TIDAK boleh jadi rutin; TIDAK boleh tanpa audit; TIDAK boleh menghapus/ubah `admin_audit`.
  - Tidak ada "super user" implisit: tetap tunduk least-privilege + SoD.
```

## Pemetaan SEC-ADMIN-*

| Kontrol di dokumen ini | SEC-ID | Verifikasi |
|---|---|---|
| Admin = allowlist DB + signature scope `admin` (bukan hardcode) | SEC-ADMIN-001 | TC-026 (403 di luar allowlist) |
| `admin_audit` append-only (REVOKE UPDATE/DELETE) | SEC-ADMIN-002 | test DB perms + SEC-DB-003 |
| Step-up signature per aksi destruktif | SEC-ADMIN-003 | TC-025, TC-026 |
| Scope token terpisah (`report`/`profile`/`admin`), admin tanpa silent refresh | SEC-AUTH-006 | TC-029 |
| Revokasi sesi saat allowlist berubah | SEC-ADMIN-001/003 | TC-030 (revokasi) |
| Break-glass ber-audit + postmortem | SEC-ADMIN-002, SEC-IR-001 | drill evidence |

## Prinsip

1. **Least privilege**: satu role = satu tujuan; tidak ada inheritance diam-diam.
2. **Separation of duties (mainnet)**: yang bisa pause ≠ yang bisa tarik treasury (SECURITY vs FINANCE).
3. **Semua aksi admin = immutable audit trail** (append-only table).
4. **Break-glass**: owner key = jalan darurat terakhir; penggunaannya wajib postmortem + log (incident-response.md).
