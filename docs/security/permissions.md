# Permissions & Roles

> Ringkasan peran. **Register formal + matriks operasi privileged (auth/approval/audit): [access-control-matrix.md](./access-control-matrix.md).**

## Roles

> Alias ↔ register (access-control-matrix.md): `User` = USER · `Collection Owner` = CREATOR · `Admin off-chain` = ADMIN · `Platform Owner` = PLATFORM_OWNER.

| Role | Definisi |
|---|---|
| Guest | belum connect wallet — browse only |
| User (USER) | wallet terhubung |
| Seller / Buyer | role transaksional (sama-sama User) |
| Collection Owner (CREATOR) | USER + owner kontrak koleksi — mint sesuai launchpad phases, atur phase & allowlist miliknya |
| Platform Owner (PLATFORM_OWNER) | pemegang owner key market/factory: pause, fee, treasury, upgrade |
| Admin off-chain (ADMIN) | allowlist DB + wallet signature; panel /admin: report queue, verified badge, blocklist — **hanya layer tampilan, tanpa kekuatan on-chain** |

## Permissions matrix

> **Konvensi**: `✓` = bisa via role itu sendiri; `—` = TIDAK bisa via role itu (role yang lebih tinggi tetap bisa bila ia juga berperan sebagai User). Kolom Admin off-chain sengaja `—` pada baris on-chain karena admin off-chain TIDAK memiliki kekuatan on-chain — moderasi hanya menyentuh layer tampilan.

| Kemampuan | Guest | User | Collection Owner | Admin | Platform Owner |
|---|---|---|---|---|---|
| Browse & search | ✓ | ✓ | ✓ | ✓ | ✓ |
| Connect wallet | — | ✓ | ✓ | ✓ | ✓ |
| Mint (on-chain) | — | ✓ per phase launchpad | ✓ | — | — |
| List / cancel / update price | — | ✓ (token milik sendiri) | ✓ | — | — |
| Buy / Offer / Bundle | — | ✓ | ✓ | — | — |
| Create collection + phases (on-chain) | — | ✓ | ✓ | — | — |
| Review report / set verified / blocklist (display-layer) | — | — | — | ✓ | — |
| Pause / set fee / treasury / upgrade (on-chain) | — | — | — | — | ✓ |

> Otorisasi enforcement: kontrak (utama) + API session/allowlist (sekunder) + UI (kosmetik saja). Operasi privileged lainnya (withdraw treasury, dll.) ada di [access-control-matrix.md](./access-control-matrix.md).

## Kemampuan lengkap per fase (MVP / fase 2 / mainnet)

> Daftar kemampuan menyeluruh (bukan hanya yang di matriks ringkas atas). Kolom fase = kapan kemampuan **tersedia**; `—` = belum ada. Enforcement = layer yang benar-benar menegakkan (UI tidak pernah otoritatif).

| # | Kemampuan | Aktor | MVP | Fase 2 | Mainnet | Enforcement | Audit |
|---|---|---|---|---|---|---|---|
| A-01 | Browse & search | Guest/User | ✓ | ✓ | ✓ | API (read, rate-limited) | — |
| A-02 | Connect wallet | User | ✓ | ✓ | ✓ | Wallet (klien) | — |
| A-03 | Login API (NEP-413 + fallback) | User | ✓ | ✓ | ✓ | API (nonce + signature) | log percobaan |
| A-04 | Edit profil (alias/bio/avatar) | User (self) | ✓ | ✓ | ✓ | API (session `profile`) | log |
| A-05 | Kirim report | User | ✓ | ✓ | ✓ | API (session `report`) | log |
| A-06 | Mint via launchpad | User per phase | ✓ | ✓ | ✓ | Kontrak (phase + allowlist + storage) | event launchpad |
| A-07 | Buat koleksi + phases | CREATOR | ✓ | ✓ | ✓ | Kontrak (factory + owner koleksi) | event launchpad |
| A-08 | List / update harga / cancel listing | User (pemilik token) | ✓ | ✓ | ✓ | Kontrak (`predecessor` + approval) | event `market_*` |
| A-09 | Buy (listing) | User | ✓ | ✓ | ✓ | Kontrak (deposit = harga) | event `market_sale` |
| A-10 | Make / cancel offer (escrow) | User | ✓ | ✓ | ✓ | Kontrak (escrow exact-match) | event offer |
| A-11 | Accept offer | User (owner token) | ✓ | ✓ | ✓ | Kontrak (owner + expiry) | event `market_sale` |
| A-12 | Bundle create / buy | User | ✓ | ✓ | ✓ | Kontrak (pre-validasi + agregasi) | event bundle |
| A-13 | Private listing (allowed_buyer) | User (seller) | ✓ | ✓ | ✓ | Kontrak (cek `allowed_buyer`) | event |
| A-14 | Withdraw storage | User | ✓ | ✓ | ✓ | Kontrak (NEP-145) | event |
| A-15 | Pause / unpause | PLATFORM_OWNER / guardian | ✓ (owner) | ✓ | ✓ (guardian pause) | Kontrak (Owner/Pausable) | event `market_pause` |
| A-16 | Update `fee_bps` | PLATFORM_OWNER | ✓ | ✓ | ✓ (DAO + timelock) | Kontrak (≤ `MAX_FEE_BPS`) | event `fee_update` |
| A-17 | Update treasury | PLATFORM_OWNER | ✓ | ✓ | ✓ (DAO + timelock) | Kontrak (Owner) | event `treasury_update` |
| A-18 | Withdraw fee treasury | PLATFORM_OWNER / FINANCE | ✓ | ✓ | ✓ (DAO) | Kontrak (hanya dana fee — INV-005) | event `treasury_withdraw` |
| A-19 | Upgrade kontrak | PLATFORM_OWNER | ✓ | ✓ | ✓ (DAO 2-of-3 + timelock) | Kontrak (owner/governance) + NEP-330 | code hash on-chain |
| A-20 | Moderasi report (hide/ignore) | ADMIN | ✓ | ✓ | ✓ | API (allowlist + step-up) | `admin_audit` |
| A-21 | Set verified badge | ADMIN | ✓ | ✓ | ✓ | API (allowlist + step-up) | `admin_audit` |
| A-22 | Edit blocklist tampilan | ADMIN | ✓ | ✓ | ✓ | API (allowlist + step-up) | `admin_audit` |
| A-23 | Kelola allowlist admin | SUPER_ADMIN | manual SQL | ✓ | ✓ (2-admin approval) | API/DB | `admin_audit` |
| A-24 | Baca antrean report (read-only) | SUPPORT | — | ✓ | ✓ | API (scope `support`) | query log |
| A-25 | Rekonsiliasi treasury/fee | FINANCE | — | — | ✓ | DAO proposal | event + laporan |
| A-26 | Break-glass (SQL via SSH) | SUPER_ADMIN | ✓ | ✓ | ✓ | DB/SSH (audit manual) | `admin_audit` |
| A-27 | Discovery/indexer (proyeksi) | Indexer | — | ✓ | ✓ | DB read-only (non-otoritatif) | lag/reconciliation |

## Enforcement layer (ringkas)

| Layer | Sifat | Contoh | Catatan |
|---|---|---|---|
| Kontrak | **Otoritatif** (dana & kepemilikan) | ownership, approval, payout, escrow, pause, fee | Tidak bisa dilangkahi API/FE |
| API | Sekunder (identitas off-chain) | session/scope, allowlist admin, rate limit | Gagal API ≠ ubah state on-chain |
| DB | Pendukung (proyeksi/audit) | allowlist, `admin_audit`, nonce, sessions | Bukan otoritas kepemilikan |
| UI | Kosmetik | sembunyikan tombol, tampilkan badge | Dilarang jadi kontrol keamanan |

## Audit & approval

| Operasi | Approval (MVP) | Approval (mainnet) | Audit wajib |
|---|---|---|---|
| Pause | tunggal (cepat) | guardian tunggal (cepat) | event `market_pause` |
| Unpause | tunggal | DAO + timelock | event `market_unpause` |
| Fee/treasury/upgrade | tunggal (owner key) | DAO 2-of-3 + timelock 24 jam | event on-chain + code hash |
| Withdraw treasury | tunggal | DAO (FINANCE + SECURITY) | event `treasury_withdraw` |
| Moderasi admin | admin tunggal | 2-admin untuk hide berdampak luas | `admin_audit` |
| Kelola allowlist admin | manual SQL | 2-admin approval | `admin_audit` |

## Separation of duties (SoD)

> Prinsip: **yang bisa pause ≠ yang bisa tarik dana**; tidak ada satu aktor dengan kekuatan penuh di mainnet.

| Peran | Pause | Tarik dana | Upgrade | Moderasi | Kelola admin |
|---|---|---|---|---|---|
| SECURITY / guardian | ✓ | — | — | — | — |
| FINANCE | — | ✓ | — | — | — |
| PLATFORM_OWNER (DAO) | — (delegasi ke guardian) | ✓ | ✓ | — | — |
| ADMIN | — | — | — | ✓ | — |
| SUPER_ADMIN | — | — | — | — | ✓ |

MVP = satu orang memegang beberapa peran (risiko diterima testnet, [key-management.md](./key-management.md) §2); mainnet WAJIB memisahkan minimal **pause ≠ treasury ≠ upgrade**.

## Pemetaan SEC-ID

| Kemampuan | SEC-ID terkait |
|---|---|
| A-03, A-04, A-05 (auth/session) | SEC-AUTH-001..006 |
| A-08..A-14 (order/kontrak) | SEC-ORDER-001..006, SEC-CONTRACT-001..005 |
| A-15..A-19 (owner ops) | SEC-CONTRACT-004/007/012, SEC-KEY-001/002 |
| A-20..A-23, A-26 (admin) | SEC-ADMIN-001/002/003 |
| A-24 (support) | SEC-API-002 |
| A-27 (indexer) | SEC-INDEX-001/002 |
