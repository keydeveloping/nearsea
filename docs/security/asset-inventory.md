# Security Asset Inventory

> Inventaris formal semua aset yang harus dilindungi. Fase berlaku: MVP (testnet) dan Mainnet.
> Label: **FACT** = properti protokol NEAR; **DESIGN** = keputusan kita (ADR); **ASSUMPTION** = perlu validasi.

## 1. Financial Assets

| Aset | Lokasi | Fase | Kontrol utama |
|---|---|---|---|
| NFT user (NEP-171) | Selalu di wallet owner (approval model, ADR-007) | MVP+ | NEP-178 approval per token; `assert_one_yocto`; market tidak pernah custody NFT |
| Ⓝ escrow offer (dana buyer) | Di dalam market contract, per `OfferKey` | MVP+ | State on-chain; refund via `cancel_offer`/expire; validasi payout di resolve callback |
| Ⓝ pembayaran (buy now) | Melewati market contract dalam 1 receipt settlement | MVP+ | Promise gagal → auto-refund ke market; **refund ke buyer via resolve callback** (SEC-ORDER-001/002) |
| Proceeds seller | Transfer langsung dari market → seller (bukan saldo tertahan) | MVP+ | Payout validation (≤ harga, ≤10 penerima) |
| Royalti kreator | Bagian dari payout on-chain | MVP+ | `nft_transfer_payout`, batas 10 penerima, `amount > 0`, Σ ≤ harga−fee |
| Fee platform 2% | Ditransfer **langsung ke akun treasury** saat settlement (tidak tertahan di kontrak) | MVP+ | `fee_bps` owner-only + **cap immutable MAX_FEE_BPS** (SEC-CONTRACT-004); market tidak pernah custody fee (ronde 23 — [contracts/market.md](../contracts/market.md) §4a) |
| FT pembayaran (NEP-141) | Belum ada | Fase 2 | `ft_on_transfer` predecessor whitelist + refund unused |
| Dana treasury (fee) | **Tidak ada** — fee masuk akun treasury saat settlement | — | Tidak ada akumulasi di kontrak ⇒ tidak ada `withdraw_fees`/honeypot saldo fee (ronde 23) |
| Insurance fund (0.1% dari fee) | Market contract / akun terpisah | **Mainnet** | G7 default: dipakai hanya untuk kompensasi insiden terverifikasi, pencairan via proposal Sputnik DAO |

**FACT**: market contract TIDAK pernah memegang NFT user (approval model). Di MVP, satu-satunya dana pihak ketiga yang dipegang kontrak = escrow offer aktif (fase M1+) + deposit yang sedang settle (`pending_purchases`, transien — INV-031); fee tidak pernah tertahan di kontrak (ronde 23). Di mainnet ditambah **insurance fund** (G7).

## 2. Authorization Assets

| Aset | Pemegang | Masa hidup | Catatan |
|---|---|---|---|
| Full-access key (owner kontrak) | Platform Owner | Sampai dirotasi | Kekuatan penuh atas kontrak — lihat key-management.md |
| Function-call key | User (opsional) | User-controlled | **FACT**: tidak bisa attach deposit apa pun → otomatis gagal di method yang butuh deposit |
| NEP-178 approval (per token, `approval_id`) | Market contract (atas nama seller) | Sampai transfer/cancel/revoke | Satu approval_id per (token, market); invalid setelah transfer |
| Nonce transaksi (per access key) | Protokol NEAR | Per tx | **FACT**: replay protection level protokol + recent block hash |
| Nonce challenge API (login/report) | Report API (DB) | TTL 5 menit, sekali pakai | [wallet-authentication.md](./wallet-authentication.md) |
| Session token (API) | Report API | TTL pendek | Signature wallet → token; tidak menyentuh dana |
| Admin allowlist | DB (dikelola manual/script) | Manual | Admin-security.md |

## 3. Cryptographic Assets

| Aset | Fase | Storage | Status |
|---|---|---|---|
| Full-access key deploy/owner (ed25519) | MVP | VPS, permission 600, tidak di git | DECIDED interim — key-management.md §MVP |
| Seed phrase akun owner | MVP | Dicetak offline (kertas/steel), **2 lokasi terpisah**, tidak pernah di VPS/cloud | DECIDED (key-management.md §2) |
| Multisig mainnet (owner kontrak) | Mainnet | Sputnik DAO V2 council 2-of-3 (via app NEAR Treasury; Astro UI deprecated) | DECIDED (ADR-013) — implementasi PROPOSED |

## 4. Platform Assets

- Koleksi & metadata token (on-chain + IPFS) — integritas via `media_hash`/`reference_hash`.
- Listing/offer state on-chain — sumber kebenaran; cache API hanya proyeksi.
- Profil custom (alias/bio/avatar) — DB kecil, mutable, non-fundamental.
- Reports & keputusan moderasi — DB kecil; audit trail `decided_by/decided_at`.
- Reputasi platform (nama domain, akun `nearsea.testnet`/`.near`, badge verified) — target impersonation.

## 5. Infrastructure Assets

| Aset | Fase | Catatan |
|---|---|---|
| VPS (FE + API report + PostgreSQL) | MVP+ | Single box — single point of failure; lihat infrastructure-security.md |
| DNS domain | MVP+ | DNS hijack = phishing full-control → cicd/infra-security |
| GitHub repo + Actions secrets | MVP+ | SSH deploy key, PAT — supply chain surface |
| RPC provider endpoints | MVP+ | Malicious/incorrect RPC = data palsu di FE (bukan dana) |
| Telegram bot token | MVP+ | Alert channel — bisa dibajak untuk spam, bukan dana |
| Backup PostgreSQL | MVP+ | Berisi profile/report — tidak ada secret |

## 6. Aset yang TIDAK kita pegang (penting untuk scope)

- **FACT**: Private key wallet user — tidak pernah menyentuh sistem kita (wallet pihak ketiga).
- **FACT**: NFT lazy-minted belum ada di MVP (fase 3) — saat ada, kontrak launchpad akan memegang "virtual" supply; modelnya baru didesain saat itu.
- Password/email user — tidak ada (wallet-based auth).

## 7. Owner / custodian per aset

> **Owner** = pemilik risiko (accountable); **Custodian** = yang menyimpan/mengendalikan secara teknis. Tidak ada aset tanpa owner.

| Aset | Owner | Custodian | Bisa diakses siapa | Fase |
|---|---|---|---|---|
| NFT user (NEP-171) | User | User (wallet) — kontrak tidak pernah custody | hanya owner (approval ke market sementara) | MVP+ |
| Escrow Ⓝ offer | Buyer (pemilik dana) | Market contract (escrow sementara) | kontrak; refund ke `offer.buyer_id` | MVP+ |
| Dana settlement | Buyer → seller/royalti/treasury | Market contract (1 receipt) | kontrak (payout tervalidasi) | MVP+ |
| Fee treasury (2%) | Platform | Market contract | PLATFORM_OWNER/FINANCE (withdraw) | MVP+ |
| Insurance fund (0.1% fee) | Platform/komunitas | Market contract / akun terpisah | DAO proposal (pencairan) | Mainnet |
| Owner/deploy key | Platform Owner | VPS `~/.near-keys` (MVP) / council (mainnet) | PLATFORM_OWNER; mainnet council 2-of-3 | MVP+ |
| Guardian pause key | Platform | VPS / guardian | SECURITY (hanya `pause`) | Mainnet |
| SSH deploy key | Platform | GitHub Secrets | CI deploy job | MVP+ |
| JWT secret | Platform | VPS env | API process | MVP+ |
| Metadata & media | Creator | IPFS/HTTP pihak ketiga | publik (untrusted render) | MVP+ |
| Profil & report (DB) | Platform | PostgreSQL VPS | app role; admin (moderasi) | MVP+ |
| `admin_audit` | Platform | PostgreSQL VPS | app role append-only (no update/delete) | MVP+ |
| Backup DB | Platform | Object storage | backup job (write) + restore manual | MVP+ |
| Domain & DNS | Platform | Registrar | PLATFORM_OWNER + registrar 2FA | MVP+ |

## 8. Rating CIA (Confidentiality / Integrity / Availability)

> Skala per dimensi: **Tinggi** / **Sedang** / **Rendah**. Rating = tingkat kebutuhan perlindungan, bukan tingkat risiko saat ini. **Risk rating** = gabungan dampak × probabilitas setelah kontrol (lihat [threat-model.md](./threat-model.md)).

| Aset | C | I | A | Alasan singkat | Risk rating | SEC-ID |
|---|---|---|---|---|---|---|
| NFT user | Rendah (publik di chain) | **Tinggi** (kepemilikan) | Tinggi (akses user) | chain publik tapi salah transfer = fatal | **Kritis** | SEC-ORDER-004/016 (INV) |
| Escrow Ⓝ offer | Sedang | **Tinggi** (jumlah & tujuan) | Sedang | dana nyata ditahan sementara | **Kritis** | SEC-ORDER-002 |
| Dana settlement | Sedang | **Tinggi** | Sedang | transfer final, tidak bisa dibalik | **Kritis** | SEC-ORDER-001, SEC-CONTRACT-005 |
| Royalti | Rendah | **Tinggi** | Sedang | hak kreator, ≤10% | Tinggi | SEC-CONTRACT-005 |
| Fee treasury | Rendah | Tinggi | Sedang | pendapatan platform | Tinggi | SEC-CONTRACT-004 |
| Insurance fund | Sedang | Tinggi | Sedang | kompensasi insiden | Tinggi | SEC-ORDER-001 |
| Owner/deploy key | **Tinggi** | **Tinggi** | Tinggi | kendali penuh kontrak | **Kritis** | SEC-KEY-001/002 |
| Guardian pause key | Sedang | Tinggi | Tinggi | kill switch | Tinggi | SEC-CONTRACT-012 |
| SSH deploy key | **Tinggi** | Tinggi | Sedang | akses server | Tinggi | SEC-CICD-002 |
| JWT secret | **Tinggi** | Tinggi | Sedang | spoof identitas off-chain | Tinggi | SEC-AUTH-003 |
| Metadata & media | Rendah (publik) | Sedang (hash) | Rendah (pihak ketiga) | untrusted render | Sedang | SEC-META-001/002 |
| Profil & report (DB) | Sedang | Tinggi | Sedang | data user minim-PII | Sedang | SEC-DB-002 |
| `admin_audit` | Rendah | **Tinggi** (immutability) | Sedang | bukti akuntabilitas | Tinggi | SEC-ADMIN-002, SEC-DB-003 |
| Backup DB | Sedang | Tinggi | Tinggi | pemulihan data app | Sedang | SEC-DB-001/004 |
| Domain & DNS | Rendah | **Tinggi** | Tinggi | hijack = phishing penuh | Tinggi | SEC-IR-002 |

## 9. Data flow (tingkat tinggi)

> Hanya aset **milik kita** yang mengalir lewat sistem kita; dana & NFT **tidak pernah** transit API/DB (non-custodial — ADR-007/010).

```text
NFT (NEP-171)      : Wallet user ──(tx)──► NFT contract ──(approval)──► Market contract
                     Market TIDAK PERNAH menyimpan NFT; hanya memanggil nft_transfer_payout saat settle.
                     Indexer (fase 2) hanya MEMBACA event → proyeksi DB (bukan otoritas).

Ⓝ escrow offer      : Wallet buyer ──(attach)──► Market contract (state Offer)
                     → refund ke offer.buyer_id (hardcoded) | distribusi saat accept.

Ⓝ settlement        : Wallet buyer ──(attach)──► Market contract ──(1 receipt)──► seller + royalti + treasury.
                     Tidak ada jalur lewat API/DB.

Fee treasury        : Settlement ──► saldo Market contract ──(withdraw owner/DAO)──► treasury account.

Kredensial off-chain: Wallet user ──(sign NEP-413)──► API ──(nonce/session)──► DB.
                     Private key TIDAK PERNAH menyentuh API/DB.

Metadata            : Creator ──► IPFS/HTTP ──► (dibaca browser / fetcher fase 2) → render.
                     Server MVP TIDAK fetch (FACT, ADR-014).

Data app            : User ──(JSON+sig)──► API ──► PostgreSQL (profiles/reports/blocklist/admin_audit/auth_nonce/sessions).
```

## 10. Retensi per aset

> Kebijakan kanonik sebagian dimiliki [database-security.md](./database-security.md) §6 dan [development/secrets-and-gitignore.md](../development/secrets-and-gitignore.md).

| Aset / data | Retensi | Setelah retensi | Sumber |
|---|---|---|---|
| `auth_nonce` | ≤5 menit (TTL) | hard delete | database-security §6 |
| `sessions` (access) | 15 menit | hard delete | api/authentication |
| `sessions` (refresh) | ≤12 jam | hard delete | api/authentication |
| `profiles` | sampai user hapus | delete row (right-to-erasure) | database-security §1 |
| `reports` | permanen (riwayat moderasi) | arsip | database-security §6 |
| `admin_audit` | permanen (append-only) | arsip + backup | admin-security §5 |
| `events_raw` (fase 2) | permanen (sumber rebuild) | arsip | database-security §6 |
| Backup harian | 7 harian + 4 mingguan + 3 bulanan | hapus otomatis | database-security §4 · disaster-recovery §Jadwal |
| Log aplikasi/auth | sesuai kebijakan log (⏳ open-by-design) | rotasi | infrastructure-security §5 |
| Owner/deploy key | sampai rotasi | revoke + hapus | key-management §4 |
| Metadata/media | selama kontrak ada (immutable on-chain) | permanen | FACT |

## 11. Cakupan SEC-ID per aset

> Setiap aset punya minimal satu requirement yang menjaganya — tidak ada aset yatim.

| Aset | SEC-ID utama | Requirement |
|---|---|---|
| NFT user | SEC-ORDER-004, SEC-CONTRACT-003/005 | dual verification, callback privat, payout tervalidasi |
| Escrow offer | SEC-ORDER-002 | refund hanya ke `buyer_id` |
| Dana settlement | SEC-ORDER-001, SEC-CONTRACT-005 | atomic resolve + payout ≤ harga−fee |
| Royalti | SEC-CONTRACT-005 | ≤10 penerima, `amount > 0`, Σ ≤ harga−fee |
| Fee treasury | SEC-CONTRACT-004 | `fee_bps` ≤ `MAX_FEE_BPS` |
| Insurance fund | SEC-ORDER-001 | kompensasi G7 (pencairan DAO) |
| Owner/deploy key | SEC-KEY-001/002 | key tidak di app/CI/git; mainnet DAO 2-of-3 |
| Guardian pause key | SEC-CONTRACT-012 | guardian `pause_callers` terpisah |
| SSH deploy key | SEC-CICD-002 | secret scanning + tidak usable di repo |
| JWT secret | SEC-AUTH-003/006 | token scoped + revocable |
| Metadata & media | SEC-META-001/002 | render aman + SSRF guard (fase 2) |
| Profil & report | SEC-DB-002 | minim PII, tanpa secret di DB |
| `admin_audit` | SEC-ADMIN-002, SEC-DB-003 | append-only (REVOKE update/delete) |
| Backup DB | SEC-DB-001/004 | backup harian + restore drill |
| Domain & DNS | SEC-IR-002 | kontak darurat + registrar lock |
