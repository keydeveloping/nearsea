# Riset: Membuat NFT Marketplace di NEAR Blockchain

> Sumber: NEAR Docs (docs.near.org) via near_docs MCP — diriset 2026-09-30

---

## 1. Arsitektur Dasar di NEAR

Poin paling penting dari riset: **NFT dan Marketplace adalah DUA kontrak yang berbeda**.

> "NFT simply store information (metadata), while NFT-marketplaces are contracts where NFT can be listed and exchanged for a price." — docs.near.org/primitives/nft/standard

NFT **tidak disimpan di wallet** user. Semua NFT hidup di dalam *NFT contract* yang bertindak sebagai bookkeeper (mint, simpan, transfer). Wallet hanya menampilkan kepemilikan.

Arsitektur yang direkomendasikan (pola umum di ekosistem NEAR, mis. Mintbase, Paras):

```
┌─────────────────┐        nft_approve (NEP-178)      ┌──────────────────────┐
│  NFT Contract   │◄──────────────────────────────────│      Frontend        │
│ (NEP-171/177/   │        nft_transfer               │  (near-connect +     │
│  178/181/297)   │◄──────────────────────────────────│   near-api-js)       │
└─────────────────┘                                   └──────────────────────┘
        │  nft_on_approve (callback saat listing)              ▲
        ▼                                                      │ buy (attach NEAR)
┌──────────────────────┐      nft_transfer + approval_id       │
│ Marketplace Contract │◄──────────────────────────────────────┘
│ - listings/sales     │      payout → seller + royalties + fee
│ - harga, payout split│
└──────────────────────┘
```

**Alur jual-beli (pola approval — standar di NEAR):**
1. Seller memanggil `nft_approve` di NFT contract dengan `account_id = marketplace` + `msg` (berisi harga, bisa stringified JSON) + deposit untuk biaya storage approval.
2. NFT contract memanggil callback `nft_on_approve(owner_id, approval_id, msg)` di marketplace contract.
   > **CATATAN PROYEK**: di NearSea, listing **TIDAK dibuat di callback** — `nft_on_approve` hanya menerima approval; listing dibuat lewat `list_nft_for_sale` dengan **dual verification** (lihat bagian 10.4). Pola callback-saja adalah pola tutorial lama.
3. Buyer membayar (attach deposit NEAR) ke marketplace.
4. Marketplace memanggil `nft_transfer` dengan `approval_id` yang sesuai → NFT pindah ke buyer.
5. Marketplace membagi pembayaran: seller + royalti kreator + fee marketplace (payout split).
6. Alternatif pola: `nft_transfer_call` — transfer NFT + trigger `nft_on_transfer` di kontrak penerima dalam 1 transaksi (dipakai untuk escrow/auction).

---

## 2. Standar (NEP) yang Wajib & Direkomendasikan

| NEP | Nama | Peran di Marketplace |
|-----|------|---------------------|
| **NEP-171** | NFT Core | WAJIB. `nft_token`, `nft_transfer`, `nft_transfer_call`, `nft_resolve_transfer` |
| **NEP-177** | NFT Metadata | WAJIB. Metadata kontrak (`spec, name, symbol, icon, base_uri, reference`) & token (`title, description, media, media_hash, copies, issued_at, ...`) |
| **NEP-178** | Approval Management | **KUNCI MARKETPLACE**. `nft_approve`, `nft_revoke`, `nft_revoke_all`, `nft_is_approved` — mengizinkan kontrak lain (marketplace) transfer NFT atas nama owner |
| **NEP-199** | NFT Royalty | `nft_transfer_payout` — transfer + objek Payout royalti sekaligus; basis distribusi royalti on-chain di settlement |
| **NEP-181** | Enumeration | Query: `nft_total_supply`, `nft_tokens`, `nft_tokens_for_owner`, dll. — untuk halaman koleksi & profil |
| **NEP-145** | Storage Management | WAJIB untuk marketplace/FT: user membayar biaya storage datanya sendiri (`storage_deposit`, `storage_unregister`) |
| **NEP-297** | Events Format | Emit event `nft_mint`, `nft_transfer`, `nft_burn` via log `EVENT_JSON:{...}` agar explorer/indexer/indexing frontend bisa melacak aktivitas |

Detail antarmuka NEP-171:

```ts
nft_token(token_id): Token | null                          // read-only
nft_transfer(receiver_id, token_id, approval_id?, memo?)   // wajib attach tepat 1 yoctoNEAR
nft_transfer_call(receiver_id, token_id, approval_id?, memo?, msg): Promise
nft_resolve_transfer(owner_id, receiver_id, token_id, approved_account_ids?): boolean  // callback refund
// kontrak penerima NFT wajib implement:
nft_on_transfer(sender_id, previous_owner_id, token_id, msg): boolean  // true = kembalikan
```

Detail NEP-178 (approval):
```ts
nft_approve(token_id, account_id, msg?)      // deposit: 1 yoctoNEAR + biaya storage approval
nft_revoke(token_id, account_id)             // 1 yoctoNEAR
nft_revoke_all(token_id)                     // 1 yoctoNEAR
nft_is_approved(token_id, approved_account_id, approval_id?): boolean
```

Royalti: struktur payout berisi daftar akun + persentase; setiap penjualan membagi harga ke kreator (mis. `anna` 5% → dapat 5% harga jual tiap kali NFT terjual).

---

## 3. Aturan Keamanan & Ekonomi yang Unik di NEAR

1. **1 yoctoNEAR (10⁻²⁴ Ⓝ) pada transfer** — `nft_transfer` wajib menempelkan tepat 1 yoctoNEAR. Tujuannya keamanan: *function-call key* (kunci limited-permission, mis. untuk game) tidak bisa menempel deposit, jadi transfer selalu lewat wallet confirmation. Selalu `assert_one_yocto()` + cek ownership.
2. **Storage staking** — kontrak membayar storage: 1E19 yoctoNEAR/byte ≈ **100 KB per 1 Ⓝ**. Gunakan NEP-145 agar user membayar slot datanya sendiri (approval, listing). Jika tidak, attacker bisa menggembungkan state sampai kontrak kehabisan saldo (storage cost attack).
3. **`predecessor_account_id` vs `signer_account_id`** — gunakan `predecessor` untuk otorisasi pemanggil langsung (mis. marketplace memanggil NFT contract); `signer` hanya jika aturan memang tentang penanda tangan transaksi asli.
4. **Verifikasi kepemilikan NFT saat listing** — dari tutorial auction: kontrak auction tidak memverifikasi NFT milik siapa; bad actor bisa membuat auction NFT yang bukan miliknya, `nft_transfer` gagal, dan bidder kehilangan dana. Marketplace harus cek `nft_token(token_id).owner_id` sebelum menerima listing (bisa off-chain di frontend atau on-chain via cross-contract call).
5. **Nilai selalu string yoctoNEAR (u128)** di JSON — `1 Ⓝ = "1000000000000000000000000"`. Konversi dengan `NEAR.fromDecimal()` (JS) / `NearToken` (Rust).
6. **Callback `#[private]`** — fungsi resolve (mis. `nft_resolve_transfer`) harus privat (hanya bisa dipanggil kontrak sendiri).
7. **Checklist keamanan resmi**: access control, batas ukuran input/string, tolak duplikat sebelum pembayaran, dsb. — docs.near.org/smart-contracts/security/checklist

---

## 4. Stack Pengembangan (Rekomendasi Resmi Docs)

**Bahasa: Rust (sangat direkomendasikan untuk production NFT).** Setup:

```bash
# 1. Rust + target WASM
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# 2. NEAR CLI (deploy & interaksi)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/near-cli-rs/releases/latest/download/near-cli-rs-installer.sh | sh

# 3. cargo-near (scaffold & build/deploy)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/cargo-near/releases/latest/download/cargo-near-installer.sh | sh

# 4. Scaffold project baru
cargo near new marketplace
```

**Library Rust:**
- `near-sdk` — framework utama smart contract NEAR
- `near-contract-standards` — implementasi standar siap pakai (NFT: NEP-171/177/178/181) di dalam near-sdk
- `near-sdk-contract-tools` (github.com/near/near-sdk-contract-tools) — derive macros untuk NEP-141/145/171/177/178/181/297 + pattern Owner, Pause, Escrow, RBAC → mengurangi boilerplate drastis. Tutorial resmi NFT sekarang memakai ini.

**Tooling & testing:**
- Sandbox/workspaces test — deploy multiple contracts (NFT + marketplace) di local sandbox
- `near-cli-rs` / Lantstool — interaksi kontrak via CLI
- Localnet untuk dev loop cepat (docs: web3-apps/tutorials/localnet)

**Deploy & testnet:**
- Buat akun testnet (testnet.mynearwallet.com), faucet
- Deploy biasa: `near deploy <account> --wasmFile contract.wasm --initFunction new`
- **Global contract**: deploy sekali + reuse hemat storage — NFT global di testnet: `nft.globals.primitives.testnet` (by account, updatable) atau by hash (immutable)
- Referensi implementasi: **github.com/near-examples/NFT** (implementasi resmi; uji dengan `near call nft.examples.testnet nft_mint ...`)

---

## 5. Sisi Frontend (dApp)

| Kebutuhan | Alat | Catatan |
|---|---|---|
| Login wallet | **near-connect** (zero-dependency, sandboxed) atau near-api-js; hooks: `near-connect-hooks` / `useNearWallet` | Support HOT, Meteor, Nightly, WalletConnect, 10+ wallet. Panduan: web3-apps/tutorials/wallet-login |
| Read/view calls (daftar NFT, listing, harga) | `JsonRpcProvider` (near-api-js) → `provider.callFunction(...)` atau RPC `query` `view` | Endpoint contoh: `https://test.rpc.fastnear.com`, `https://rpc.testnet.near.org` (banyak provider lain; catatan: FASTNEAR gratis publik untuk mainnet = `free.rpc.fastnear.com`, testnet FASTNEAR berbayar) |
| Write calls (mint, list, buy) | wallet: `callFunction({contractId, methodName, args, deposit, gas})` | Deposit harga beli = yoctoNEAR string |
| Konversi unit | `NEAR.fromDecimal('1.5')` / `NearToken` | Jangan pakai `number` untuk u128 |
| Riwayat transaksi/sales history | **NEP-297 events** + indexer; API praktis: **NearBlocks API** (api.nearblocks.io) | RPC tidak menyediakan data historis penuh |
| Metadata media | IPFS (`media` URL + `media_hash` sha256 base64; `base_uri` untuk batch) | Contoh: `...ipfs.dweb.link/` |

---

## 6. Kurva Belajar: Tutorial Resmi (Jalur Tercepat)

**NFT Zero-to-Hero** (10 langkah, Rust) — smart-contracts/tutorials/zero-to-hero/nfts:
1. Pre-deployed contract (mint tanpa coding) → 2. Arsitektur kontrak → 3. Minting → 4. Upgrade kontrak → 5. Enumeration → 6. Core (transfer) → 7. Events → 8. **Approvals** → 9. **Royalty** → 10. **Marketplace** (beli & jual) — https://near-examples.github.io/nft-tutorial/marketplace

> 🔗 **Situs tutorial lengkap**: https://near-examples.github.io/nft-tutorial/ — sidebar berisi Introduction, Pre-deployed Contract, Minting, Upgrade a Contract, Enumeration, Transfers, Approvals, Royalty, Events, Marketplace, dan halaman bonus *Lazy Minting, Collections, and More!* (`/nft-tutorial/series`).
> 🔗 **Repo kode tutorial**: https://github.com/near-examples/nft-tutorial — studinya dianalisis mendalam di bagian 10.

**Mastering NEAR** (web3-apps/tutorials/mastering-near):
- 3.1-nft: auction yang memenangkan NFT — belajar cross-contract call (`ext_contract` trait, attach 30 Tgas + 1 yoctoNEAR), mint NFT, verifikasi ownership, testing 2 kontrak di sandbox
- 2.1-frontend: integrasi frontend dengan wallet + RPC
- 2.2-indexing: data historis via NearBlocks

Lainnya: smart-contracts/quickstart (kontrak pertama + auction), web3-apps/quickstart (dApp penuh).

---

## 7. Kontrak Referensi Produksi (dari Contracts List resmi)

| Kontrak | Proyek | Tag | Catatan |
|---|---|---|---|
| [nft-tutorial → market-contract](https://github.com/near-examples/nft-tutorial) | near-examples | NFT, market | Implementasi marketplace edukatif langkah-demi-langkah — dianalisis mendalam di bagian 10 |
| [mb-interop-market](https://github.com/Mintbase/mb-contracts/tree/main/mb-interop-market) | MintBase | NFT market | **NFT market contract** — acuan langsung marketplace |
| [mb-nft-v2](https://github.com/Mintbase/mb-contracts/tree/main/mb-nft-v2) | MintBase | NFT | NFT contract produksi |
| [mb-factory-v2](https://github.com/Mintbase/mb-contracts/tree/main/mb-factory-v2) | MintBase | factory, NFT | Factory kontrak NFT per-koleksi |
| [NFT-Minting-withFT](https://github.com/joe-rlo/NFT-Minting-withFT) | ShardDog | NFT, FT | Mint NFT pakai FT + **royalties, payment distribution, premint** |
| [ShardDog-NFT-Protocol](https://github.com/joe-rlo/ShardDog-NFT-Protocol) | ShardDog | NFT | Multi-koleksi |
| [near-examples/NFT](https://github.com/near-examples/NFT) | NEAR | NFT | Implementasi referensi resmi |
| [sweat-booster](https://github.com/sweatco/sweat-booster) | Sweat Economy | FT, NFT | NFT vouchers |

---

## 8. Fitur Lanjutan (Opsional, dari Docs)

- **Cross-chain NFT marketplace** — Chain Signatures + Omnibridge: user bayar NFT pakai BTC/crypto eksternal; NFT logic tetap di NEAR; NFT bahkan bisa di-mint di chain lain (chain-abstraction docs).
- **Attach NFT ke call** — `nft_transfer_call` untuk escrow/auction/staking NFT dalam 1 transaksi.
- **Meta-transactions** — relayer bayar gas user (web3-apps/tutorials/meta-transactions) untuk onboarding tanpa gas.
- **Global contracts** — hemat biaya deploy & storage saat meluncurkan banyak kontrak NFT.
- **Indexers / BigQuery / Data API** — untuk fitur analytics, orderbook historis, aktivitas koleksi.

---

## 9. Roadmap Implementasi yang Disarankan

1. **Setup**: Rust + wasm target + cargo-near + near-cli-rs; akun testnet.
2. **NFT Contract** (Rust + `near-sdk-contract-tools`): derive NEP-171/177/178/181/199/297 → `new`, `nft_mint` (dengan royalty payout structure), `nft_transfer`, approvals, enumeration.
3. **Marketplace Contract**: storage management (NEP-145) untuk listing; `nft_on_approve` (receiver approvals) untuk membuat listing dari `msg` (JSON: `{sale_conditions: {near: "..."}, royalties: {...}}`); `offer`/`buy` (attach deposit Ⓝ) → cek harga → payout split (royalti + fee) → `nft_transfer` dengan approval_id → update/hapus listing.

   > **CATATAN PROYEK (ronde 16)** — langkah ini menggambarkan **tutorial**, bukan keputusan NearSea. Di NearSea listing **tidak** dibuat dari callback `nft_on_approve`: listing = **2 transaksi** (`nft_approve` lalu `list_nft_for_sale`) + dual verification (ADR-002), model approval non-custodial + auto-stale (ADR-007). Nama method kanonik NearSea: `buy`, `make_offer`, `accept_offer` (bukan `offer`). Detail: [docs/features/marketplace.md](docs/features/marketplace.md).
4. **Testing**: sandbox workspaces — deploy NFT + marketplace bersama, uji mint → approve → buy → payout, edge cases (harga berubah, revoke, storage).
5. **Frontend**: React + near-connect (login), JsonRpcProvider (view), wallet transaction (mint/list/buy), render metadata IPFS, polling event via NearBlocks API.
6. **Deploy testnet** → audit checklist keamanan (security/checklist) → deploy mainnet.

---

## 10. Studi Mendalam: `market-contract` (near-examples/nft-tutorial)

Sumber yang dipelajari:
- Situs tutorial: https://near-examples.github.io/nft-tutorial/
- Repo kode: https://github.com/near-examples/nft-tutorial
- Halaman marketplace: https://near-examples.github.io/nft-tutorial/marketplace
- Source: `market-contract/src/{lib.rs, sale.rs, sale_views.rs, nft_callbacks.rs, internal.rs, external.rs}`

> Catatan resmi tutorial: ini contoh edukatif — *"there is no canonical implementation"* (NEAR tidak memaksakan satu bentuk marketplace; desain bebas selama patuh NEP-171/178).

### 10.1 Struktur repo tutorial

- Folder per tahap: `nft-contract-skeleton` → `nft-contract-basic` (minting) → `nft-contract-approval` → `nft-contract-royalty` → `nft-contract-events` → `market-contract` → `nft-series` (bonus: lazy minting, koleksi, allowlist) + `integration-tests/` (dijalankan CI GitHub Actions `tests.yml`).
- Setiap tahap juga dipetakan ke **git branch** (`1.skeleton`, `2.minting`, `3.enumeration`, `4.core`, `5.approval`, `6.royalty`, `7.events`, `8.marketplace`) — tiap branch adalah kontrak yang sudah jadi sebagian.
- Build & deploy: `git clone … && git switch 6.royalty && yarn build` → `near deploy` dengan `out/main.wasm` → init `new_default_meta` → mint `nft_mint` (attach 0.1 Ⓝ) → transfer pakai 1 yoctoNEAR.
- Toolchain tutorial: Rust (NEAR SDK ~4.0), NEAR-CLI, yarn, cargo-near.

### 10.2 Struktur kode `market-contract`

| File | Isi |
|---|---|
| `lib.rs` | State kontrak, init `new(owner_id)`, storage management NEP-145 |
| `sale.rs` | Logika inti jual-beli: listing, offer, purchase, payout |
| `sale_views.rs` | Enumeration listing (query untuk frontend) |
| `nft_callbacks.rs` | Callback dari NFT contract (`nft_on_approve`) |
| `internal.rs` | Helper `internal_remove_sale`, `hash_account_id` |
| `external.rs` | Interface cross-contract via macro `ext_contract` |

**State kontrak:**

```rust
pub struct Contract {
    pub owner_id: AccountId,
    pub sales: UnorderedMap<ContractAndTokenId, Sale>,                 // key: "nft_contract_id.token_id"
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>, // index per seller
    pub by_nft_contract_id: LookupMap<AccountId, UnorderedSet<TokenId>>,     // index per NFT contract
    pub storage_deposits: LookupMap<AccountId, NearToken>,             // deposit storage tiap akun
}

pub struct Sale {
    pub owner_id: AccountId,              // seller
    pub approval_id: u64,                 // approval ID marketplace di NFT contract
    pub nft_contract_id: String,
    pub token_id: String,
    pub sale_conditions: SalePriceInYoctoNear, // harga dalam yoctoNEAR
}
```

- Key listing unik = `format!("{}{}{}", contract_id, ".", token_id)` — kontrak NFT + delimiter + token id, sehingga satu marketplace bisa melayani **banyak koleksi/kontrak NFT**.
- Index `by_owner_id` memakai kunci **hash** akun (`hash_account_id`) untuk menghindari collision pada nested set (pola `StorageKey` + `BorshStorageKey` dengan inner-key `{ account_id_hash: CryptoHash }`).
- Gas: `GAS_FOR_RESOLVE_PURCHASE = 115 Tgas`, `GAS_FOR_NFT_TRANSFER = 15 Tgas`. Init memakai `#[init]` + `PanicOnDefault`.

### 10.3 Model storage listing (NEP-145)

- Berbeda dari NFT contract (bayar per-call), marketplace memakai **pre-deposit**: 1 listing = **0.01 Ⓝ**.

  > **CATATAN PROYEK (ronde 16)** — angka 0.01 Ⓝ adalah **contoh dari tutorial**, BUKAN keputusan NearSea. Nominal storage-per-entry NearSea = **open-by-design** (dihitung dari `storage_usage` saat implementasi — OQ-007); jangan hardcode 0.01 Ⓝ. Mekanisme NEP-145-nya (pre-deposit + `storage_withdraw`) tetap dipakai.
- User setor lumpsum via `storage_deposit` (bisa untuk diri sendiri atau orang lain, minimal `storage_per_sale()`), saldo tercatat di `storage_deposits`.
- Saat listing habis terjual/dihapus, kelebihan bisa ditarik kembali lewat `storage_withdraw` (1 yoctoNEAR) — contoh di docs: setor 10 Ⓝ, list 100 NFT → 9 Ⓝ bisa ditarik, 1 Ⓝ tetap menutup listing aktif.
- `list_nft_for_sale` mengecek: `owner_paid_storage >= storage_per_sale() * (jumlah_listing_saya + 1)`, gagal → panic "Insufficient storage paid".

### 10.4 Alur listing (dual verification)

1. Seller `nft_approve` di NFT contract (otorisasi marketplace) dengan `msg` berisi sale conditions.
2. Seller panggil `list_nft_for_sale { nft_contract_id, token_id, approval_id }` di marketplace (setelah cek storage).
3. Marketplace membuat **dua promise paralel** ke NFT contract:
   - `nft_token(token_id)` → verifikasi kepemilikan (anti-klaim NFT orang lain),
   - `nft_is_approved(token_id, marketplace, approval_id)` → verifikasi approval valid.
4. Callback `process_listing` (`#[private]`, `#[callback_result]` ganda): assert `owner_id` cocok ("Signer is not NFT owner") + approval true ("Marketplace contract is not approved") → simpan `Sale` + isi kedua index.
5. Hook `nft_on_approve` dari NEP-178 juga diimplementasikan (di contoh ini kosong — tempat ekstra logic saat approval).

### 10.5 Alur pembelian (offer → payout royalties)

```
offer(nft_contract_id, token_id)                       // buyer attach deposit ≥ harga
  → cek sale ada, bukan NFT sendiri, deposit ≥ price
  → process_purchase (#[private])
      → hapus sale dulu (optimistic), lalu cross-contract:
        nft_transfer_payout(buyer, token_id, approval_id,
                            msg, price, MAX_PAYOUT=10)  // 1 yoctoNEAR + 15 Tgas
  → resolve_purchase (#[private], callback)
      → validasi Payout → transfer ke tiap penerima
      → gagal? → refund buyer (Promise::new(buyer).transfer(price))
```

> **CATATAN PROYEK (ronde 16)** — nama `offer` di alur tutorial = **method beli**. Di NearSea:
> `buy` (beli listing), `make_offer` (pasang offer + escrow), `accept_offer` (seller terima).
> `msg` tutorial berisi sale conditions; NearSea meneruskan payout langsung sebagai argumen
> (lihat [docs/features/marketplace.md](docs/features/marketplace.md) § Skema Argumen).

`nft_transfer_payout` (extension royalti **NEP-199**) sekaligus **mentransfer NFT dan mengembalikan objek Payout royalti** dari NFT contract. Aturan validasi payout di `resolve_purchase`:

- Promise gagal → refund buyer.
- `payout.len() > 10` atau kosong → tolak (batas gas; `10` adalah max akun royalti).
- Total payout melebihi harga (overflow `checked_sub`) → tolak.
- Sisa pembulatan harus **0 atau 1 yoctoNEAR** — toleransi 1 yocto mengakomodasi royalti berbasis basis-point yang tidak bulat (mis. 3333+3333+3333 bps).
- Valid → `for (receiver_id, amount) in payout { Promise::new(receiver_id).transfer(amount) }` (seller + kreator).

Operasi seller lainnya: `remove_sale` dan `update_price` — keduanya `assert_one_yocto()` + assert pemanggil = `sale.owner_id` (gagal → seluruh transaksi revert).

### 10.6 Pola desain yang layak ditiru

1. **Optimistic removal + revert** — sale dihapus sebelum transfer; jika transfer/payout gagal, state dipulihkan lewat callback dan dana di-refund.
2. **Callback selalu `#[private]`** — `process_purchase`, `resolve_purchase`, `process_listing` hanya bisa dipanggil kontrak sendiri.
3. **Dual cross-contract verification** sebelum menyimpan listing (ownership + approval) — menutup celah "marketplace NFT milik orang lain".
4. **Storage pre-deposit per listing** (NEP-145) dengan refund kelebihan — kontrak tidak menanggung biaya storage user.
5. **1 yoctoNEAR guard** pada semua mutasi seller (remove/update).
6. **Payout dengan batas gas** — maksimum 10 penerima royalti per penjualan, divalidasi ketat sebelum distribusi.
7. **Pembatasan akun royalti dari NFT contract** (`max_len_payout = 10` pada `nft_transfer_payout`).

### 10.7 Demo deployment (dari halaman tutorial)

`cargo near deploy` → init `new` dengan `owner_id` → mint `token-1` (0.1 Ⓝ) → `nft_approve` marketplace → `list_nft_for_sale` dengan `msg` berisi sale conditions (harga; bentuk final mengikuti [docs/features/marketplace.md](docs/features/marketplace.md)) via `near-cli-rs` di testnet. Versi yang dipakai: rustc 1.77.1, near-cli-rs 0.17.0, cargo-near 0.6.1, NEP-171 v1.0.0.

> **CATATAN PROYEK (ronde 16)** — di tutorial harga lewat `msg` hasil `nft_approve`; di NearSea
> **harga = argumen langsung** `list_nft_for_sale(...)` (2-tx, ADR-002). Versi toolchain di atas
> = catatan lingkungan riset; **pin final sudah dikunci di TASK-001** (Rust 1.93.1, near-sdk 5.29.1,
> cargo-near 0.22.0) — lihat [docs/architecture/tech-stack.md](docs/architecture/tech-stack.md) §Version pins.

---

## 11. Peta Fitur OpenSea → Implementasi di NEAR

Target: paritas fitur dengan OpenSea. Prinsip desain utama yang berpihak pada kita:

> **Di NEAR, orderbook bisa 100% on-chain.** Gas NEAR sangat murah (transaksi ~<0.01 Ⓝ), berbeda dari Ethereum yang memaksa OpenSea membangun orderbook off-chain + settlement on-chain. Konsekuensinya: arsitektur kita lebih sederhana, trustless, tanpa infrastruktur tanda tangan off-chain. **Royalti juga dipaksa on-chain saat settlement** (`nft_transfer_payout`) — lebih kuat dari OpenSea yang royaltinya opsional/enforsmen off-chain.

### 11.1 Tabel pemetaan fitur

| # | Fitur OpenSea | Status | Implementasi di NEAR | Lapisan |
|---|---|---|---|---|
| 1 | List for sale / Buy Now (fixed price) | ✅ Pola terbukti | `nft_approve` → `list_nft_for_sale` → `offer` → `nft_transfer_payout` (bagian 10) | On-chain |
| 2 | Cancel listing / ubah harga | ✅ Pola terbukti | `remove_sale` / `update_price` (guard 1 yocto) | On-chain |
| 3 | Royalti kreator | ✅ Lebih kuat | Payout on-chain dipaksa di settlement (bps, maks 10 akun) | On-chain |
| 4 | Offers (bid pada NFT yang tidak di-list) | ➕ Dibangun | State `offers` per (contract.token) → `make_offer` (attach NEAR = escrow), `accept_offer` (cross-call `nft_transfer_payout`), `cancel_offer`, refund saat expire | On-chain |
| 5 | Auction (English, timed, reserve) | ➕ Dibangun | Pola auction contract (mastering-near 3.x): `bid` (escrow, bidder lama langsung refund), track highest bid + deadline, `claim` setelah lewat waktu | On-chain |
| 6 | Multi-currency (bayar pakai token lain) | ➕ Dibangun | NEP-141: buyer `ft_transfer_call` → market `ft_on_transfer` → settle; validasi predecessor = FT whitelist. Referensi: FT tutorial langkah 7, auction 3.2, ShardDog NFT-Minting-withFT | On-chain |
| 7 | Lazy minting (NFT gratis sampai terjual) | ➕ Dibangun | Tutorial `nft-series`: metadata + trait disimpan dulu, `nft_mint` nyata dieksekusi saat first purchase | On-chain |
| 8 | Collections | ✅ Bentuk beda | 1 kontrak per koleksi via **factory** (ala Mintbase `mb-factory-v2` / global contracts) ATAU multi-koleksi dalam 1 kontrak (ShardDog); multiple edition via `copies` + `num_to_mint` | On-chain |
| 9 | Private listing (pembeli tertentu) | ➕ Kecil | Tambah `buyer_id: Option<AccountId>` di Sale, dicek di `offer` | On-chain |
| 10 | Bundle (jual banyak NFT) | ➕ Kecil | Sale berisi `Vec<(contract_id, token_id)>`, approve semua token, transfer batch | On-chain |
| 11 | Activity feed (sales/listing/transfer/offer) | ✅ Infra ada | Event NEP-297 → indexer. Cepat: NearBlocks API; serius: custom indexer (near-indexer framework) → Postgres | Off-chain |
| 12 | Collection stats (floor price, volume, owners, supply) | ➕ Indexer | Agregasi event sales/listing di indexer | Off-chain |
| 13 | Search, filter traits, rarity ranking | ➕ Indexer | Trait di `TokenMetadata.extra` / JSON IPFS; indexer ekstrak → filter/sort + hitung rarity | Off-chain |
| 14 | Profil user (owned / created / activity) | ✅ | NEP-181 `nft_tokens_for_owner` + riwayat dari indexer | Hybrid |
| 15 | Wallet connect | ✅ | `near-connect` (HOT, Meteor, Nightly, WalletConnect, 10+) | Frontend |
| 16 | Gasless listing (onboarding tanpa gas) | ➕ | Meta-transactions (relayer) | Infra |
| 17 | Fiat on-ramp | ✅ Pihak ke-3 | Transak/Ramp via wallet | Eksternal |
| 18 | Cross-chain (beli pakai BTC/ETH) | ➕ Bonus | Chain Signatures + Omnibridge | Infra |

**Batas teknis yang wajib dihormati desain:**
- Media (gambar/video) **selalu IPFS**, bukan on-chain: max upload 4 MB per call, storage 1 Ⓝ/100 KB. Trait/extra → JSON di IPFS, `media_hash` menyimpan sha256.
- Gas maksimum 300 Tgas per call — payout FT + royalti banyak akun harus dipangkas (batas 10 penerima dari tutorial bukan batas protokol, tapi batas gas yang sehat).
- Storage listing pakai pre-deposit NEP-145 + refund agar kontrak tahan terhadap jutaan listing (anti storage-cost attack).

### 11.2 Arsitektur yang diusulkan

```
nft-factory (create_collection)
   └─ collection-<nama>.<kamu>.near        market-v1
      NEP-171/177/178/181/199/297          ├─ Listings  (fixed price)  ← porting market-contract (bag. 10)
      + royalty bps per token/koleksi      ├─ Offers                    ← baru
                                           ├─ Auctions                  ← baru
                                           └─ FT payments (NEP-141)     ← baru

indexer:  events NEP-297 → Postgres → GraphQL/REST API  (floor, volume, traits, activity, rarity)
frontend: Next.js + near-connect + RPC view + indexer API  (discovery, profil, halaman koleksi)
```

Keputusan kunci: **satu kontrak market untuk semua mode trading** (list/offer/auction) — sederhanakan UX dan settlement, dan karena on-chain orderbook murah, tidak perlu memecah kontrak.

### 11.3 Roadmap bertahap menuju paritas OpenSea

> **CATATAN (audit 2026-10-01)**: pembagian fase di bawah ini adalah **draf riset pra-keputusan**. Roadmap FINAL yang mengikat ada di [tasks/implementation-plan.md](tasks/implementation-plan.md) — di sana offers, private listing, bundle, dan launchpad masuk MVP (M1), auction = fase 2, custom indexer pakai Neardata.

- **Fase 1 — MVP (core trading)**: kontrak NFT + royalti (basis `near-sdk-contract-tools`), market fixed-price (porting `market-contract` bagian 10), activity feed via NearBlocks, frontend: browse/list/buy/profile. 
- **Fase 2 — Trading lanjutan**: Offers + escrow, Auction, private listing, multi-currency (FT), notifikasi event.
- **Fase 3 — Ekosistem**: factory + pembuatan koleksi di UI, lazy minting, custom indexer (trait filter, floor/volume/rarity), meta-transactions.
- **Fase 4 — Diferensiasi**: bundle, cross-chain payment (Chain Signatures), fiat on-ramp.

---

## 12. Link Cepat Docs

- **Situs tutorial NFT Zero-to-Hero: https://near-examples.github.io/nft-tutorial/**
- **Repo kode tutorial: https://github.com/near-examples/nft-tutorial**
- Standar NFT: docs.near.org/primitives/nft/standard (NEP-171/177/178)
- Menggunakan NFT: docs.near.org/primitives/nft/nft (deploy, mint, transfer, royalties)
- Buat kontrak NFT: docs.near.org/primitives/nft/sdk-contract-tools
- Zero-to-Hero NFT: docs.near.org/smart-contracts/tutorials/zero-to-hero/nfts
- Smart contract quickstart: docs.near.org/smart-contracts/quickstart
- 1 yoctoNEAR: docs.near.org/smart-contracts/security/one_yocto
- Checklist keamanan: docs.near.org/smart-contracts/security/checklist
- Storage staking: docs.near.org/protocol/storage/storage-staking
- Events (NEP-297): docs.near.org/smart-contracts/anatomy/events
- near-connect: docs.near.org/tools/near-connect
- Wallet login guide: docs.near.org/web3-apps/tutorials/wallet-login
- RPC API: docs.near.org/api/rpc/introduction
- Chain abstraction (cross-chain): docs.near.org/chain-abstraction/what-is
- FT marketplace (multi-currency): near-examples.github.io/ft-tutorial/marketplace + docs.near.org/smart-contracts/tutorials/zero-to-hero/fts (langkah 7)
- Auction + bid pakai FT: docs.near.org/web3-apps/tutorials/mastering-near (3.1-nft, 3.2-ft)

---

## 13. Security Research — Executive Index (pre-implementation)

> Fase security research selesai 2026-10-01: 26 dokumen keamanan + ADR-010..015 + register SEC-*. Indeks eksekutif:

**What we learned**: Non-custodial chain-centric (ADR-010) menghilangkan kelas serangan server terhadap dana; on-chain orderbook (ADR-012) mengeliminasi kelas signature-order attacks; metadata = untrusted penuh; owner key = skenario bencana terburuk (CS-1) yang dibatasi secara struktural (NFT tak pernah dipegang kontrak, MAX_FEE_BPS immutable, escrow user-recoverable).

**Architecture required**: lihat [docs/security/trust-boundaries.md](docs/security/trust-boundaries.md) (10 batas) + [docs/security/smart-contract-security-architecture.md](docs/security/smart-contract-security-architecture.md) (upgrade, access control, pause, fee cap, settlement).

**Threats that matter most**: CS-1 owner key compromise; payout/settlement accounting bug (INV-001..005); kontrak NFT jahat pada platform terbuka (risiko residual DoS/UX — dana terlindungi invariant); frontend supply chain (CS-6); DNS hijack (CS-8).

**Decisions made**: ADR-010..015 (boundary, wallet-auth protocol, order authority, key mgmt interim→multisig, metadata isolation, indexer consistency) + security-requirements.md (40+ requirement ber-ID, status PROPOSED/DECIDED/READY-FOR-IMPLEMENTATION/BLOCKED — tidak ada IMPLEMENTED).

**Decisions open**: G8 pilihan final auditor (kandidat sudah riset; keputusan bisnis); G11 threat-intel & G15 formal verification (fase lanjut); G12 CDN/WAF & G13 contract-account signing (fase lanjut); G14 bug bounty (mainnet). **G7 = DEFAULT diterapkan** (insurance fund 0.1% fee — bisa di-override user). SUDAH TERSELESAIKAN (riset 2026-10-01): G1 permissionless+mitigasi, G2 NEP-413, G3 tooling invariant, G4 Sputnik DAO, G5 pola migrate, G9 DNS, G10 Neardata, G16 NEP-330.

**Assumptions**: A1-A4 (gap-analysis) — user access-key standard, single-box VPS, single owner key (testnet), near-sdk `#[private]` semantics.

**Requires external validation**: audit eksternal market+launchpad sebelum mainnet (M4); restore drill; invariant fuzz.

**Must be completed before coding**: requirement **P0** yang relevan masuk definisi "selesai" setiap task Fase 1. Status per requirement **mengikuti register** [docs/security/security-requirements.md](docs/security/security-requirements.md) — jangan menyimpulkan dari ringkasan ini. Per 2026-10-02: mayoritas P0 berstatus **DECIDED**; yang sudah **READY-FOR-IMPLEMENTATION** = SEC-AUTH-001; beberapa masih **PROPOSED** (mis. SEC-ORDER-003, SEC-CONTRACT-002) dan wajib naik status sebelum task terkait dimulai.
