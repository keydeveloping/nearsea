# System Architecture

> Menjawab: sistem ini terdiri dari apa saja, dan bagaimana komponen bertukar data.

## Diagram (final ronde 13 + audit)

```text
                  ┌─────────────────────┐
                  │      Frontend       │
                  │   dApp (Next.js)    │
                  └──────┬────────┬─────┘
   view call (RPC)       │        │ REST + wallet tx (sign)
                       ▼ │        ▼
   ┌───────────────────┐ │   ┌──────────────────────┐
   │   NEAR Chain      │ │   │ Report API (VPS)     │
   │ ┌───────────────┐ │   │ Next.js API routes   │
   │ │ NFT Contract  │ │   │ └─ PostgreSQL        │
   │ │ NEP-171/177/  │ │   │   (profiles, reports,│
   │ │ 178/181/199/  │ │   │    audit)            │
   │ │ 297           │ │   └──────────┬───────────┘
   │ └───┬───────────┘ │              │
   │     │ nft_on_approve             │ polling riwayat
   │     │ (NFT→Market)               │ (30–60 dtk)
   │ ┌───▼───────────┐ │   ┌──────────▼───────────┐
   │ │ Market        │ │   │ NearBlocks API       │
   │ │ Contract      │ │   │ (riwayat tx)         │
   │ └───┬───────────┘ │   └──────────────────────┘
   │ ┌───▼───────────┐ │   ┌──────────────────────┐
   │ │ Factory +     │ │   │ RPC Provider         │
   │ │ Launchpad     │ │   │ FASTNEAR → official  │
   │ └───────────────┘ │   │ → dRPC (fallback)    │
   └───────────────────┘   └──────────────────────┘
   Interaksi lintas kontrak (NEAR Chain):
   - listing   : nft_approve (→ nft_on_approve callback) lalu list_nft_for_sale (dual verification: nft_token + nft_is_approved)
   - settlement: nft_transfer_payout (Market → NFT, NEP-199) → resolve_purchase (Market internal)
   [Indexer custom = fase 2, sumber Neardata — proyeksi, tidak pernah otoritas]
```

## Komponen

| Komponen | Tanggung jawab | Teknologi |
|---|---|---|
| NFT Contract (per koleksi) | mint (launchpad), transfer, approval NEP-178, metadata, royalti (NEP-199, cap 10%), enumeration | Rust + near-sdk-contract-tools |
| Market Contract | listing 2-tx, offer escrow, private/bundle, settlement + payout (fee 2%, MAX_FEE_BPS), stale, Pausable | Rust + near-sdk |
| Factory + Launchpad | deploy koleksi, config phase (harga/alokasi/allowlist/waktu), allowlist on-chain set | Rust + near-sdk |
| Report API + PostgreSQL | report, verified badge, blocklist, profil custom, auth nonce — **proyeksi & layanan aplikasi, bukan jalur dana** | Next.js API routes + Prisma |
| Frontend | semua interaksi user; wallet near-connect; **re-verify harga/ownership via view call sebelum sign** | Next.js + TS + Tailwind |
| NearBlocks API | riwayat transaksi (polling sumber FE/API) | pihak ketiga |
| Storage aset | gambar/video/metadata JSON | IPFS (provider ditunda — ronde 5) |

## Alur data kunci

- **Listing (2-tx, ADR-002)**: Seller `nft_approve(market, msg=harga)` di NFT contract (Tx-1) → Seller `list_nft_for_sale` di market (Tx-2) → market dual verification via 2 cross-contract call (`nft_token` ownership + `nft_is_approved`) → callback `process_listing` (`#[private]`) → listing aktif.
- **Pembelian**: Buyer `buy` (listing aktif; attach Ⓝ = harga) ATAU `make_offer` (escrow Ⓝ = nominal offer) → seller `accept_offer` → market `nft_transfer_payout` (settlement + royalti NEP-199) → `resolve_purchase` (`#[private]`) → distribusi seller + royalti − fee 2%, refund otomatis jika gagal.
- **Read**: listing/ownership = view call on-chain (sumber kebenaran); riwayat = NearBlocks; profil/report = Report API.

## Keputusan terkait (terkunci)

- Single market contract untuk list+offer+private/bundle (auction = fase 2).
- MVP **tanpa indexer sendiri** — tetapi ada Report API kecil + PostgreSQL (ronde 3/5).
- Koleksi open via factory + badge verified (kurasi off-chain, flag di DB Report API — lihat backend-architecture.md).
- Model listing approval non-custodial (ADR-007); orderbook on-chain tanpa off-chain order signature (ADR-012).

---

## C4 — Level 1: System Context

> Siapa memakai sistem dan sistem eksternal apa yang disentuh. Traceability: ADR-004 (data layer MVP), ADR-010 (chain-centric boundary), ADR-014 (metadata isolation).

```text
        ┌─────────────────────────────────────────────────────────────────────┐
        │                            AKTOR                                     │
        │  ┌───────────┐   ┌───────────┐   ┌───────────┐   ┌───────────────┐   │
        │  │  Trader   │   │  Kreator  │   │   Admin   │   │  Auditor/DAO  │   │
        │  │ (buy/sell)│   │ (mint/list)│  │ (moderasi)│   │ (owner mainnet)│  │
        │  └─────┬─────┘   └─────┬─────┘   └─────┬─────┘   └───────┬───────┘   │
        └────────┼───────────────┼───────────────┼─────────────────┼──────────┘
                 │               │               │                 │
                 ▼               ▼               ▼                 ▼
        ┌─────────────────────────────────────────────────────────────────────┐
        │                        NearSea (sistem)                              │
        │   Frontend dApp · Kontrak NFT/Market/Factory · Report API · DB       │
        └──────┬──────────────────────┬───────────────────────┬───────────────┘
               │                      │                       │
               ▼                      ▼                       ▼
        ┌─────────────┐       ┌───────────────┐       ┌───────────────────┐
        │ NEAR Chain  │       │ NearBlocks API│       │ IPFS Gateway      │
        │ (RPC node)  │       │ (riwayat tx)  │       │ (media/metadata)  │
        └─────────────┘       └───────────────┘       └───────────────────┘
```

| Aktor / sistem eksternal | Hubungan dengan NearSea | Batas kepercayaan |
|---|---|---|
| Trader (buyer/seller) | Baca discovery, sign tx via wallet | Memegang kunci sendiri; FE tidak pernah sign atas namanya |
| Kreator | Mint koleksi (launchpad), list NFT | Sama seperti trader |
| Admin | Moderasi report, verified badge, blocklist | Allowlist DB + scope `admin` + step-up (SEC-ADMIN-001/003) |
| Auditor / DAO council | Owner actions mainnet (upgrade/fee/pause) | Sputnik DAO 2-of-3 + timelock (ADR-013) |
| NEAR Chain + RPC | Otoritas tunggal dana/ownership | **Dipercaya sebagai otoritas** (ADR-010); RPC hanya pintu masuk |
| NearBlocks API | Proyeksi riwayat/aktivitas | **Tidak dipercaya** — tidak pernah memicu tx |
| IPFS gateway | Hosting media/metadata | Konten untrusted (ADR-014, metadata-security.md) |

## C4 — Level 2: Container

> Kontainer = unit yang bisa di-deploy. MVP = 1 VPS (ADR-009); indexer = fase 2.

```text
┌────────────────────────────── Browser (untrusted client) ──────────────────────────────┐
│  Next.js App (client bundle)  ──  near-connect wallet adapter  ──  Zustand stores       │
│    TanStack Query (cache read) · i18n · design-system primitives                        │
└───────┬───────────────────────────────┬───────────────────────────┬────────────────────┘
        │ HTTPS (view call / RPC)       │ HTTPS REST (auth/profil)  │ wallet popup (sign)
        ▼                               ▼                           ▼
┌───────────────────┐        ┌──────────────────────┐     ┌──────────────────────┐
│ RPC Provider      │        │ Caddy (TLS, headers, │     │ NEAR Wallet /        │
│ FASTNEAR→official │        │ rate limit, static)  │     │ near-connect provider│
│ →dRPC (fallback)  │        └──────────┬───────────┘     └──────────┬───────────┘
└───────────────────┘                   ▼                            │
                              ┌──────────────────────┐                │
                              │ Next.js server       │                │
                              │ (SSR + API routes)   │                │
                              │ auth/profil/report/  │                │
                              │ admin                │                │
                              └───────┬──────┬───────┘                │
                                      │      │                        │
                                      ▼      ▼                        ▼
                        ┌──────────────┐  ┌────────────────┐  ┌───────────────┐
                        │ PostgreSQL   │  │ NearBlocks API │  │ NEAR Chain    │
                        │ (app-mutable)│  │ (proxy riwayat)│  │ NFT + Market  │
                        └──────────────┘  └────────────────┘  └───────────────┘
```

| Kontainer | Teknologi | State | Skala | Catatan |
|---|---|---|---|---|
| Browser app | Next.js App Router (client) | localStorage (wallet/read-state) | per-user | Tidak pernah simpan private key (features/auth.md) |
| Caddy | Reverse proxy + TLS | Config file | 1 proses | Let's Encrypt otomatis; security headers |
| Next.js server | Node runtime (SSR + API routes) | Stateless | 1→N (scaling.md) | Sesi JWT, bukan in-memory |
| PostgreSQL | PostgreSQL + Prisma | Persisten (volume) | 1 → primary+replica (fase 2) | App-mutable + proyeksi fase 2 |
| Indexer (fase 2) | Worker Neardata → Postgres | Checkpoint tersimpan | 1→N worker | Proyeksi saja (SEC-INDEX-001) |
| Kontrak on-chain | Rust + near-sdk | Chain state | 1 market, N NFT | Otoritas tunggal |

## Antarmuka komponen (protocol / port / base URL)

> Semua alamat di bawah memakai placeholder kanonik (`nearsea.example`, `<account>`); nilai final diisi saat deploy ([environments.md](../deployment/environments.md)). Port internal VPS ditandai PROPOSED sampai scaffold.

| Komponen | Protokol / antarmuka | Port | Base URL / alamat | Auth | Status |
|---|---|---|---|---|---|
| Frontend (browser) | HTTPS | 443 | `https://nearsea.example` (placeholder) | wallet (near-connect) | ✅ |
| Caddy | HTTPS in → HTTP out | 443→3000 | `https://<domain>` | — | ✅ |
| Next.js server | HTTP (internal) | 3000 (PROPOSED) | `http://127.0.0.1:3000` | — | ✅ |
| Report API | REST JSON | via Caddy `/api/*` | `https://nearsea.example/api/*` | JWT scope + signature | ✅ |
| PostgreSQL | PostgreSQL wire | 5432 (localhost only) | `postgresql://…@127.0.0.1:5432/nearsea` | DB role non-superuser | ✅ |
| PgBouncer (fase 2) | PostgreSQL wire | 6432 (PROPOSED) | `127.0.0.1:6432` | passthrough | PROPOSED |
| Indexer worker (fase 2) | Neardata stream | outbound 443 | `https://<neardata-endpoint>` | API key (bila perlu) | PROPOSED |
| RPC provider | JSON-RPC 2.0 | 443 | `https://test.rpc.fastnear.com` → `https://rpc.testnet.near.org` → `https://near-testnet.drpc.org` | publik / API key | ✅ |
| NearBlocks API | REST | 443 | `https://api.nearblocks.io` | API key (opsional) | ✅ |
| IPFS gateway | HTTP gateway | 443 | `https://ipfs.dweb.link`, `https://ipfs.io` (allowlist) | publik | ⏳ open-by-design |
| Market contract | NEAR contract | — | `market.<account>.testnet` | predecessor + 1 yocto | ✅ |
| NFT contract (per koleksi) | NEAR contract | — | `<koleksi>.nearsea.testnet` | NEP-178 approval | ✅ |
| Factory + Launchpad | NEAR contract | — | `factory.<account>.testnet` | owner / phase rules | ✅ |
| Telegram Bot API | HTTPS | 443 | `https://api.telegram.org` | bot token (server) | ✅ |

> **Catatan port:** daftar port final (3000/5432/6432) ditandai PROPOSED — dikunci saat scaffold (TASK-001) & Docker Compose ([backend-architecture.md](./backend-architecture.md), [infrastructure.md](./infrastructure.md)).

## Sequence — Listing (2 transaksi + dual verification)

> Traceability: ADR-002, ADR-007, SEC-ORDER-004, INV-016, RESEARCH.md §10.4.

```text
Seller      NFT Contract        Market Contract           RPC (view)
  │              │                    │                        │
  │─ nft_approve(market, msg=harga, approval_id) ─►│           │
  │   (1 yocto + storage approval)  │                          │
  │◄─ approval tersimpan ───────────│                          │
  │              │                    │                        │
  │─ list_nft_for_sale{contract,token,approval_id,price} ─►│   │
  │              │                    │  cek storage_deposit (NEP-145)
  │              │                    │─ nft_token(token_id) ─►│
  │              │                    │─ nft_is_approved(token,market,approval_id) ─►│
  │              │                    │◄─ owner_id + approved ─│
  │              │  process_listing (#[private], callback ganda)
  │              │                    │  assert owner==seller && approved==true
  │              │                    │  simpan Sale + index          (SEC-ORDER-004)
  │◄─ listing ACTIVE ────────────────│                          │
```

- **Kegagalan di antara langkah**: approve sukses tetapi list gagal → approval menggantung; seller bisa list ulang atau `nft_revoke` (ADR-002 Consequences). Tidak ada dana hilang.
- **Verifikasi ulang saat settle**: ownership + approval dicek lagi sebelum `nft_transfer_payout` (INV-016) — listing bisa stale antara list dan buy.

## Sequence — Buy settlement

> Traceability: ADR-005 (fee 2% on-chain), ADR-012, SEC-ORDER-001/004/006, INV-001/002/003/011.

```text
Buyer        Market Contract        NFT Contract          Treasury   Seller/Royalti
  │               │                     │                    │            │
  │─ buy(contract,token) attach Ⓝ=harga ─►│                   │            │
  │               │ cek: sale ada, buyer≠seller, deposit≥harga (INV-023)
  │               │ hapus Sale (optimistic removal — SEC-ORDER-006)
  │               │─ nft_transfer_payout(buyer, token, approval_id, price, max=10) ─►│
  │               │   (1 yocto + 15 Tgas)                     │            │
  │               │◄─ Payout object (seller+royalti) ─────────│            │
  │               │ resolve_purchase (#[private], 115 Tgas)   │            │
  │               │  validasi: ≤10 penerima, amount>0, Σpayout≤harga−fee (INV-002/003)
  │               │─ fee 2% (fee_bps=200) ─────────────────►│            │
  │               │─ proceeds seller + royalti ──────────────────────────►│
  │◄─ NFT pindah + receipt final ──────────────────────────────────────────│
  │               │                                                      │
  │        [GAGAL: promise/payout invalid → resolve memulihkan Sale + refund penuh
  │         ke buyer (buyer_id hardcoded — SEC-ORDER-002); listing tetap ada bila
  │         kegagalan bukan stale — INV-001]
```

- **Race (20 pembeli 1 NFT)**: pembeli pertama menghapus Sale; pembeli ke-2..20 revert → deposit kembali otomatis. Pemenang ditentukan urutan eksekusi on-chain, bukan RPC ([concurrency-and-races.md](../development/concurrency-and-races.md), SEC-ORDER-006).
- **Bundle**: pre-validasi semua item sebelum transfer pertama (INV-025); kegagalan residual → status PARTIAL + kompensasi G7 ([order-protocol-security.md](../security/order-protocol-security.md) §6).

## Failure & retry semantics

> Prinsip: kegagalan lapisan off-chain **tidak pernah** mengubah kebenaran settlement (chain otoritatif — ADR-010).

| Komponen gagal | Gejala | Semantik retry | Sumber/rujukan |
|---|---|---|---|
| RPC read (view call) | timeout / 429 | Failover terkunci FASTNEAR → official → dRPC; health check banding block hash | [infrastructure.md](./infrastructure.md) |
| RPC write (kirim tx) | 429 / timeout | Retry dengan backoff; **cek status via tx hash dulu** sebelum kirim ulang (hindari duplikat) | features/auth.md, concurrency-and-races.md §5 |
| NearBlocks | 5xx / rate limit | Retry backoff; badge/riwayat berhenti update, UI tetap berfungsi | features/notifications.md |
| Report API (Next.js) | 5xx | Client retry idempoten (GET) / idempotency key (POST report per hari) | api-security-architecture.md |
| PostgreSQL | down | API 503; baca discovery (RPC) tetap jalan karena FE tidak lewat API untuk discovery | ADR-004 |
| IPFS gateway | 404 / timeout | Ganti gateway allowlist; kegagalan = placeholder, bukan error user | metadata-security.md |
| Kontrak paused | tx revert `CHAIN_PAUSED` | Cancel/withdraw tetap boleh; aksi tulis ditolak sampai unpause | INV-022, SEC-CONTRACT-007 |
| Settlement promise gagal | revert callback | Refund otomatis + state pulih (listing aktif lagi) | SEC-ORDER-001, INV-001 |
| Indexer lag (fase 2) | data basi | Indikator "data mungkin basi"; data uang tetap dari view call | indexer-security.md §1 |

**Idempotensi**: operasi write API yang boleh diulang memakai kunci idempoten (report: `(account, target, hari kalender)`); notifikasi memakai `(receipt_id, event_index)` ([error-handling.md](../development/error-handling.md) §6).

## Topologi environment

> Mapping branch → environment → deploy: [development/git-workflow.md](../development/git-workflow.md) & [ci-cd.md](../development/ci-cd.md). Detail variabel: [deployment/environments.md](../deployment/environments.md).

```text
local            dev/staging              testnet (publik)         mainnet
─────            ───────────              ────────────────         ───────
localnet/        RPC testnet              RPC testnet              RPC mainnet
sandbox          kontrak dev.*            kontrak *.testnet        kontrak final
deploy/test      subdomain staging        subdomain testnet        domain produksi
seed data        DB terpisah              DB terpisah              DB produksi
dev wallet       dev wallet               wallet tim               user nyata
                 branch: dev              branch: testnet          branch: mainnet
                                          gate CI + 1 approval     manual + reviewer
```

| Env | Chain | Kontrak | API/DB | Secrets | Deploy |
|---|---|---|---|---|---|
| local | localnet/sandbox | deploy per test | opsional (Docker) | `.env` lokal | manual |
| dev/staging | testnet | `dev.*` terpisah | DB staging terpisah | secret dev | otomatis push `dev` |
| testnet | testnet | `<nama>.testnet` | DB testnet | secret testnet | otomatis push `testnet` (gate CI) |
| mainnet | mainnet | akun final | DB produksi | secret produksi | manual `workflow_dispatch` + reviewer |

- **Owner key**: testnet = single owner key (interim, ADR-013); mainnet = Sputnik DAO V2 council 2-of-3 + timelock.
- **Treasury**: ⏳ open-by-design — ditetapkan saat deploy testnet ([features/payments.md](../features/payments.md)).

## Anggaran gas lintas-kontrak

> Nilai bertanda **FACT/riset** berasal dari [RESEARCH.md](../../RESEARCH.md) §10; nilai estimasi ditandai **PROPOSED**. Batas keras: **300 Tgas per call** (FACT NEAR).

| Langkah | Gas | Sumber | Catatan |
|---|---|---|---|
| `nft_approve` (NFT contract) | ~10–15 Tgas (PROPOSED) | riset pola NEP-178 | Deposit 1 yocto + biaya storage approval (NEP-145) |
| `list_nft_for_sale` (market) | **≈10–20 Tgas total** (PROPOSED) | Tulis ~5 + 2 view call (dual verification ~3–5 masing-masing); SSOT total = order-protocol-security §11 |
| `nft_token` view (cross-contract) | ~3–5 Tgas (PROPOSED) | — | Bagian dual verification |
| `nft_is_approved` view (cross-contract) | ~3–5 Tgas (PROPOSED) | — | Bagian dual verification |
| `process_listing` callback | sisanya dari budget list | — | `#[private]`, callback ganda |
| `nft_transfer_payout` | **15 Tgas** (FACT/riset) | RESEARCH.md §10.2 | 1 yocto + NEP-199 payout |
| `resolve_purchase` | **115 Tgas** (FACT/riset) | RESEARCH.md §10.2 | Distribusi seller + royalti + treasury |
| `buy` total (worst case) | < 150 Tgas | — | Masih jauh di bawah 300 Tgas |
| `buy_bundle` (10 token) | PROPOSED — perlu diukur | — | Loop `nft_transfer_payout` × 10 → wajib diuji batas gas (INV-021) |

- **Risiko gas**: `buy_bundle` dengan 10 item adalah jalur paling rawan gas; batas statis 10 item (INV-021) adalah kontrolnya. Pengukuran nyata dilakukan di test sandbox (TASK terkait, [testing-strategy.md](../testing/testing-strategy.md)).
- **Royalti FT (fase 2)**: payout FT + royalti banyak akun harus dipangkas — batas 10 penerima adalah batas gas yang sehat ([RESEARCH.md](../../RESEARCH.md) §11).

## Kepemilikan komponen (ownership)

| Komponen | Owner | Perubahan menyentuh | Gate |
|---|---|---|---|
| Market contract | Tim (owner key → DAO mainnet) | ADR + features/marketplace + tests + SEC/INV | Audit eksternal sebelum mainnet (TASK-027) |
| NFT contract (template) | Tim | Standar NEP + tests | Reproducible build NEP-330 |
| Factory + Launchpad | Tim | ADR-008 + tests | Audit eksternal |
| Frontend | Tim FE | 04-ux-ui-spec + frontend-architecture | TASK-008b branding |
| Report API + DB | Tim BE | api/* + database/* + SEC-API/DB | Endpoint wajib test |
| CI/CD | Tim infra | development/ci-cd + cicd-security | Proteksi 3 branch (SEC-CICD-003) |
| Indexer (fase 2) | Tim BE | indexer-security + ADR-015 | SEC-META-002 prasyarat |
| Infra VPS | Tim infra | infrastructure.md + infrastructure-security | Restore drill (SEC-DB-004) |

> **Perubahan keputusan arsitektur** wajib lewat ADR baru/update + [DOCUMENTATION-MAP.md](../DOCUMENTATION-MAP.md) trigger T4.
