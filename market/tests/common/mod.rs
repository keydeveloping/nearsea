//! Harness suite sandbox dua-kontrak (koleksi + market) — TASK-006 / tiket 08.
//!
//! Deploy **dua kontrak nyata** (wasm `contract` + `market`) di sandbox chain dan menyediakan
//! helper untuk skenario slice M1. Aturan yang dipegang harness ini:
//!
//! - **Tidak ada mock kontrak.** Semua interaksi lewat wasm nyata (docs/testing/testing-strategy.md
//!   §Strategi mocking: "jangan pernah mem-mock jalur uang").
//! - **Saldo dibaca setelah rantai "tenang"** (`Slice::quiesce`), bukan langsung setelah tx.
//!   Query `near-workspaces` default memakai `Finality::Optimistic`; membacanya tepat setelah
//!   transaksi bisa melihat state antara (sebagian transfer belum masuk, sebagian refund deposit
//!   belum selesai) sehingga assert distribusi jadi salah. Maju 2 blok dulu = state quiescent.
//! - Wasm dibaca dari artefak `cargo near build` (`target/near/<crate>/<crate>.wasm`) — jalur yang
//!   sama dipakai gate CI (docs/development/ci-cd.md).

use near_sdk::AccountId;
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, Contract, Worker};
use serde_json::{json, Value};
/// Harga listing default test = 1 Ⓝ (docs/testing/testing-strategy.md §Strategi fixture:
/// nilai uang selalu string yoctoNEAR).
pub const PRICE: u128 = 1_000_000_000_000_000_000_000_000;
/// Harga minimum listing (INV-030) = 0.01 Ⓝ.
pub const MIN_PRICE: u128 = 10_000_000_000_000_000_000_000;
/// Royalti koleksi test = 5% (di bawah cap 10% INV-027).
pub const ROYALTY_BPS: u16 = 500;
/// Fee platform default = 2% (ADR-005).
pub const FEE_BPS: u16 = 200;
/// `floor(PRICE × FEE_BPS / 10_000)`.
pub const FEE: u128 = PRICE * FEE_BPS as u128 / 10_000;
/// `floor(PRICE × ROYALTY_BPS / 10_000)` — payout yang dikembalikan koleksi.
pub const ROYALTY: u128 = PRICE * ROYALTY_BPS as u128 / 10_000;
/// Residual ke seller = `harga − fee − royalti` (INV-002, bukan dust).
pub const SELLER_PROCEEDS: u128 = PRICE - FEE - ROYALTY;
/// Satu hari dalam nanodetik (window fase).
pub const DAY_NS: u64 = 86_400_000_000_000;

/// Saldo storage NEP-145 yang dikreditkan ke tiap akun test di koleksi/market.
pub const STORAGE_TOPUP: u128 = 500_000_000_000_000_000_000_000;

/// Prefix `LookupMap` dari `StorageKey::Sales` (varian pertama → diskriminan borsh 0) dan
/// `StorageKey::PendingPurchases` (varian kedua → 1) — layout kanonik market.md §7.
pub const PREFIX_PENDING_PURCHASES: u8 = 1;

fn wasm_bytes(relative: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    std::fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "wasm tidak ditemukan di {}: {error}. \
             Jalankan `cargo near build non-reproducible-wasm --no-abi --manifest-path <crate>/Cargo.toml` \
             untuk contract/ dan market/ lebih dulu.",
            path.display()
        )
    })
}

/// Wasm koleksi **pihak ketiga** (fixture nakal) untuk TC-003 — dibangun terpisah karena
/// crate-nya sengaja di luar workspace produksi.
fn rogue_collection_wasm() -> Vec<u8> {
    wasm_bytes(
        "../target/near/nearsea_rogue_collection_fixture/nearsea_rogue_collection_fixture.wasm",
    )
}

/// Sandbox dibagi antar test dalam satu binary: start sandbox ≈ 20 detik, dan menjalankannya per
/// test akan melewati anggaran `contract-test` (≤8 menit, docs/development/ci-cd.md).
/// Kontrak koleksi+market tetap **deploy baru per test**, jadi tiap test membangun state sendiri
/// (docs/testing/testing-strategy.md §Kebijakan flaky test: isolasi).
static WORKER: tokio::sync::OnceCell<Worker<Sandbox>> = tokio::sync::OnceCell::const_new();
/// Sandbox berbagi + satu penulis root account → test dijalankan berurutan.
pub static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn shared_worker() -> anyhow::Result<Worker<Sandbox>> {
    let worker = WORKER
        .get_or_try_init(|| async {
            near_workspaces::sandbox()
                .await
                .map_err(|error| anyhow::anyhow!("sandbox gagal start: {error}"))
        })
        .await?;
    Ok(worker.clone())
}

/// Fixture satu sandbox: dua kontrak + akun test.
///
/// `creator` menerima royalti, `treasury` menerima fee, `seller` pihak penjual.
/// Owner market = root account sandbox supaya aksi owner-only (TC-047) bisa diuji tanpa
/// membuat akun owner terpisah.
pub struct Slice {
    pub worker: Worker<Sandbox>,
    pub owner: Account,
    pub collection: Contract,
    pub market: Contract,
    pub creator: Account,
    pub seller: Account,
    pub treasury: Account,
}

impl Slice {
    pub async fn new() -> anyhow::Result<Self> {
        Self::with_fee_bps(FEE_BPS).await
    }

    pub async fn with_fee_bps(fee_bps: u16) -> anyhow::Result<Self> {
        Self::build(fee_bps, ROYALTY_BPS).await
    }

    /// Fixture dengan fee **dan** royalti non-default (mis. keduanya di cap) — INV-004/INV-027.
    pub async fn with_caps(fee_bps: u16, royalty_bps: u16) -> anyhow::Result<Self> {
        Self::build(fee_bps, royalty_bps).await
    }

    async fn build(fee_bps: u16, royalty_bps: u16) -> anyhow::Result<Self> {
        let worker = shared_worker().await?;
        let owner = worker.root_account()?;

        let collection = worker
            .dev_deploy(&wasm_bytes(
                "../target/near/nearsea_nft_collection/nearsea_nft_collection.wasm",
            ))
            .await?;
        let market = worker
            .dev_deploy(&wasm_bytes(
                "../target/near/nearsea_market/nearsea_market.wasm",
            ))
            .await?;

        let creator = worker.dev_create_account().await?;
        let seller = worker.dev_create_account().await?;
        let treasury = worker.dev_create_account().await?;

        collection
            .call("new")
            .args_json(json!({
                "owner_id": owner.id(),
                "creator_id": creator.id(),
                "royalty_bps": royalty_bps,
                "name": "Genesis",
                "symbol": "GNS",
                "base_uri": null,
                "market_id": market.id(),
            }))
            .transact()
            .await?
            .into_result()?;

        market
            .call("new")
            .args_json(json!({
                "owner_id": owner.id(),
                "fee_bps": fee_bps,
                "treasury": treasury.id(),
            }))
            .transact()
            .await?
            .into_result()?;

        let slice = Self {
            worker,
            owner,
            collection,
            market,
            creator,
            seller,
            treasury,
        };

        for account in [&slice.owner, &slice.creator, &slice.seller, &slice.treasury] {
            slice.register_collection(account).await?;
            slice.register_market(account).await?;
        }

        Ok(slice)
    }

    /// Daftar saldo storage NEP-145 di koleksi (INV-019: storage mint dibayar invoker).
    pub async fn register_collection(&self, account: &Account) -> anyhow::Result<()> {
        account
            .call(self.collection.id(), "storage_deposit")
            .args_json(json!({}))
            .deposit(NearToken::from_yoctonear(STORAGE_TOPUP))
            .transact()
            .await?
            .into_result()?;
        Ok(())
    }

    /// Daftar saldo storage NEP-145 di market (INV-020: user bayar storage listing).
    pub async fn register_market(&self, account: &Account) -> anyhow::Result<()> {
        account
            .call(self.market.id(), "storage_deposit")
            .args_json(json!({}))
            .deposit(NearToken::from_yoctonear(STORAGE_TOPUP))
            .transact()
            .await?
            .into_result()?;
        Ok(())
    }

    /// Akun pembeli baru, sudah terdaftar di storage koleksi (syarat `nft_transfer`).
    pub async fn new_buyer(&self) -> anyhow::Result<Account> {
        let buyer = self.worker.dev_create_account().await?;
        self.register_collection(&buyer).await?;
        Ok(buyer)
    }

    /// Buka satu fase publik (`set_phases`, owner-only) yang mulai sekarang.
    pub async fn open_public_phase(
        &self,
        allocation: u32,
        max_per_wallet: u32,
    ) -> anyhow::Result<()> {
        self.open_public_phase_priced(allocation, max_per_wallet, PRICE)
            .await
    }

    pub async fn open_public_phase_priced(
        &self,
        allocation: u32,
        max_per_wallet: u32,
        price: u128,
    ) -> anyhow::Result<()> {
        let now = self.worker.view_block().await?.timestamp();
        self.owner
            .call(self.collection.id(), "set_phases")
            .args_json(json!({ "phases": [{
                "name": "Public",
                "price_yocto": price.to_string(),
                "allocation": allocation,
                "max_per_wallet": max_per_wallet,
                "allowlist_required": false,
                "starts_at": now,
                "ends_at": now + DAY_NS,
            }]}))
            .deposit(NearToken::from_yoctonear(STORAGE_TOPUP))
            .max_gas()
            .transact()
            .await?
            .into_result()?;
        Ok(())
    }

    /// Mint `quantity` token ke `minter`; deposit exact = harga fase (INV-018).
    pub async fn mint(&self, minter: &Account, quantity: u32) -> anyhow::Result<Vec<String>> {
        let result = minter
            .call(self.collection.id(), "nft_mint")
            .args_json(json!({ "phase_index": null, "quantity": quantity }))
            .deposit(NearToken::from_yoctonear(PRICE * u128::from(quantity)))
            .max_gas()
            .transact()
            .await?;
        result.clone().into_result()?;

        Ok(event_payloads(&result, "launchpad_mint")
            .into_iter()
            .flat_map(|payload| {
                payload["token_ids"]
                    .as_array()
                    .map(|ids| {
                        ids.iter()
                            .filter_map(|id| id.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            })
            .collect())
    }

    /// Mint satu token dan kembalikan id-nya.
    pub async fn mint_one(&self, minter: &Account) -> anyhow::Result<String> {
        let ids = self.mint(minter, 1).await?;
        Ok(ids
            .first()
            .cloned()
            .expect("launchpad_mint memuat token id"))
    }

    /// Tx-1 listing: `nft_approve(market)`; kembalikan `approval_id` yang diberikan koleksi.
    ///
    /// `approval_id` tidak bisa di-query langsung (tidak ada method publiknya), jadi dibaca dari
    /// `nft_token(token_id).approved_account_ids[market]` — nilai yang sama dipakai market saat
    /// `nft_transfer_payout` (INV-011).
    pub async fn approve_market(&self, owner: &Account, token_id: &str) -> anyhow::Result<u64> {
        owner
            .call(self.collection.id(), "nft_approve")
            .args_json(json!({
                "token_id": token_id,
                "account_id": self.market.id(),
                "msg": null,
            }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        self.approval_id_of(token_id).await?.ok_or_else(|| {
            anyhow::anyhow!("market tidak tercatat sebagai approved untuk token {token_id}")
        })
    }

    /// Tx-2 listing: `list_nft_for_sale` dengan deposit storage penuh.
    pub async fn list(
        &self,
        seller: &Account,
        token_id: &str,
        price: u128,
        allowed_buyer: Option<&AccountId>,
    ) -> anyhow::Result<ExecutionFinalResult> {
        self.list_with_deposit(seller, token_id, price, allowed_buyer, STORAGE_TOPUP)
            .await
    }

    /// `list_nft_for_sale` dengan deposit storage yang bisa diatur (TC-020).
    pub async fn list_with_deposit(
        &self,
        seller: &Account,
        token_id: &str,
        price: u128,
        allowed_buyer: Option<&AccountId>,
        deposit: u128,
    ) -> anyhow::Result<ExecutionFinalResult> {
        let approval_id = self.approval_id_of(token_id).await?;
        Ok(seller
            .call(self.market.id(), "list_nft_for_sale")
            .args_json(json!({
                "nft_contract_id": self.collection.id(),
                "token_id": token_id,
                "approval_id": approval_id,
                "price": price.to_string(),
                "allowed_buyer": allowed_buyer,
            }))
            .deposit(NearToken::from_yoctonear(deposit))
            .max_gas()
            .transact()
            .await?)
    }

    /// Mint + approve + list satu token (jalur bahagia).
    pub async fn mint_and_list(&self, seller: &Account, price: u128) -> anyhow::Result<String> {
        let token_id = self.mint_one(seller).await?;
        self.approve_market(seller, &token_id).await?;
        self.list(seller, &token_id, price, None)
            .await?
            .into_result()?;
        Ok(token_id)
    }

    /// `buy` dengan deposit `deposit`.
    pub async fn buy(
        &self,
        buyer: &Account,
        token_id: &str,
        deposit: u128,
    ) -> anyhow::Result<ExecutionFinalResult> {
        Ok(buyer
            .call(self.market.id(), "buy")
            .args_json(json!({
                "nft_contract_id": self.collection.id(),
                "token_id": token_id,
            }))
            .deposit(NearToken::from_yoctonear(deposit))
            .max_gas()
            .transact()
            .await?)
    }

    /// `approval_id` yang dipegang market untuk satu token (dari `nft_token`).
    pub async fn approval_id_of(&self, token_id: &str) -> anyhow::Result<Option<u64>> {
        let token: Option<Value> = self
            .owner
            .view(self.collection.id(), "nft_token")
            .args_json(json!({ "token_id": token_id }))
            .await?
            .json()?;
        Ok(token
            .and_then(|token| {
                token["approved_account_ids"]
                    .get(self.market.id().as_str())
                    .cloned()
            })
            .and_then(|id| id.as_u64()))
    }

    /// Pemilik token saat ini (`None` = token tidak ada). Dasar bukti non-custodial (ADR-007).
    pub async fn token_owner(&self, token_id: &str) -> anyhow::Result<Option<AccountId>> {
        let token: Option<Value> = self
            .owner
            .view(self.collection.id(), "nft_token")
            .args_json(json!({ "token_id": token_id }))
            .await?
            .json()?;
        Ok(token.and_then(|token| {
            token["owner_id"]
                .as_str()
                .map(|id| id.parse().expect("account id"))
        }))
    }

    /// Entry listing aktif (`None` = tidak ada / sudah terjual — optimistic removal).
    pub async fn sale(&self, token_id: &str) -> anyhow::Result<Option<Value>> {
        let sale: Option<Value> = self
            .owner
            .view(self.market.id(), "get_sale")
            .args_json(json!({
                "nft_contract_id": self.collection.id(),
                "token_id": token_id,
            }))
            .await?
            .json()?;
        Ok(sale)
    }

    /// Entry pembelian berjalan (INV-031) — dipakai TC-054.
    pub async fn pending_purchase(&self, token_id: &str) -> anyhow::Result<Option<Value>> {
        let pending: Option<Value> = self
            .owner
            .view(self.market.id(), "get_pending_purchase")
            .args_json(json!({
                "nft_contract_id": self.collection.id(),
                "token_id": token_id,
            }))
            .await?
            .json()?;
        Ok(pending)
    }

    /// Saldo storage yang masih **bisa dipakai** (`available`) di market — dipakai INV-020.
    ///
    /// `total` adalah jumlah yang pernah disetor (bertambah tiap deposit terlampir), jadi yang
    /// mencerminkan storage terpakai/terbebas adalah `available`.
    pub async fn market_storage_available(&self, account: &AccountId) -> anyhow::Result<u128> {
        self.storage_available(self.market.id(), account).await
    }

    /// Saldo storage `available` di koleksi — dipakai INV-019.
    pub async fn collection_storage_available(&self, account: &AccountId) -> anyhow::Result<u128> {
        self.storage_available(self.collection.id(), account).await
    }

    async fn storage_available(
        &self,
        contract: &AccountId,
        account: &AccountId,
    ) -> anyhow::Result<u128> {
        let balance: Option<Value> = self
            .owner
            .view(contract, "storage_balance_of")
            .args_json(json!({ "account_id": account }))
            .await?
            .json()?;
        Ok(balance
            .and_then(|balance| balance["available"].as_str().and_then(|v| v.parse().ok()))
            .unwrap_or(0))
    }

    /// Storage minimum satu entry `Sale` (bounds NEP-145 market) — basis TC-020.
    pub async fn market_storage_minimum(&self) -> anyhow::Result<u128> {
        let bounds: Value = self
            .owner
            .view(self.market.id(), "storage_balance_bounds")
            .await?
            .json()?;
        Ok(bounds["min"]
            .as_str()
            .and_then(|value| value.parse().ok())
            .expect("storage_balance_bounds.min"))
    }

    /// Majukan rantai supaya seluruh receipt transaksi sebelumnya selesai, lalu state berhenti
    /// berubah. **Wajib** sebelum membaca saldo/state untuk assert (lihat catatan modul).
    ///
    /// Aman untuk TC-054: jeda `RECOVERY_DELAY_BLOCKS` (20) diukur dari `created_height`, jadi
    /// memajukan rantai justru yang membuat pemulihan boleh dijalankan.
    pub async fn quiesce(&self) -> anyhow::Result<()> {
        self.worker.fast_forward(2).await?;
        Ok(())
    }

    /// Saldo NEAR akun. Baca setelah [`Slice::quiesce`] agar angkanya final.
    pub async fn balance_of(&self, account: &AccountId) -> anyhow::Result<u128> {
        Ok(self
            .worker
            .view_account(account)
            .await?
            .balance
            .as_yoctonear())
    }

    /// Saldo empat pihak (seller, creator, treasury, buyer) setelah rantai ditenangkan.
    pub async fn balances(&self, buyer: &Account) -> anyhow::Result<(u128, u128, u128, u128)> {
        self.quiesce().await?;
        Ok((
            self.balance_of(self.seller.id()).await?,
            self.balance_of(self.creator.id()).await?,
            self.balance_of(self.treasury.id()).await?,
            self.balance_of(buyer.id()).await?,
        ))
    }
}

/// Fixture TC-003: market nyata + **koleksi pihak ketiga yang nakal**.
///
/// Market memvalidasi payout dari koleksi sebagai UNTRUSTED (docs/features/payments.md
/// §Algoritma Distribusi Payout), jadi koleksi yang mengembalikan payout tidak valid harus
/// berakhir refund penuh — bukan distribusi yang salah.
pub struct RogueSlice {
    pub slice: Slice,
    pub rogue: Contract,
    pub royalty_receiver: Account,
}

impl RogueSlice {
    /// Deploy market + fixture nakal; `mode` menentukan bentuk payout yang dikembalikan
    /// (`Valid` | `Empty` | `ZeroAmount` | `OverReceiverCap` | `OverPrice`).
    pub async fn new(mode: &str) -> anyhow::Result<Self> {
        let slice = Slice::new().await?;
        let rogue = slice.worker.dev_deploy(&rogue_collection_wasm()).await?;
        let royalty_receiver = slice.worker.dev_create_account().await?;

        rogue
            .call("new")
            .args_json(json!({
                "owner_id": slice.owner.id(),
                "token_owner": slice.seller.id(),
                "royalty_receiver": royalty_receiver.id(),
            }))
            .transact()
            .await?
            .into_result()?;

        // `set_mode` owner-only — `dev_deploy` menandatangani dengan akun kontrak, jadi
        // panggilan harus lewat root account yang memang di-set sebagai owner.
        slice
            .owner
            .call(rogue.id(), "set_mode")
            .args_json(json!({ "mode": mode }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(Self {
            slice,
            rogue,
            royalty_receiver,
        })
    }

    /// Listing satu token pada koleksi nakal (approval fixture selalu `true`, jadi cukup
    /// `list_nft_for_sale` dengan deposit storage).
    pub async fn list(&self, price: u128) -> anyhow::Result<()> {
        self.slice
            .seller
            .call(self.slice.market.id(), "list_nft_for_sale")
            .args_json(json!({
                "nft_contract_id": self.rogue.id(),
                "token_id": "0",
                "approval_id": 0,
                "price": price.to_string(),
                "allowed_buyer": null,
            }))
            .deposit(NearToken::from_yoctonear(STORAGE_TOPUP))
            .max_gas()
            .transact()
            .await?
            .into_result()?;
        Ok(())
    }

    pub async fn buy(
        &self,
        buyer: &Account,
        deposit: u128,
    ) -> anyhow::Result<ExecutionFinalResult> {
        Ok(buyer
            .call(self.slice.market.id(), "buy")
            .args_json(json!({
                "nft_contract_id": self.rogue.id(),
                "token_id": "0",
            }))
            .deposit(NearToken::from_yoctonear(deposit))
            .max_gas()
            .transact()
            .await?)
    }

    pub async fn sale(&self) -> anyhow::Result<Option<Value>> {
        let sale: Option<Value> = self
            .slice
            .owner
            .view(self.slice.market.id(), "get_sale")
            .args_json(json!({
                "nft_contract_id": self.rogue.id(),
                "token_id": "0",
            }))
            .await?
            .json()?;
        Ok(sale)
    }

    /// Berapa kali fixture dipanggil `nft_transfer_payout` (bukti market sampai ke NEP-199).
    pub async fn payout_calls(&self) -> anyhow::Result<u32> {
        Ok(self
            .slice
            .owner
            .view(self.rogue.id(), "payout_calls")
            .await?
            .json()?)
    }

    /// Saldo buyer + royalty receiver + treasury setelah rantai ditenangkan.
    pub async fn balances(&self, buyer: &Account) -> anyhow::Result<(u128, u128, u128)> {
        self.slice.quiesce().await?;
        Ok((
            self.slice.balance_of(buyer.id()).await?,
            self.slice.balance_of(self.royalty_receiver.id()).await?,
            self.slice.balance_of(self.slice.treasury.id()).await?,
        ))
    }
}

/// Selisih saldo empat pihak antara dua pembacaan.
#[derive(Debug, Clone, Copy)]
pub struct BalanceDelta {
    pub seller: i128,
    pub creator: i128,
    pub treasury: i128,
    pub buyer: i128,
}

impl BalanceDelta {
    pub fn between(before: (u128, u128, u128, u128), after: (u128, u128, u128, u128)) -> Self {
        let d = |a: u128, b: u128| a as i128 - b as i128;
        Self {
            seller: d(after.0, before.0),
            creator: d(after.1, before.1),
            treasury: d(after.2, before.2),
            buyer: d(after.3, before.3),
        }
    }
}

/// Semua payload `data[0]` untuk satu nama event NearSea di dalam log transaksi.
pub fn event_payloads(result: &ExecutionFinalResult, event: &str) -> Vec<Value> {
    result
        .logs()
        .iter()
        .filter_map(|log| log.strip_prefix("EVENT_JSON:"))
        .filter_map(|json| serde_json::from_str::<Value>(json).ok())
        .filter(|payload| payload["event"].as_str() == Some(event))
        .filter_map(|payload| {
            payload["data"]
                .as_array()
                .and_then(|data| data.first().cloned())
        })
        .collect()
}

/// Satu payload event; panic bila tidak ada (memudahkan pesan gagal).
pub fn event_payload(result: &ExecutionFinalResult, event: &str) -> Value {
    event_payloads(result, event)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("event {event} tidak ter-emit; log: {:?}", result.logs()))
}

/// Panic message dari receipt yang gagal (dipakai untuk assert kode registry).
pub fn failure_message(result: &ExecutionFinalResult) -> String {
    format!("{:?}", result.failures())
}

/// Jumlah penerima unik di payout event `market_sale`.
pub fn payout_receivers(sale_event: &Value) -> Vec<String> {
    sale_event["payout"]
        .as_object()
        .map(|payout| payout.keys().cloned().collect())
        .unwrap_or_default()
}
