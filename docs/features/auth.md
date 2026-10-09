# Feature — Auth (Wallet)

> Spesifikasi fitur koneksi wallet. Satu file = satu fokus kerja agent.

## Objective

User menghubungkan wallet NEAR untuk bertransaksi; tanpa wallet tetap bisa browse.

## Preconditions

- Browser modern; near-connect terpasang di app.

## Flow

1. Klik Connect → modal near-connect (pilih wallet).
2. Approve di wallet → alamat tersimpan (localStorage).
3. Revisit → reconnect otomatis; disconnect manual via menu.

## UI

- WalletButton: `disconnected | connecting | connected(alamat+saldo) | error`.
- Connected → dropdown: profil, settings, disconnect.

## Modal State Detail

> Modal koneksi = komponen `Modal` (frontend-architecture.md §10). State berikut **wajib** punya tampilan berbeda agar user tidak bingung.

| State | Pemicu | Tampilan / aksi |
|---|---|---|
| `closed` | default | Tombol Connect di header |
| `wallet-picker` | klik Connect | Daftar wallet (near-connect); wallet tak terpasang → disabled + hint install |
| `connecting` | user pilih wallet | Spinner + nama wallet; tombol lain disabled |
| `approving` | menunggu approve di wallet | Instruksi "setujui di wallet"; tombol Cancel |
| `connected` | approve sukses | Tutup modal; header tampil alamat + saldo |
| `rejected` | user tolak popup | Toast "dibatalkan"; kembali `disconnected` (modal boleh tetap terbuka) |
| `error` | wallet error/timeout | Pesan + tombol Retry; tidak menampilkan stack trace (error-handling.md §1) |
| `network-mismatch` | wallet di network berbeda dari `NEAR_NETWORK` | Banner peringatan + transaksi di-disable (lihat § Account/Network Switching) |

## Supported Wallets

> Daftar bersumber dari **near-connect** (bukan daftar buatan sendiri): semua wallet yang didukung near-connect aktif sejak v1 ([frontend-architecture.md](../architecture/frontend-architecture.md) § Wallet integration, docs.near.org/tools/near-connect).

- Contoh wallet: **HOT**, **Meteor**, **Nightly**, **MyNearWallet**, **Ledger** (via adapter near-connect), dan wallet near-connect lain.
- **Daftar final = daftar near-connect** — tidak ada allowlist manual; wallet baru yang ditambahkan near-connect otomatis muncul. Daftar literal ⏳ mengikuti rilis near-connect saat implementasi.
- Wallet tanpa dukungan `signMessage` → jalur **custom challenge fallback** ([api/authentication.md](../api/authentication.md)); wallet tanpa dukungan `signAndSendTransaction` → tidak bisa bertransaksi (hanya browse).

## Account / Network Switching

```text
1. User buka menu wallet → Switch account (di dalam wallet) ATAU ganti NEAR_NETWORK.
2. near-connect melaporkan account/network baru → FE deteksi perubahan.
3. FE: putuskan sesi API in-memory (JWT) + invalidate query ber-scope akun (['profile',*], ['nearBalance',*], ['offersByAccount',*], ['tokensByOwner',*]).
4. FE: muat ulang state akun baru (alamat + saldo); reconnect otomatis bila wallet masih connected.
5. Bila network wallet ≠ NEAR_NETWORK → banner peringatan + disable semua tx (tidak ada silent cross-network).
```

- **Ganti akun**: state per akun (read-state notifikasi) dipilih ulang dari kunci localStorage per `account_id` ([notifications.md](./notifications.md)).
- **Ganti network**: state testnet/mainnet **tidak** bercampur (kunci localStorage menyertakan `<network>`; kontrak/env berbeda).
- Transaksi lintas network **tidak pernah** di-sign tanpa peringatan; network mismatch = hard block, bukan sekadar warning.

## Session & Reconnect Token Handling

> Transaksi chain **tidak** butuh sesi server — wallet sign langsung ke kontrak. Sesi API hanya untuk write non-chain (profil/report/admin) — [api/authentication.md](../api/authentication.md).

| Data | Tempat | TTL | Perilaku reconnect |
|---|---|---|---|
| Wallet connection (alamat publik) | localStorage | — | Revisit → reconnect otomatis; **tidak** menyimpan kunci |
| Access token API (JWT) | **in-memory** | 15 menit | Hilang saat reload → minta signature ulang (bukan persist) |
| Refresh token API | in-memory (rotasi) | ≤ 12 jam | Rotasi tiap pakai; reuse → family dicabut (SEC-AUTH-006) |
| Sesi admin | in-memory | 15 menit | **Tanpa** silent refresh; re-login + step-up |

- **Reload halaman**: wallet tetap connected (alamat di localStorage); sesi API hilang (in-memory) → user diminta sign ulang hanya saat akan menulis (profil/report).
- **Disconnect manual**: hapus state koneksi + hapus sesi API in-memory; **tidak** memengaruhi approval on-chain yang sudah ada (listing tetap berlaku).
- **Koneksi putus saat signing**: cek status tx via hash **dulu**; reset state hanya bila tx tidak ditemukan/gagal (error-handling.md §5).
- Sesi API **tidak** di localStorage/sessionStorage — mencegah XSS mencuri token (frontend-architecture.md §11).

## Deep-link / Mobile

- Connect & signing di mobile browser melalui near-connect; wallet mobile memakai deep-link ke app wallet untuk approve/sign, kembali ke dApp via callback/redirect.
- **`callbackUrl`** (NEP-413 opsional) dipakai untuk kembali ke halaman asal setelah sign; nilai final mengikuti konfigurasi domain (placeholder kanonik `nearsea.example`, final saat deploy — [wallet-authentication.md](../security/wallet-authentication.md) §3).
- Deep-link tanpa wallet terpasang → fallback ke web wallet / halaman install.
- Perilaku spesifik per wallet mobile (skema deep-link, return URL) = ⏳ open-by-design — dikunci saat implementasi mengikuti near-connect.

## Access-Key Handling

- App **tidak pernah** melihat/menyimpan private key atau seed phrase — hanya alamat publik (SEC-KEY-001).
- Transaksi memakai key yang ada di wallet; **function-call key** (dibatasi kontrak) dan **full-access key** sama-sama bisa dipakai wallet untuk `signAndSendTransaction`.
- Verifikasi API (`signMessage`) memakai RPC `view_access_key_list` untuk memastikan public key terikat & aktif pada account (skema-agnostik: ed25519 / secp256k1 / ml-dsa-65-hash) — SEC-AUTH-004.
- Perilaku `signMessage` untuk function-call key vs full-access key = ⏳ open-by-design (dikunci saat implementasi + test — [api/authentication.md](../api/authentication.md) § Access-key removal/rotation).
- User dapat merevoke approval on-chain kapan pun (`nft_revoke` / `nft_revoke_all`); listing menjadi stale dan tidak bisa dibeli (tidak memerlukan aksi app).

## Profile-Signature Flow (Pointer)

- Alur signature untuk update profil (alias/bio/avatar) memakai **protokol kanonik NEP-413 + fallback** — **dimiliki** [security/wallet-authentication.md](../security/wallet-authentication.md) §3 (template pesan login: `Domain → Network → Account → Nonce → Expires → Scope`).
- Dokumen ini **tidak** mendefinisikan ulang template. Ringkas alur: `GET /api/auth/nonce` → wallet `signMessage` → `POST /api/auth/verify` → sesi scope `profile` → `PATCH /api/accounts/me/profile` ([api/authentication.md](../api/authentication.md), [users.md](./users.md)).

## API / Chain

- Tidak ada API auth untuk transaksi — semua signing di wallet (API non-chain: [api/authentication.md](../api/authentication.md)).

## Error Cases

| Kasus | Perlakuan | Kode error |
|---|---|---|
| User menolak popup | toast "dibatalkan", state kembali disconnected | (lokal, tanpa kode API) |
| Wallet tidak terpasang | tombol wallet tsb disabled + hint install | (lokal) |
| Network mismatch (testnet vs mainnet) | banner peringatan + disable transaksi | (lokal) |
| Koneksi putus saat signing | cek status tx via tx hash DULU; reset state hanya bila tx tidak ditemukan/gagal | `CHAIN_TIMEOUT` |
| Nonce tidak dikenal | minta nonce baru | `AUTH_NONCE_UNKNOWN` |
| Nonce kedaluwarsa (>5 menit) | minta nonce baru | `AUTH_NONCE_EXPIRED` |
| Nonce sudah dipakai (replay) | tolak; minta nonce baru | `AUTH_NONCE_USED` |
| Message/recipient/domain/scope mismatch | tolak; jangan retry otomatis | `AUTH_MESSAGE_MISMATCH` |
| Signature tidak valid | tolak; user boleh coba sign ulang | `AUTH_SIGNATURE` |
| Public key bukan milik/aktif | tolak; sarankan cek wallet | `AUTH_KEY_NOT_FOUND` |
| Sesi API kedaluwarsa | minta signature ulang (bukan silent untuk admin) | `AUTH_SESSION_EXPIRED` |
| Token API malformed/alg ditolak | tolak; hapus sesi in-memory | `AUTH_TOKEN_INVALID` |
| Refresh kedaluwarsa | re-login | `AUTH_REFRESH_INVALID` |
| Refresh dipakai ulang | family dicabut → re-login | `AUTH_REFRESH_REUSED` |
| Scope kurang | tolak aksi | `FORBIDDEN_SCOPE` |
| Gagal berulang (5×/menit) | lockout sementara + `Retry-After` | `RATE_LIMITED` |

> Pemetaan kode → pesan user & envelope **dimiliki** [error-handling.md](../development/error-handling.md) §3/§4. FE **wajib** menampilkan pesan i18n berdasarkan kode, bukan teks mentah kontrak/wallet. Registry kode lengkap auth ada di [api/authentication.md](../api/authentication.md) § Daftar kode error.

## Security

- App tidak pernah melihat/menyimpan private key — hanya alamat publik.

## Status implementasi (TASK-007, 2026-10-08)

> Kode: `frontend/features/auth/` + `frontend/lib/near/` + `frontend/lib/format/money.ts` +
> `frontend/i18n/en/auth.json`. Detail arsitektur: [frontend-architecture.md](../architecture/frontend-architecture.md)
> §Wallet integration.

| Bagian flow | Status | Catatan |
|---|---|---|
| Klik Connect → pilih wallet | ✅ | Popup **bawaan near-connect** (bukan komponen `Modal` sendiri). Daftar wallet dari manifest resmi — tanpa allowlist manual |
| Approve di wallet → alamat tersimpan | ✅ | `signIn()`; near-connect menyimpan wallet terpilih di localStorage |
| Revisit → reconnect otomatis | ⚠️ belum diverifikasi | Mekanisme ada di near-connect (`getConnectedWallet()`); belum diuji dengan wallet nyata |
| Disconnect manual | ✅ | `disconnect()` → `signOut()`; state kembali `disconnected` |
| Tampilan alamat + saldo di header | ✅ | Saldo dari probe RPC `view_account`; yoctoNEAR → tampilan lewat `lib/format/money.ts` (BigInt) |
| Network mismatch → banner + disable tx | ⚠️ sebagian | Banner + **flag gerbang** `transactionsDisabled` ada dan teruji; yang **belum ada** adalah tombol aksi yang memakainya (baru muncul di TASK-008) |
| Sesi API (JWT) in-memory | — | Fitur API; belum ada (Fase 2) |
| Deep-link mobile | ⏳ | Tetap open-by-design; mengikuti near-connect saat diuji di perangkat |

- **Deteksi jaringan (batasan yang disengaja)**: near-connect **tidak mengekspos network wallet**, jadi
  mismatch tidak bisa dibaca langsung. Yang dibuktikan on-chain adalah **akun tidak ada di jaringan yang
  dikonfigurasi** (probe `view_account` gagal "account does not exist"). Akibatnya: akun testnet yang
  belum dibuat/di-fund juga memicu peringatan ini (positif palsu yang disengaja), dan akun yang ada di
  **kedua** jaringan tidak terdeteksi (negatif palsu). Copy UI karena itu menyebut **kedua** kemungkinan
  ("wallet di jaringan lain **atau** akun belum dibuat/di-fund"), bukan mengklaim jaringan yang salah.
  RPC yang tidak terjangkau (`unreachable`) **tidak** memicu peringatan — RPC bermasalah bukan bukti
  akun ada di jaringan lain — tetapi tetap membuat `transactionsDisabled = true`.
- **State yang dipakai**: `loading` (pra-hidrasi) / `disconnected` / `connecting` (+ hint "approve di
  wallet") / `connected(alamat+saldo)` / `disconnecting` / `error` (+ Retry yang mengulang **aksi yang
  gagal**, pesan i18n, tanpa teks mentah wallet). `wallet-picker` dan `approving` versi kustom tidak
  dibuat — popup near-connect yang menanganinya; `rejected` tidak dipisah menjadi state sendiri karena
  gejalanya sama dari sisi UI (kembali ke keadaan belum-connect + pesan yang terbaca).
- **Mode baca**: halaman tetap ter-render tanpa wallet (bukan halaman kosong); lapisan near-connect
  hanya di-mount di browser.
- **Belum diklaim**: connect dengan wallet testnet nyata + persistensi setelah reload (butuh akun
  testnet & browser; jalur emas Playwright = TASK-008).

## Acceptance Criteria

- Given belum connect, When klik Connect lalu approve, Then **alamat + saldo** tampil di header dan bertahan setelah reload.
  - ⚠️ Separuh terverifikasi (TASK-007): alamat + saldo tampil. "Bertahan setelah reload" belum diuji
    dengan wallet nyata — lihat §Status implementasi.
- **AC-WALLET-2/3** (tolak popup → pesan "dibatalkan", kembali `disconnected`; wallet tak terpasang →
  hint install, bukan crash): pesannya ada (`auth.error.rejected`, `auth.error.wallet_unavailable`)
  dan teruji di unit dengan error yang disimulasikan; perilaku wallet nyata = TASK-008.
- **AC-WALLET-4** (disconnect → alamat dihapus dari state, kembali `disconnected`): ✅ teruji.
