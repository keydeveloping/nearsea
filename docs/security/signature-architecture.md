# Signature Architecture

> Mengklasifikasi SEMUA jenis signature dalam sistem dan apa otorisasi yang diwakili masing-masing. Prinsip: **satu signature = satu tujuan; tidak boleh dicampur.**

## 1. Taksonomi signature

| # | Jenis | Siapa sign | Diverifikasi oleh | Format | Mewakili otorisasi | Masa hidup |
|---|---|---|---|---|---|---|
| S1 | **Transaction signature (on-chain)** | User (access key) | Protocol NEAR (validator) | SignedTransaction ed25519 | SEMUA efek tx: mutasi listing, escrow offer, mint, payout | Sekali eksekusi; anti-replay = nonce key + recent block hash (FACT protokol) |
| S2 | **Auth challenge signature (API)** | User (access key, via sign-message wallet) | Report API | **NEP-413** (utama — status Final) / custom string fallback (ADR-011) | Identitas + scope off-chain (report/profile/admin) | 1 nonce, TTL 5 menit |
| S3 | **Admin step-up signature** | Admin (allowlist) | Report API + allowlist check | Sama dengan S2, scope `admin`, action name eksplisit | Aksi moderasi spesifik (hide, verify, blocklist) | 1 nonce per aksi |
| S4 | **Deploy/upgrade tx** | Platform Owner key | Protocol + (mainnet) Sputnik DAO V2 2-of-3 + timelock — **DECIDED (ADR-013), implementasi M4** | SignedTransaction | Kode kontrak baru / param owner (fee, pause) | Per tx |

**Template kanonik per baris** (SSOT = [wallet-authentication.md](./wallet-authentication.md) §3 — dokumen ini hanya merujuk, tidak mendefinisikan ulang):

| # | Template kanonik | Urutan field | Catatan |
|---|---|---|---|
| S1 | — (payload tx protokol) | receiver_id, method, args, deposit, gas | Bukan pesan teks; wallet menampilkan ringkasan |
| S2 | `NearSea login` | `Domain → Network → Account → Nonce → Expires → Scope` | `Nonce` WAJIB di dalam message (bukan hanya payload NEP-413) |
| S3 | `NearSea admin action` | `Domain → Network → Account → Action → Target → Nonce → Expires` | Signature kedua; tidak memakai ulang token login |
| S4 | — (payload tx protokol) | receiver_id, method, args, deposit, gas | mainnet lewat proposal DAO |

## 2. Apa yang TIDAK kita pakai (dan kenapa)

- **Off-chain order signature (model Seaport/EIP-712)** — TIDAK dipakai. **FACT**: model kita = orderbook on-chain (ADR-003/ADR-012); order adalah state kontrak, bukan pesan tertanda tangan off-chain. Konsekuensi: setiap list/offer = tx on-chain (gas murah di NEAR — FACT), tapi tidak ada order gasless. Trade-off didokumentasikan di ADR-012.
- **Backend signer service untuk order** — tidak ada; backend tidak pernah menandatangani apa pun atas nama user (non-custodial).

## 3. Domain / context separation

| Signature | Binding konteks |
|---|---|
| S1 (tx) | receiver_id (kontrak spesifik) + method + deposit — FACT protokol; network di bagian chain signature (testnet/mainnet terpisah) |
| S2/S3 (API) | string message memuat `domain` (domain produksi/staging berbeda) + `network` + `scope` + `nonce` + `expires` |
| S4 | receiver = kontrak target; mainnet harus lewat prosedur terpisah (deploy.md checklist) |

**Aturan**: API menolak signature yang message-nya tidak persis cocok template kanonik (memcmp penuh, bukan parse longgar).

## 4. Nonce & replay matrix

| Objek | Nonce | Disimpan di | Membatalkan |
|---|---|---|---|
| Tx on-chain | per access key (protokol) | chain state | tidak perlu (sekali eksekusi) |
| Auth challenge | random 32B | DB `auth_nonce`, consumed atomic | expire 5m / pakai sekali |
| Admin step-up | random 32B per action | sama, scope `admin` | sama |
| NEP-178 approval | `approval_id` increment | NFT contract state | revoke/cancel/transfer |

## 5. Signature yang hilang / belum didesain (jangan mengarang)

- ~~UNKNOWN: format sign-message~~ **RESOLVED (2026-10-01)**: S2 memakai **NEP-413** (Final, dukungan wallet luas); custom challenge hanya fallback — wallet-authentication.md.
- **DECIDED (ADR-013)**: timelock untuk S4 di mainnet (delay param change fee) via Sputnik DAO proposal; implementasi gate M4; tidak ada di MVP (testnet).
- Tidak ada signature backend untuk settlement — FACT by design; jika suatu saat dibutuhkan (mis. gasless relay, fase lanjut), harus jadi ADR baru, bukan perpanjangan S1–S4.

## 6. Detail skema kunci & algoritma

> Sumber skema = protokol NEAR (FACT). API **skema-agnostik**: ia tidak memverifikasi kripto sendiri, tetapi menyerahkan verifikasi ke RPC/layanan sesuai tipe kunci.

| Skema | Ukuran kunci | Ukuran signature | Encoding NEAR | Dipakai untuk | Catatan verifikasi |
|---|---|---|---|---|---|
| ed25519 | 32 B | 64 B | `ed25519:<base58>` | jalur utama (named/implicit/testnet) | default; didukung semua wallet |
| secp256k1 | 33 B (compressed) | 64 B (+recovery) | `secp256k1:<base58>` | wallet alternatif; akun `0x…` | verifikasi via library secp256k1 |
| ml-dsa-65 | post-quantum (hash on-chain) | besar | disimpan on-chain sebagai `ml-dsa-65-hash:` | akun dengan kunci PQ | ownership via `view_access_key_list`; detail penanganan = [wallet-authentication.md](./wallet-authentication.md) §11 |
| (future) | — | — | — | ⏳ open-by-design | skema baru = review + test vector |

- **S1/S4** diverifikasi validator NEAR — kita tidak menangani byte signature.
- **S2/S3** diverifikasi API; karena key ownership dicek via RPC (SEC-AUTH-004), API tidak perlu tahu cara kripto per skema kecuali untuk memverifikasi signature itu sendiri.

## 7. Contoh byte-level (worked) — S2 NEP-413

> Nilai di bawah adalah **contoh kanonik** dengan placeholder; byte persis dikunci oleh **test vector** ([wallet-authentication.md](./wallet-authentication.md) §8, diisi saat implementasi). Struktur serialisasi mengikuti NEP-413.

```text
PAYLOAD NEP-413 (yang di-sign wallet):
  Borsh(struct SignMessage { message: String, nonce: [u8;32], recipient: String, callback_url: Option<String> })

PREFIX (NEP-413, FACT protokol):
  u32 little-endian dari (2^31 + 413) = 0x8000019D
  → byte: 9D 01 00 80

LANGKAH SERIALISASI:
  1. message    = "NearSea login\nDomain: nearsea.example\nNetwork: testnet\nAccount: alice.testnet\nNonce: <64hex>\nExpires: 2026-10-02T12:05:00Z\nScope: profile"
                   (UTF-8; panjang di-prefix u32 LE)
  2. nonce      = 32 byte mentah (decode dari 64 hex) — BUKAN string
  3. recipient  = "nearsea.example" (UTF-8; panjang di-prefix u32 LE)
  4. callback   = None → byte 0x00 (Option Borsh)
  bytes_to_sign = prefix(4B) || borsh(SignMessage)
  signature     = ed25519_sign(private_key, sha256? TIDAK — NEP-413 sign atas bytes_to_sign; lihat test vector)

OUTPUT WALLET:
  { publicKey: "ed25519:<b58>", signature: "ed25519:<b58 64B>", state: "signed" }
```

> **Penting**: API tidak boleh mengarang serialisasi. Ia membandingkan `message` + `recipient` dengan template kanonik (memcmp penuh) dan memverifikasi signature memakai implementasi NEP-413 resmi. Byte-level detail di atas **wajib dikonfirmasi test vector** sebelum diklaim benar.

## 8. Langkah validasi per jenis signature

> Urutan **wajib** (validasi dulu, konsumsi nonce belakangan) — SEC-AUTH-001.

```text
S1 — Transaction signature (on-chain):
  1. Wallet membangun SignedTransaction (nonce key + recent block hash).
  2. Validator verifikasi signature atas hash tx.
  3. Kontrak menegakkan otorisasi: predecessor + ownership/approval + assert_one_yocto.
  (Tidak ada langkah kita — non-custodial.)

S2 — Auth challenge (login):
  1. Nonce ada & belum dipakai & belum expired & milik account_id.
  2. message exact-match template kanonik login (Domain→Network→Account→Nonce→Expires→Scope).
  3. recipient == domain terkonfigurasi.
  4. public_key terdaftar & aktif milik account_id (view_access_key_list).
  5. signature valid atas payload NEP-413.
  6. KONSUMSI nonce atomically (UPDATE … WHERE used=false) — hanya jika 1..5 lolos.
  7. Terbitkan session token (scope dari message).

S3 — Admin step-up:
  1. Sesi admin valid (scope admin + allowlist DB).
  2. Nonce step-up ada & belum dipakai & belum expired.
  3. message exact-match template kanonik admin (Domain→Network→Account→Action→Target→Nonce→Expires).
  4. Action & Target cocok dengan aksi yang diminta (bukan sekadar format).
  5. public_key masih aktif (re-cek — kunci yang dicabut tidak bisa menyelesaikan aksi).
  6. signature valid; KONSUMSI nonce.
  7. Eksekusi + tulis `admin_audit`.

S4 — Deploy/upgrade:
  1. Tx valid (protokol) — receiver = kontrak target.
  2. MVP: owner key. Mainnet: proposal Sputnik DAO 2-of-3 + timelock.
  3. Publish wasm hash; verifikasi reproducible build NEP-330 (SEC-CONTRACT-006).
```

## 9. Matriks revokasi

| Objek signature | Cara revokasi | Efek | Latensi | SEC-ID |
|---|---|---|---|---|
| S1 (tx) | — (sekali eksekusi) | tidak perlu | — | — |
| S2 access token | denylist `jti` + hapus sesi | token lama ditolak | instan (server-side) | SEC-AUTH-003/006 |
| S2 refresh token | rotasi tiap pakai; reuse → cabut family | rantai sesi invalid | instan | SEC-AUTH-006 |
| S2 (semua sesi akun) | hapus `sessions` akun + denylist | semua sesi akun mati | instan | SEC-AUTH-006 |
| S3 step-up | nonce sekali pakai | signature tidak bisa dipakai ulang | instan | SEC-ADMIN-003 |
| S3 (allowlist berubah) | hapus dari allowlist + revoke sesi | admin kehilangan akses | instan | SEC-ADMIN-001 |
| S4 (owner key) | rotasi owner (MVP) / proposal ganti council (mainnet) | key lama tak berwenang | MVP: redeploy; mainnet: proposal+timelock | SEC-KEY-001/002 |
| NEP-178 approval | `nft_revoke` / `nft_revoke_all` (owner-only) / transfer token | approval invalid | on-chain | INV-011 |

## 10. Pemetaan test signature

| Signature | Test | Jenis |
|---|---|---|
| S2 happy path | TC-028 (verify valid) | api |
| S2 replay nonce | TC-028 (nonce dipakai ulang → 401) | api |
| S2 domain mismatch | TC-028 (TV-3, recipient beda → 401) | api |
| S2 key bukan milik akun | TC-028 (→ 401 AUTH_KEY_NOT_FOUND) | api |
| S2 skema non-ed25519 | TC-028 (TV-6, secp256k1) | api |
| S3 step-up happy | TC-025 | api |
| S3 step-up ditolak (tanpa/expired/beda target) | TC-026 | api |
| S3 nonce step-up sekali pakai | TC-026 | api |
| S4 hash verify | SEC-CONTRACT-006 (post-deploy) | CI gate |
| NEP-413 test vectors | [wallet-authentication.md](./wallet-authentication.md) §8 (diisi saat implementasi) | vector |
