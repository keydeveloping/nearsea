# Wallet Authentication (API off-chain)

> Desain PROTOKOL otentikasi API (report/profil/admin) — bukan kode. Transaksi on-chain TIDAK lewat sini (wallet sign langsung ke chain).
> Keputusan dasar: ADR-011. **Protokol: DECIDED** (ronde 11 + riset 2026-10-01); implementasi menunggu Fase 1 (status detail per item ada di bagian Status).

## 1. Model akun & skema kunci di NEAR (FACT — docs.near.org, diverifikasi 2026-10-01)

| Tipe akun | Format | Kunci tipikal | Implikasi auth |
|---|---|---|---|
| Named (TLA) | `alice.near` / `x.testnet` | ed25519 full/function-call keys | Verifikasi signature ed25519 — jalur utama |
| Named (sub) | `app.alice.near` | sama | sama |
| Implicit | 64 lowercase hex | ed25519 | sama |
| Ethereum-like | `0x` + 40 hex | (mapped key) | Didukung protokol — API harus menerima format ini, verifikasi tetap via RPC |
| Deterministic | `0s` + 40 hex | — | Bukan transfer target; validasi input harus mengenalinya (docs: account-validation) |

- **Skema kunci**: ed25519 (utama) + secp256k1 + **post-quantum ml-dsa-65** (kunci disimpan on-chain sebagai hash `ml-dsa-65-hash:`). Verifikasi ownership via `view_access_key_list` = skema-agnostik → SEC-AUTH-004 aman untuk semua skema.
- **ERC-1271 padanan**: tidak ada standar off-chain verifikasi atas nama contract account → out-of-scope MVP (ASSUMPTION didokumentasikan).

## 2. NEP-413 — DIPUTUSKAN sebagai format utama (riset 2026-10-01)

**FACT (riset web, 2026-10-01)**: NEP-413 ("Wallet API — signMessage") berstatus **Final** di NEPs; didukung hampir semua wallet modern (dipakai luas oleh NEAR Intents/Defuse; Privy menambah dukungan 2025). Payload standar: `{message, nonce, recipient, callbackUrl}`.

- **`recipient`** = domain binding kita; **`nonce`** = 32-byte random (simpan & konsumsi seperti di bawah); **`message`** = teks berisi account + scope + network.
- Custom challenge string (ronde 11) menjadi **fallback** untuk wallet tanpa NEP-413 — format pesan harus tetap berisi domain+network+scope.

## 3. Protokol challenge-response (DECIDED ronde 11, dirinci)

```text
1. FE → GET /api/auth/nonce?account_id=alice.near
2. API menyimpan {nonce, account_id, expires_at=now+5m, used=false} di DB (tabel auth_nonce)
   → balas {nonce: "<32-byte-random-hex>", domain: "nearsea.example", network: "testnet", expires_at}
3. FE meminta wallet sign message (NEP-413 utama; fallback = template teks kanonik):
   "NearSea login\nDomain: nearsea.example\nNetwork: testnet\nAccount: alice.near\nNonce: <nonce>\nExpires: <iso8601>\nScope: report|profile|admin"
4. FE → POST /api/auth/verify {account_id, nonce, message, recipient, signature, public_key?}
   (message + recipient dikirim agar server bisa memcmp penuh — tidak percaya string FE)
5. API verifikasi (URUTAN: validasi dulu, konsumsi nonce belakangan):
   a. message exact-match terhadap template kanonik; domain & scope & network & account cocok
   b. signature valid atas public key yang terikat account_id
      - cek on-chain via RPC view `access_key` (validasi key milik account & aktif) — SEC-AUTH-004
   c. HANYA jika 5a–5b lolos → nonce dikonsumsi atomically (`UPDATE … WHERE used=false`)
      (percobaan gagal TIDAK mengonsumsi nonce; kegagalan dihitung rate-limit per IP+account — 5 gagal/menit → lockout sementara, bukan "per nonce")
6. API terbitkan session token (JWT pendek, TTL 15 menit, refresh ≤ 12 jam) berisi {sub: account_id, scope}
7. Logout = hapus session server-side (jika ada session store) + hapus di client
```

**Template step-up admin (terpisah dari login — S3, kanonik lengkap)**:
`"NearSea admin action\nDomain: …\nNetwork: …\nAccount: …\nAction: <aksi kanonik>\nTarget: <id>\nNonce: …\nExpires: …"` — signature kedua diminta saat aksi destruktif; urutan field persis seperti ini agar exact-match bisa diimplementasi. **Kosakata `<aksi kanonik>` = 9 nilai (SSOT: [admin-security.md](./admin-security.md) §11)**: `report_decide` | `verified_set` | `blocklist_add` | `blocklist_remove` | `allowlist_add` | `allowlist_revoke` | `appeal_decide` | `approval_execute` | `break_glass`.

> **SSOT template.** Dokumen ini adalah **pemilik kanonik** template pesan login & step-up. Dokumen lain (mis. [api/authentication.md](../api/authentication.md)) **merujuk** ke sini, tidak mendefinisikan ulang. Urutan field login: `Domain → Network → Account → Nonce → Expires → Scope`. Field `Nonce` **wajib ada di dalam message** (bukan hanya di payload NEP-413) agar jalur fallback punya binding nonce yang sama.
>
> **Placeholder domain.** Nilai `nearsea.example` pada contoh di atas adalah **placeholder kanonik** — domain produksi belum ditetapkan (open-by-design, [infrastructure.md](../architecture/infrastructure.md)); nilai final diisi saat deploy ([environments.md](../deployment/environments.md)). Implementasi membaca domain dari konfigurasi, bukan hardcode.

## 4. Properti keamanan yang wajib

| Properti | Mekanisme |
|---|---|
| Anti-replay challenge | nonce sekali pakai (atomic consume) + TTL 5 menit |
| Domain binding | `domain` di dalam message yang di-sign → signature dari app lain tidak bisa direplay (analog domain-binding SIWE) |
| Chain binding | MVP: network disiratkan di message (`network: testnet`); transaksi dana tidak pernah lewat API ini → wrong-chain risk rendah; SEC-AUTH-005 |
| Scope separation | token ber-scope (`report`, `profile`, `admin`); admin butuh allowlist + scope `admin` |
| Session rotation | refresh token rotasi setiap pakai (token lama invalid) — SEC-AUTH-006 |
| CSRF | token di header Authorization (bukan cookie) → CSRF tidak relevan; jika pindah ke cookie: SameSite=Strict + CSRF token |
| Phishing resistance | message menampilkan domain + scope dalam bahasa jelas; FE tidak pernah minta seed phrase (FACT: tidak perlu) |

## 5. Rate & abuse

- `nonce` mint: 10/IP/menit + 30/akun/jam.
- `verify`: 10/IP/menit; **5 gagal/menit per IP+account → lockout sementara** (fail-bucket). Percobaan gagal **TIDAK memblokir nonce** — nonce yang belum dikonsumsi tetap valid sampai TTL-nya habis (selaras TC-028 & AC-RATE-2).
- Semua percobaan gagal di-log (account, IP hash, alasan) untuk alert Telegram.

## 6. Skema payload NEP-413 (eksak)

> Bentuk kanonik payload yang dikirim wallet dan yang diverifikasi server. Field **wajib** persis; field opsional ditandai. `message` dimiliki §3 (template kanonik) — di sini hanya bentuk JSON-nya.

```json
{
  "message": "NearSea login\nDomain: nearsea.example\nNetwork: testnet\nAccount: alice.testnet\nNonce: 3f9a1c7d5b2e8f04a6c1d9e7b3f5028c4a7d1e6f9b0c3a5d8e2f4b7c1a9d6e30\nExpires: 2026-10-02T12:05:00Z\nScope: profile",
  "nonce": "3f9a1c7d5b2e8f04a6c1d9e7b3f5028c4a7d1e6f9b0c3a5d8e2f4b7c1a9d6e30",
  "recipient": "nearsea.example",
  "callbackUrl": "https://nearsea.example/auth/callback"
}
```

| Field | Tipe | Wajib | Aturan validasi server |
|---|---|---|---|
| `message` | string (UTF-8) | ya | exact-match template kanonik §3 (memcmp penuh, constant-time); memuat Nonce |
| `nonce` | string 64 hex (32 B) | ya | sama dengan nonce tersimpan; TTL 5 menit; sekali pakai |
| `recipient` | string | ya | == domain terkonfigurasi (bukan `localhost`/domain lain) |
| `callbackUrl` | string URL | tidak | opsional; tidak dipakai untuk otorisasi |

**Request `POST /api/auth/verify`** (yang dikirim FE — server **tidak** mempercayai string FE):

```json
{
  "account_id": "alice.testnet",
  "nonce": "3f9a1c7d…",
  "message": "<persis seperti di atas>",
  "recipient": "nearsea.example",
  "public_key": "ed25519:7Kq3…",
  "signature": "ed25519:4mB9…"
}
```

> `message` + `recipient` **wajib dikirim balik** agar server bisa membandingkan penuh; tanpa itu server tidak bisa menolak mismatch (§3 langkah 5a).

## 7. Algoritma verifikasi (pseudocode)

> Urutan **wajib**: validasi dulu, konsumsi nonce belakangan (SEC-AUTH-001). Byte-level serialisasi NEP-413 dikunci test vector §8.

```text
function verify_login(req):
  n = db.auth_nonce.find(req.nonce)
  if n == null:                       reject 401 AUTH_NONCE_UNKNOWN
  if n.used:                          reject 401 AUTH_NONCE_USED
  if n.expires_at <= now_utc:         reject 401 AUTH_NONCE_EXPIRED
  if n.account_id != req.account_id:  reject 401 AUTH_MESSAGE_MISMATCH

  # 1) message exact-match template kanonik §3 (constant-time)
  expected = canonical_login_message(
      domain=cfg.domain, network=cfg.network, account=n.account_id,
      nonce=n.nonce, expires=n.expires_at, scope=req.scope)
  if not constant_time_equal(expected, req.message): reject 401 AUTH_MESSAGE_MISMATCH
  if req.recipient != cfg.domain:                    reject 401 AUTH_MESSAGE_MISMATCH

  # 2) public key milik account & aktif (skema-agnostik — SEC-AUTH-004)
  keys = rpc.view_access_key_list(req.account_id)
  if req.public_key not in keys or not keys[req.public_key].active:
      reject 401 AUTH_KEY_NOT_FOUND

  # 3) verifikasi signature atas payload NEP-413
  if not nep413_verify(payload=req, pk=req.public_key, sig=req.signature):
      reject 401 AUTH_SIGNATURE

  # 4) konsumsi nonce ATOMIC — hanya jika 1..3 lolos
  changed = db.auth_nonce.update(used=true where nonce=req.nonce and used=false)
  if changed == 0:                    reject 401 AUTH_NONCE_USED   # race

  return issue_session(account_id=req.account_id, scope=req.scope)
```

- Kegagalan langkah mana pun **tidak** mengonsumsi nonce; dihitung rate-limit per IP+account (5 gagal/menit → lockout sementara).
- `canonical_login_message` membaca template §3 (bukan literal tersebar di kode).
- Step-up admin memakai varian `canonical_admin_message` (urutan §3) + cek Action/Target cocok dengan aksi yang diminta.

## 8. NEP-413 test vectors

> **Status: ⏳ diisi saat implementasi.** Tabel ini sengaja dikosongkan — nilai byte-level (serialisasi payload, prefix NEP-413 `2^31+413`, encoding nonce) harus dikunci oleh vektor nyata dari wallet, bukan dikarang. Implementasi wajib mengisi dan menjalankannya sebagai test. (Pola tabel diselaraskan dengan [api/authentication.md](../api/authentication.md) § NEP-413 test vectors.)

| # | account_id | network | recipient | nonce (32B) | message (hash) | public_key | signature | hasil diharapkan |
|---|---|---|---|---|---|---|---|---|
| TV-1 | _diisi_ | testnet | nearsea.example | _diisi_ | _diisi_ | ed25519:… | ed25519:… | 200 (valid) |
| TV-2 | _diisi_ | testnet | nearsea.example | _diisi_ (nonce diubah) | _diisi_ | ed25519:… | ed25519:… | 401 AUTH_SIGNATURE |
| TV-3 | _diisi_ | testnet | evil.example | _diisi_ | _diisi_ | ed25519:… | ed25519:… | 401 AUTH_MESSAGE_MISMATCH |
| TV-4 | _diisi_ | testnet | nearsea.example | _diisi_ (sudah dipakai) | _diisi_ | ed25519:… | ed25519:… | 401 AUTH_NONCE_USED |
| TV-5 | _diisi_ | testnet | nearsea.example | _diisi_ (expired) | _diisi_ | ed25519:… | ed25519:… | 401 AUTH_NONCE_EXPIRED |
| TV-6 | _diisi_ | mainnet | nearsea.example | _diisi_ | _diisi_ | secp256k1:… | secp256k1:… | 200 (skema lain) |
| TV-7 | _diisi_ | testnet | nearsea.example | _diisi_ | _diisi_ (Scope diubah) | ed25519:… | ed25519:… | 401 AUTH_MESSAGE_MISMATCH |
| TV-8 | _diisi_ | testnet | nearsea.example | _diisi_ | _diisi_ (step-up, Target beda) | ed25519:… | ed25519:… | 401 AUTH_MESSAGE_MISMATCH |

- Vektor wajib mencakup: NEP-413 valid, nonce mismatch, domain/recipient mismatch, nonce dipakai ulang, nonce expired, scope mismatch, target step-up mismatch, dan minimal satu skema kunci non-ed25519.

## 9. Skema session store

> Skema kanonik dimiliki [database/database-schema.md](../database/database-schema.md); tabel di bawah adalah rujukan untuk implementasi auth.

```sql
-- Nonce challenge (sekali pakai, TTL 5 menit)
auth_nonce(
  nonce        TEXT PRIMARY KEY,   -- 64 hex (32 B)
  account_id   TEXT NOT NULL,
  scope        TEXT NOT NULL,      -- report | profile | admin
  expires_at   TIMESTAMPTZ NOT NULL,
  used         BOOLEAN NOT NULL DEFAULT false,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- UNIQUE(nonce) + index (account_id, created_at); konsumsi: UPDATE … WHERE used=false

-- Sesi (access JWT + refresh opaque) — SSOT DDL = database-schema.md (ringkasan di bawah)
sessions(
  token_id           TEXT PRIMARY KEY,  -- = jti access token (denylist)
  account_id         TEXT NOT NULL,
  scope              session_scope NOT NULL,
  refresh_token_hash TEXT UNIQUE,       -- SHA-256 dari refresh opaque; NULL untuk scope admin
  family_id          TEXT NOT NULL,     -- rantai rotasi; reuse → cabut seluruh family
  rotated_from       TEXT NULL,         -- token_id sebelumnya (deteksi reuse)
  expires_at         TIMESTAMPTZ NOT NULL, -- refresh ≤12 jam; access 15 menit dicek di JWT exp
  revoked_at         TIMESTAMPTZ,       -- denylist server-side (reuse → cabut satu family)
  created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- Denylist jti: baris dengan revoked_at terisi sampai exp (access token lama ditolak)
```

- **Refresh disimpan sebagai HASH (SHA-256), bukan token mentah** — kebocoran DB tidak langsung memberi refresh yang valid.
- **Admin**: `refresh_token_hash = NULL` → tidak ada silent refresh ([admin-security.md](./admin-security.md) §2); refresh endpoint menolak scope admin.
- Retensi: access 15 menit, refresh ≤12 jam; hard delete setelah expired (database-security.md §6).
- Reuse refresh (rotated_from sudah dipakai) → `revoked_at` diisi untuk seluruh `family_id` → 401 `AUTH_REFRESH_REUSED`.

## 10. CSRF & migrasi ke cookie

> **Posisi MVP: token di header `Authorization` (Bearer) → CSRF tidak relevan.** Bagian ini mendefinisikan langkah **jika** kelak pindah ke cookie (mis. kebutuhan SSR), agar migrasi tidak membuka celah.

```text
JIKA memakai cookie (BUKAN MVP):
  1. Set cookie sesi: HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age≤900.
  2. Tambah cookie CSRF terpisah (double-submit): nilai acak 32B, TIDAK HttpOnly.
  3. Setiap request mutasi (POST/PATCH/DELETE) wajib header `X-CSRF-Token` == cookie CSRF.
  4. Verifikasi Origin/Referer terhadap allowlist domain (defense-in-depth).
  5. Endpoint GET tidak boleh punya efek samping (tidak ada state change via GET).
  6. Refresh tetap via header; tidak mengandalkan cookie untuk otorisasi admin.
  7. Step-up signature TETAP wajib untuk aksi destruktif (CSRF bukan pengganti step-up).

RISIKO JIKA SALAH MIGRASI:
  - SameSite=None tanpa CSRF token → CSRF penuh.
  - Cookie tanpa HttpOnly → XSS bisa mencuri sesi.
  - Menganggap CSRF token menggantikan step-up → aksi destruktif tanpa signature.
```

## 11. Penanganan ml-dsa-65 (post-quantum)

> ml-dsa-65 (Dilithium) didukung protokol NEAR; kunci disimpan on-chain sebagai **hash** dengan prefix `ml-dsa-65-hash:` (FACT — [wallet-authentication.md](./wallet-authentication.md) §1).

| Aspek | Perlakuan |
|---|---|
| Identifikasi | `public_key.type == ml-dsa-65` (atau key ber-hash) saat `view_access_key_list` |
| Ownership | dicek via `view_access_key_list` — **skema-agnostik** (SEC-AUTH-004); key harus terdaftar & aktif |
| Verifikasi signature | library ml-dsa yang mendukung varian NEAR; ukuran signature jauh lebih besar dari ed25519 → batas body size API harus mengakomodasi (mis. 16 KB → tinjau ulang) |
| Nonce/message | identik dengan skema lain (template §3 tidak berubah) |
| Wallet support | tergantung dukungan wallet; jika wallet belum mendukung `signMessage` PQ → user memakai wallet lain / fallback ⏳ open-by-design |
| Test | TV dengan skema PQ (⏳ diisi saat implementasi) |
| Catatan | jangan berasumsi panjang signature 64 B — kode verifikasi harus membaca tipe kunci, bukan hardcode |

## 12. Contract-account (padanan ERC-1271)

> **Status: ⏳ UNKNOWN / out-of-scope MVP (fase lanjut).** Tidak ada standar off-chain yang mapan untuk verifikasi signature **atas nama contract account** di NEAR (padanan EVM ERC-1271 belum distandardisasi).

| Aspek | Posisi |
|---|---|
| Dukungan MVP | **Tidak ada** — akun harus EOA (named/implicit/`0x`) |
| Akun contract | `view_access_key_list` bisa kosong/tidak bermakna → verify gagal `AUTH_KEY_NOT_FOUND` (perilaku aman) |
| Rencana | jika dibutuhkan (mis. smart account / MPC wallet), desain baru + ADR; jangan perpanjang S2 diam-diam |
| Alternatif sementara | contract account tetap bisa transaksi on-chain (S1); hanya auth API yang tidak didukung |
| Risiko | rendah untuk MVP (user = EOA) |

## 13. Status

- [x] Protokol dasar — DECIDED (ronde 11)
- [x] NEP-413 sebagai format utama — **DECIDED (2026-10-01; status Final, dukungan wallet luas)**
- [x] Verifikasi key ownership on-chain saat verify — **bagian dari protokol** (SEC-AUTH-004); yang masih open hanya caching/TTL-nya: OPEN QUESTION (kecil)
- [ ] Contract-account signing (padanan ERC-1271) — UNKNOWN, fase lanjut
- [ ] Dukungan input semua tipe akun (0x/implicit/named) + skema kunci (ed25519/ml-dsa-65) via view_access_key_list — masuk implementasi (docs: account-validation)
- [x] Skema payload NEP-413 eksak (§6), algoritma verifikasi (§7), skema session store (§9), CSRF/cookie migration (§10), penanganan ml-dsa-65 (§11), contract-account status (§12) — **spesifikasi implementasi (dokumen ini)**
- [ ] NEP-413 test vectors (§8) — ⏳ **diisi saat implementasi** (nilai byte-level dikunci vektor nyata, bukan dikarang)
