//! NearSea Market — listing 2-tx (dual verification) + storage NEP-145.
//!
//! Reference implementasi: docs/contracts/market.md §1–§2, §5, §7.
//! Tiket ini (TASK-004) hanya jalur listing: `list_nft_for_sale` + callback `process_listing`,
//! `remove_sale`, `update_price`, dan view listing. Settlement (`buy`/`resolve_purchase`),
//! offer/bundle, serta konfigurasi fee/treasury milik TASK-005/009/010.
//!
//! Prefix storage kanonik: docs/contracts/market.md §7 (SEC-CONTRACT-008).

use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::{
    assert_one_yocto, env, near, AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault,
    Promise, PromiseError,
};
use near_sdk_contract_tools::nft::{
    ext_nep171, ext_nep178, Nep145Controller, StorageBalanceBounds, Token,
};
use near_sdk_contract_tools::owner::Owner;
use near_sdk_contract_tools::pause::Pause;
use near_sdk_contract_tools::standard::nep297::{Event, EventLog, ToEventLog};
use near_sdk_contract_tools::{Nep145, Owner as OwnerDerive, Pause as PauseDerive};
use std::borrow::Cow;

/// Fee platform default 2% — nilai fee ≠ cap (INV-004, ADR-005). Dipakai settlement TASK-005.
pub const FEE_BPS_DEFAULT: u16 = 200;
/// Cap immutable fee — ditegakkan juga di `update_fee_bps` (SEC-CONTRACT-004).
pub const MAX_FEE_BPS: u16 = 500;

// Fee default tidak boleh melewati cap (INV-004) — ditegakkan saat kompilasi.
const _: () = assert!(FEE_BPS_DEFAULT <= MAX_FEE_BPS);

/// Harga minimum listing 0.01 Ⓝ (INV-030) — batas inklusif.
pub const MIN_PRICE_YOCTO: u128 = 10_000_000_000_000_000_000_000;

/// Storage yang wajib tersedia di saldo NEP-145 seller sebelum listing (INV-020).
/// PROPOSED — plafon aman untuk entry `Sale`; nominal final diukur di sandbox (OQ-007).
pub const STORAGE_PER_SALE_BYTES: u64 = 500;

/// Gas satu view call dual verification (`nft_token` / `nft_is_approved`) — PROPOSED
/// (usulan 3–5 Tgas; docs/contracts/market.md §6).
pub const GAS_FOR_NFT_VIEW: Gas = Gas::from_tgas(5);
/// Gas callback `process_listing` (tulis entry `Sale` + event) — PROPOSED (usulan ~5–10 Tgas).
pub const GAS_FOR_PROCESS_LISTING: Gas = Gas::from_tgas(10);
/// Total anggaran dual verification = 2 view + callback — PROPOSED (usulan 15–20 Tgas,
/// docs/contracts/market.md §6). Diturunkan dari bagian-bagiannya supaya tidak ada angka kedua
/// yang bisa menyimpang; dikonfirmasi saat pengukuran sandbox TASK-006.
pub const GAS_FOR_DUAL_VERIFY: Gas =
    Gas::from_gas(GAS_FOR_NFT_VIEW.as_gas() * 2 + GAS_FOR_PROCESS_LISTING.as_gas());

/// Teks panic kontrak = kode registry, supaya FE memetakan ke kode user yang stabil
/// (docs/development/error-handling.md §3/§4).
mod err {
    pub const CHAIN_REVERT: &str = "CHAIN_REVERT";
    pub const CHAIN_PAUSED: &str = "CHAIN_PAUSED";
    pub const INVALID_PRICE: &str = "INVALID_PRICE";
    pub const CONFLICT_ALREADY_LISTED: &str = "CONFLICT_ALREADY_LISTED";
}

/// Prefix storage milik kontrak ini — bukan `~*` (milik derive near-sdk-contract-tools).
#[derive(BorshStorageKey, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    Sales,
}

/// Kunci listing — satu token tidak bisa punya dua listing aktif (INV-007).
pub type SaleKey = (AccountId, String);

/// Listing aktif (docs/contracts/market.md §5). Entry ada ⇔ ACTIVE
/// (optimistic removal saat settle/delist — INV-009). `allowed_buyer ≠ null` = private listing.
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sale {
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub owner_id: AccountId,
    pub approval_id: Option<u64>,
    pub price_yocto: U128,
    pub allowed_buyer: Option<AccountId>,
    pub listed_at: u64,
}

/// Envelope event NearSea — `data` selalu array satu entri (docs/api/webhooks.md §Skema payload).
/// Dipakai semua event custom `x-nearsea-market`; bentuknya disamakan dengan kontrak koleksi.
pub struct SingleEvent<T> {
    standard: &'static str,
    version: &'static str,
    event: &'static str,
    data: Vec<T>,
}

impl<T> SingleEvent<T> {
    pub fn new(event: &'static str, payload: T) -> Self {
        Self {
            standard: "x-nearsea-market",
            version: "1.0.0",
            event,
            data: vec![payload],
        }
    }
}

impl<T: Serialize> ToEventLog for SingleEvent<T> {
    type Data = Vec<T>;

    fn to_event_log(&self) -> EventLog<'_, &Self::Data> {
        EventLog {
            standard: Cow::Borrowed(self.standard),
            version: Cow::Borrowed(self.version),
            event: Cow::Borrowed(self.event),
            data: &self.data,
        }
    }
}

/// Isi `data[0]` event `market_list` (docs/api/webhooks.md).
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketList {
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub seller: AccountId,
    pub price_yocto: U128,
    pub approval_id: Option<u64>,
    pub allowed_buyer: Option<AccountId>,
}

/// Isi `data[0]` event `market_delist`.
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketDelist {
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub seller: AccountId,
}

/// Isi `data[0]` event `market_update_price`.
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketUpdatePrice {
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub seller: AccountId,
    pub old_price_yocto: U128,
    pub new_price_yocto: U128,
}

/// Storage minimum satu entry `Sale` (byte × `storage_byte_cost`) — INV-020.
fn storage_per_sale() -> NearToken {
    env::storage_byte_cost().saturating_mul(u128::from(STORAGE_PER_SALE_BYTES))
}

#[near(contract_state)]
#[derive(PanicOnDefault, OwnerDerive, PauseDerive, Nep145)]
pub struct Market {
    sales: UnorderedMap<SaleKey, Sale>,
}

#[near]
impl Market {
    /// Init sekali (PanicOnDefault) — SEC-CONTRACT-002 (TC-001).
    /// `fee_bps`/`treasury` (docs/contracts/market.md §1) menyusul di TASK-005 karena hanya
    /// dipakai settlement; tiket listing ini tidak memerlukannya.
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        let mut contract = Self {
            sales: UnorderedMap::new(StorageKey::Sales),
        };
        Owner::init(&mut contract, &owner_id);
        Nep145Controller::set_storage_balance_bounds(
            &mut contract,
            &StorageBalanceBounds {
                min: storage_per_sale(),
                max: None,
            },
        );
        contract
    }

    /// Listing 2-tx: seller sudah `nft_approve(market)` di koleksi (tx-1), lalu memanggil ini (tx-2).
    /// Kontrak **tidak** percaya klaim pemanggil — kepemilikan & approval diverifikasi sendiri lewat
    /// dua view call ke koleksi (`process_listing`) — ADR-002 (mekanisme yang sama dipakai lagi saat
    /// settle, SEC-ORDER-004, di TASK-005).
    ///
    /// Deposit = **storage NEP-145** (bukan 1 yocto): entry `Sale` tumbuh → dibayar seller (INV-020).
    /// NFT tidak pernah masuk kontrak ini — market hanya memegang approval (non-custodial, ADR-007).
    #[payable]
    pub fn list_nft_for_sale(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        price: U128,
        allowed_buyer: Option<AccountId>,
    ) -> Promise {
        Self::assert_not_paused();

        if price.0 < MIN_PRICE_YOCTO {
            env::panic_str(err::INVALID_PRICE);
        }

        let seller = env::predecessor_account_id();
        let key = (nft_contract_id.clone(), token_id.clone());
        if self.sales.get(&key).is_some() {
            env::panic_str(err::CONFLICT_ALREADY_LISTED);
        }

        // `approval_id` ABI `u64` → `u32` internal NEP-178 (pola nft_transfer_payout). Id di luar
        // rentang itu tidak mungkin pernah diterbitkan koleksi → tolak, jangan cek approval "tanpa id".
        let approval_u32 = approval_id
            .map(|id| u32::try_from(id).unwrap_or_else(|_| env::panic_str(err::CHAIN_REVERT)));

        // Deposit terlampir masuk saldo storage seller, lalu saldo wajib cukup untuk entry `Sale`.
        // Dicek di sini (sinkron) supaya deposit kurang = revert tx, bukan callback gagal senyap.
        self.credit_attached_storage(&seller);
        self.assert_storage_for_listing(&seller);

        let token_promise = ext_nep171::ext(nft_contract_id.clone())
            .with_static_gas(GAS_FOR_NFT_VIEW)
            .nft_token(token_id.clone());
        let approved_promise = ext_nep178::ext(nft_contract_id.clone())
            .with_static_gas(GAS_FOR_NFT_VIEW)
            .nft_is_approved(token_id.clone(), env::current_account_id(), approval_u32);

        token_promise.and(approved_promise).then(
            Self::ext_self()
                .with_static_gas(GAS_FOR_PROCESS_LISTING)
                .process_listing(
                    seller,
                    nft_contract_id,
                    token_id,
                    approval_id,
                    price,
                    allowed_buyer,
                ),
        )
    }

    /// Hasil dual verification (INV-013 — `#[private]`, hanya kontrak sendiri).
    /// `seller_id` = pemanggil asli `list_nft_for_sale`; predecessor di sini = market.
    #[private]
    #[allow(clippy::too_many_arguments)]
    pub fn process_listing(
        &mut self,
        #[callback_result] token: Result<Option<Token>, PromiseError>,
        #[callback_result] approved: Result<bool, PromiseError>,
        seller_id: AccountId,
        nft_contract_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        price: U128,
        allowed_buyer: Option<AccountId>,
    ) {
        let token = match token {
            Ok(Some(token)) => token,
            _ => env::panic_str(err::CHAIN_REVERT),
        };

        if token.owner_id != seller_id || !matches!(approved, Ok(true)) {
            env::panic_str(err::CHAIN_REVERT);
        }

        let key = (nft_contract_id.clone(), token_id.clone());
        if self.sales.get(&key).is_some() {
            env::panic_str(err::CONFLICT_ALREADY_LISTED);
        }

        let storage_usage_start = env::storage_usage();
        self.sales.insert(
            &key,
            &Sale {
                nft_contract_id: nft_contract_id.clone(),
                token_id: token_id.clone(),
                owner_id: seller_id.clone(),
                approval_id,
                price_yocto: price,
                allowed_buyer: allowed_buyer.clone(),
                listed_at: env::block_timestamp(),
            },
        );
        self.charge_storage(&seller_id, storage_usage_start);

        SingleEvent::new(
            "market_list",
            MarketList {
                nft_contract_id,
                token_id,
                seller: seller_id,
                price_yocto: price,
                approval_id,
                allowed_buyer,
            },
        )
        .emit();
    }

    /// Batalkan listing. Hanya `sale.owner_id`, wajib tepat 1 yocto, dan **tetap boleh saat paused**
    /// (jalur pengembalian aset — INV-022). NFT tidak pernah berpindah wallet (non-custodial).
    ///
    /// Market **tidak** mencabut approval ke koleksi: NEP-178 hanya mengizinkan pemilik token
    /// memanggil `nft_revoke`/`nft_revoke_all` (tidak ada varian untuk approved account). Approval
    /// yang tersisa tidak berbahaya — tanpa entry `Sale` market tidak punya alasan memindahkan token,
    /// dan transfer berikutnya oleh owner otomatis mencabut semua approval (NEP-178). Lihat
    /// docs/contracts/market.md §2 catatan `remove_sale`.
    #[payable]
    pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: String) {
        assert_one_yocto();

        let key = (nft_contract_id.clone(), token_id.clone());
        let sale = self
            .sales
            .get(&key)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
        let seller = env::predecessor_account_id();
        if sale.owner_id != seller {
            env::panic_str(err::CHAIN_REVERT);
        }

        let storage_usage_start = env::storage_usage();
        self.sales.remove(&key);
        // Storage yang dibebaskan kembali ke saldo seller — ditarik lewat `storage_withdraw`.
        self.charge_storage(&seller, storage_usage_start);

        SingleEvent::new(
            "market_delist",
            MarketDelist {
                nft_contract_id,
                token_id,
                seller,
            },
        )
        .emit();
    }

    /// Ubah harga in-place. `approval_id` tidak berubah — hanya state harga milik market.
    #[payable]
    pub fn update_price(&mut self, nft_contract_id: AccountId, token_id: String, new_price: U128) {
        assert_one_yocto();
        Self::assert_not_paused();

        if new_price.0 < MIN_PRICE_YOCTO {
            env::panic_str(err::INVALID_PRICE);
        }

        let key = (nft_contract_id.clone(), token_id.clone());
        let mut sale = self
            .sales
            .get(&key)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
        let seller = env::predecessor_account_id();
        if sale.owner_id != seller {
            env::panic_str(err::CHAIN_REVERT);
        }

        let old_price = sale.price_yocto;
        sale.price_yocto = new_price;
        self.sales.insert(&key, &sale);

        SingleEvent::new(
            "market_update_price",
            MarketUpdatePrice {
                nft_contract_id,
                token_id,
                seller,
                old_price_yocto: old_price,
                new_price_yocto: new_price,
            },
        )
        .emit();
    }

    /// Listing aktif satu token (`None` = tidak ada listing).
    pub fn get_sale(&self, nft_contract_id: AccountId, token_id: String) -> Option<Sale> {
        self.sales.get(&(nft_contract_id, token_id))
    }

    /// Listing aktif dengan paginasi; `limit` di-clamp ke 100 (bukan error) — pola
    /// docs/features/marketplace.md §Discovery. **Urutan tidak dijamin** (urutan map); sort/filter
    /// MVP dilakukan client-side atas hasil view (ADR-004) — `newest` = `listed_at` desc di FE.
    pub fn get_sales(&self, offset: Option<u64>, limit: Option<u8>) -> Vec<Sale> {
        // `offset` di luar `usize` (mis. di wasm32) = lewat akhir koleksi → hasil kosong, bukan panic.
        let offset = usize::try_from(offset.unwrap_or(0)).unwrap_or(usize::MAX);
        let limit = limit.map_or(20, usize::from).min(100);
        self.sales.values().skip(offset).take(limit).collect()
    }

    /// Jumlah listing aktif.
    pub fn get_supply_sales(&self) -> u64 {
        self.sales.len()
    }

    /// Mutasi baru diblokir saat paused (INV-022); jalur cancel/withdraw tidak memakai helper ini.
    fn assert_not_paused() {
        if Self::is_paused() {
            env::panic_str(err::CHAIN_PAUSED);
        }
    }

    /// Deposit terlampir (bila ada) masuk ke saldo storage NEP-145 pemanggil.
    fn credit_attached_storage(&mut self, account_id: &AccountId) {
        let attached = env::attached_deposit();
        if attached.is_zero() {
            return;
        }
        Nep145Controller::deposit_to_storage_account(self, account_id, attached)
            .unwrap_or_else(|e| env::panic_str(&format!("{}: {e}", err::CHAIN_REVERT)));
    }

    /// Saldo storage seller wajib menutup satu entry `Sale` sebelum listing diterima (INV-020).
    fn assert_storage_for_listing(&self, account_id: &AccountId) {
        let balance = self
            .get_storage_balance(account_id)
            .unwrap_or_else(|_| env::panic_str(err::CHAIN_REVERT));
        if balance.available < storage_per_sale() {
            env::panic_str(err::CHAIN_REVERT);
        }
    }

    /// Tagih pertumbuhan storage ke pemilik entry; saat entry dihapus, kredit dikembalikan (INV-020).
    fn charge_storage(&mut self, account_id: &AccountId, storage_usage_start: u64) {
        Nep145Controller::storage_accounting(self, account_id, storage_usage_start)
            .unwrap_or_else(|e| env::panic_str(&format!("{}: {e}", err::CHAIN_REVERT)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::mock::MockAction;
    use near_sdk::test_utils::{get_created_receipts, get_logs, VMContextBuilder};
    use near_sdk::{testing_env, NearToken};
    use near_sdk_contract_tools::nft::Nep145 as Nep145External;
    use near_sdk_contract_tools::owner::OwnerExternal;

    const PRICE: u128 = 1_000_000_000_000_000_000_000_000;
    const APPROVAL_ID: u64 = 7;
    const TOKEN: &str = "0";
    /// Awal window waktu test (nanodetik) — stabil di semua run.
    const T0: u64 = 1_800_000_000_000_000_000;

    fn owner() -> AccountId {
        "owner.testnet".parse().unwrap()
    }

    fn seller() -> AccountId {
        "alice.testnet".parse().unwrap()
    }

    fn buyer() -> AccountId {
        "bob.testnet".parse().unwrap()
    }

    fn market_id() -> AccountId {
        "market.testnet".parse().unwrap()
    }

    fn nft() -> AccountId {
        "nft.testnet".parse().unwrap()
    }

    fn run(predecessor: &AccountId, deposit: u128) {
        testing_env!(VMContextBuilder::new()
            .current_account_id(market_id())
            .predecessor_account_id(predecessor.clone())
            .attached_deposit(NearToken::from_yoctonear(deposit))
            .block_timestamp(T0)
            .build());
    }

    fn new_market() -> Market {
        run(&owner(), 0);
        Market::new(owner())
    }

    fn deposit_storage(market: &mut Market, account: &AccountId, amount: u128) {
        run(account, amount);
        market.storage_deposit(None, None);
    }

    fn fund_storage(market: &mut Market, account: &AccountId) {
        deposit_storage(market, account, storage_per_sale().as_yoctonear());
    }

    fn token_owned_by(token_id: &str, owner: AccountId) -> Token {
        Token {
            token_id: token_id.to_string(),
            owner_id: owner,
            extensions_metadata: Default::default(),
        }
    }

    fn list_token(market: &mut Market, token_id: &str, price: u128) -> Promise {
        run(&seller(), 0);
        market.list_nft_for_sale(
            nft(),
            token_id.to_string(),
            Some(APPROVAL_ID),
            U128(price),
            None,
        )
    }

    /// Jalankan callback `process_listing` langsung (di unit test tidak ada promise nyata).
    fn verify_token(
        market: &mut Market,
        token_id: &str,
        price: u128,
        owner: AccountId,
        approved: Result<bool, PromiseError>,
    ) {
        verify_token_with(market, token_id, price, owner, approved, None);
    }

    fn verify_token_with(
        market: &mut Market,
        token_id: &str,
        price: u128,
        owner: AccountId,
        approved: Result<bool, PromiseError>,
        allowed_buyer: Option<AccountId>,
    ) {
        run(&nft(), 0);
        market.process_listing(
            Ok(Some(token_owned_by(token_id, owner))),
            approved,
            seller(),
            nft(),
            token_id.to_string(),
            Some(APPROVAL_ID),
            U128(price),
            allowed_buyer,
        );
    }

    /// Market dengan satu listing aktif milik `seller()` untuk `TOKEN`.
    fn listed_market() -> Market {
        let mut market = new_market();
        fund_storage(&mut market, &seller());
        let _ = list_token(&mut market, TOKEN, PRICE);
        verify_token(&mut market, TOKEN, PRICE, seller(), Ok(true));
        market
    }

    /// Nama method pada receipt yang dibuat sejak `testing_env!` terakhir.
    fn created_method_names() -> Vec<String> {
        get_created_receipts()
            .iter()
            .flat_map(|receipt| receipt.actions.iter())
            .filter_map(|action| match action {
                MockAction::FunctionCallWeight { method_name, .. } => {
                    Some(String::from_utf8(method_name.clone()).expect("utf8 method name"))
                }
                _ => None,
            })
            .collect()
    }

    // --- init & pause (SEC-CONTRACT-002, SEC-CONTRACT-007) ---

    #[test]
    fn test_new_sets_owner_and_storage_bounds() {
        let market = new_market();

        assert_eq!(market.own_get_owner(), Some(owner()));
        assert_eq!(market.storage_balance_bounds().min, storage_per_sale());
        assert_eq!(market.storage_balance_bounds().max, None);
        assert_eq!(market.get_supply_sales(), 0);
    }

    #[test]
    fn test_pause_toggles_state() {
        let mut market = new_market();
        assert!(!Market::is_paused());

        market.pause();
        assert!(Market::is_paused());

        market.unpause();
        assert!(!Market::is_paused());
    }

    // --- listing: dual verification (TC-002 paruh list, INV-007/013, SEC-ORDER-004) ---

    #[test]
    fn test_list_creates_sale_after_dual_verification() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());

        let promise = list_token(&mut market, TOKEN, PRICE);
        drop(promise);

        // Non-custodial: jalur listing hanya membuat view call — tidak ada transfer keluar (ADR-007).
        let methods = created_method_names();
        assert!(
            methods.contains(&"nft_token".to_string())
                && methods.contains(&"nft_is_approved".to_string()),
            "dual verification wajib dua view call ke koleksi: {methods:?}"
        );
        assert!(
            !methods.contains(&"nft_transfer".to_string())
                && !methods.contains(&"nft_transfer_payout".to_string()),
            "listing tidak boleh memindahkan NFT: {methods:?}"
        );

        verify_token(&mut market, TOKEN, PRICE, seller(), Ok(true));

        let sale = market
            .get_sale(nft(), TOKEN.to_string())
            .expect("listing aktif");
        assert_eq!(sale.owner_id, seller());
        assert_eq!(sale.price_yocto, U128(PRICE));
        assert_eq!(sale.approval_id, Some(APPROVAL_ID));
        assert_eq!(sale.allowed_buyer, None);
        assert_eq!(sale.listed_at, T0);
        assert_eq!(market.get_supply_sales(), 1);

        let logs = get_logs();
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""event":"market_list""#)
                    && log.contains(r#""standard":"x-nearsea-market""#)
                    && log.contains(r#""seller":"alice.testnet""#)
                    && log.contains(r#""token_id":"0""#)
                    && log.contains(&format!(r#""price_yocto":"{PRICE}""#))
                    && log.contains(r#""approval_id":7"#)),
            "event market_list wajib ter-emit dengan payload katalog: {logs:?}"
        );
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_list_rejected_when_caller_is_not_token_owner() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());
        let _ = list_token(&mut market, TOKEN, PRICE);

        // Koleksi melaporkan pemilik lain → klaim sepihak seller ditolak.
        verify_token(&mut market, TOKEN, PRICE, buyer(), Ok(true));
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_list_rejected_when_market_is_not_approved() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());
        let _ = list_token(&mut market, TOKEN, PRICE);

        verify_token(&mut market, TOKEN, PRICE, seller(), Ok(false));
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_list_rejected_when_verification_promise_failed() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());
        let _ = list_token(&mut market, TOKEN, PRICE);

        verify_token(
            &mut market,
            TOKEN,
            PRICE,
            seller(),
            Err(PromiseError::Failed),
        );
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_list_rejected_when_token_does_not_exist() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());

        run(&nft(), 0);
        market.process_listing(
            Ok(None),
            Ok(true),
            seller(),
            nft(),
            TOKEN.to_string(),
            Some(APPROVAL_ID),
            U128(PRICE),
            None,
        );
    }

    // --- listing: harga, duplikat, storage, paused (INV-030/007/020/022, TC-013/020) ---

    #[test]
    fn test_list_accepts_price_exactly_at_minimum() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());

        let _ = list_token(&mut market, TOKEN, MIN_PRICE_YOCTO);
        verify_token(&mut market, TOKEN, MIN_PRICE_YOCTO, seller(), Ok(true));

        assert_eq!(
            market
                .get_sale(nft(), TOKEN.to_string())
                .expect("listing aktif")
                .price_yocto,
            U128(MIN_PRICE_YOCTO)
        );
    }

    #[test]
    #[should_panic(expected = "INVALID_PRICE")]
    fn test_list_rejects_price_below_minimum() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());

        let _ = list_token(&mut market, TOKEN, MIN_PRICE_YOCTO - 1);
    }

    #[test]
    #[should_panic(expected = "CONFLICT_ALREADY_LISTED")]
    fn test_list_rejects_duplicate_listing() {
        let mut market = listed_market();

        let _ = list_token(&mut market, TOKEN, PRICE);
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_list_rejected_without_storage_deposit() {
        let mut market = new_market();

        let _ = list_token(&mut market, TOKEN, PRICE);
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_list_rejected_when_attached_deposit_below_minimum() {
        let mut market = new_market();

        run(&seller(), storage_per_sale().as_yoctonear() - 1);
        let _ = market.list_nft_for_sale(
            nft(),
            TOKEN.to_string(),
            Some(APPROVAL_ID),
            U128(PRICE),
            None,
        );
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_list_rejects_approval_id_out_of_u32_range() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());

        run(&seller(), 0);
        let _ = market.list_nft_for_sale(
            nft(),
            TOKEN.to_string(),
            Some(u64::from(u32::MAX) + 1),
            U128(PRICE),
            None,
        );
    }

    #[test]
    #[should_panic(expected = "CHAIN_PAUSED")]
    fn test_list_rejected_when_paused() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());
        Pause::set_is_paused(&mut market, true);

        let _ = list_token(&mut market, TOKEN, PRICE);
    }

    #[test]
    fn test_list_private_listing_records_allowed_buyer() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());

        run(&seller(), 0);
        let promise = market.list_nft_for_sale(
            nft(),
            TOKEN.to_string(),
            Some(APPROVAL_ID),
            U128(PRICE),
            Some(buyer()),
        );
        drop(promise);
        verify_token_with(&mut market, TOKEN, PRICE, seller(), Ok(true), Some(buyer()));

        assert_eq!(
            market
                .get_sale(nft(), TOKEN.to_string())
                .expect("listing aktif")
                .allowed_buyer,
            Some(buyer())
        );
    }

    // --- remove_sale (TC-044, INV-012/022) ---

    #[test]
    fn test_remove_sale_deletes_listing_and_emits_delist() {
        let mut market = listed_market();

        run(&seller(), 1);
        market.remove_sale(nft(), TOKEN.to_string());

        assert!(market.get_sale(nft(), TOKEN.to_string()).is_none());
        assert_eq!(market.get_supply_sales(), 0);

        let logs = get_logs();
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""event":"market_delist""#)
                    && log.contains(r#""seller":"alice.testnet""#)
                    && log.contains(r#""token_id":"0""#)),
            "event market_delist wajib ter-emit: {logs:?}"
        );
    }

    #[test]
    fn test_remove_sale_releases_storage_to_seller() {
        let mut market = listed_market();
        let locked = market
            .storage_balance_of(seller())
            .expect("seller terdaftar")
            .available;

        run(&seller(), 1);
        market.remove_sale(nft(), TOKEN.to_string());

        let released = market
            .storage_balance_of(seller())
            .expect("seller terdaftar")
            .available;
        assert!(
            released > locked,
            "storage entry yang dibebaskan kembali ke saldo seller ({released:?} > {locked:?})"
        );
    }

    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_remove_sale_requires_one_yocto() {
        let mut market = listed_market();

        run(&seller(), 0);
        market.remove_sale(nft(), TOKEN.to_string());
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_remove_sale_rejected_for_non_owner() {
        let mut market = listed_market();

        run(&buyer(), 1);
        market.remove_sale(nft(), TOKEN.to_string());
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_remove_sale_rejected_for_unknown_listing() {
        let mut market = new_market();

        run(&seller(), 1);
        market.remove_sale(nft(), TOKEN.to_string());
    }

    #[test]
    fn test_remove_sale_allowed_while_paused() {
        let mut market = listed_market();
        Pause::set_is_paused(&mut market, true);

        run(&seller(), 1);
        market.remove_sale(nft(), TOKEN.to_string());

        assert!(market.get_sale(nft(), TOKEN.to_string()).is_none());
    }

    // --- update_price (INV-030, INV-012) ---

    #[test]
    fn test_update_price_changes_price_in_place() {
        let mut market = listed_market();
        let new_price = PRICE * 2;

        run(&seller(), 1);
        market.update_price(nft(), TOKEN.to_string(), U128(new_price));

        let sale = market
            .get_sale(nft(), TOKEN.to_string())
            .expect("listing tetap ada");
        assert_eq!(sale.price_yocto, U128(new_price));
        assert_eq!(
            sale.approval_id,
            Some(APPROVAL_ID),
            "update harga tidak mengubah approval_id"
        );

        let logs = get_logs();
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""event":"market_update_price""#)
                    && log.contains(&format!(r#""old_price_yocto":"{PRICE}""#))
                    && log.contains(&format!(r#""new_price_yocto":"{new_price}""#))),
            "event market_update_price wajib memuat harga lama & baru: {logs:?}"
        );
    }

    #[test]
    #[should_panic(expected = "INVALID_PRICE")]
    fn test_update_price_rejects_below_minimum() {
        let mut market = listed_market();

        run(&seller(), 1);
        market.update_price(nft(), TOKEN.to_string(), U128(MIN_PRICE_YOCTO - 1));
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_update_price_rejected_for_non_owner() {
        let mut market = listed_market();

        run(&buyer(), 1);
        market.update_price(nft(), TOKEN.to_string(), U128(PRICE * 2));
    }

    #[test]
    #[should_panic(expected = "CHAIN_PAUSED")]
    fn test_update_price_rejected_when_paused() {
        let mut market = listed_market();
        Pause::set_is_paused(&mut market, true);

        run(&seller(), 1);
        market.update_price(nft(), TOKEN.to_string(), U128(PRICE * 2));
    }

    // --- view listing (§5) ---

    #[test]
    fn test_get_sales_pagination_and_supply() {
        let mut market = new_market();
        // Satu deposit menutup tiga entry `Sale`.
        deposit_storage(
            &mut market,
            &seller(),
            storage_per_sale().as_yoctonear() * 3,
        );

        for token_id in ["0", "1", "2"] {
            let _ = list_token(&mut market, token_id, PRICE);
            verify_token(&mut market, token_id, PRICE, seller(), Ok(true));
        }

        assert_eq!(market.get_supply_sales(), 3);

        // Urutan map tidak dijamin (sort milik FE — ADR-004), jadi assert atas *himpunan* id,
        // bukan posisi: halaman offset+limit harus subset dari seluruh id dan berukuran `limit`.
        let all: Vec<String> = market
            .get_sales(None, None)
            .iter()
            .map(|sale| sale.token_id.clone())
            .collect();
        assert_eq!(all.len(), 3);

        let page: Vec<String> = market
            .get_sales(Some(1), Some(1))
            .iter()
            .map(|sale| sale.token_id.clone())
            .collect();
        assert_eq!(page.len(), 1);
        assert!(
            all.contains(&page[0]),
            "halaman {page:?} harus berisi id dari koleksi {all:?}"
        );

        // `limit` di-clamp ke 100, bukan error (docs/features/marketplace.md §Discovery).
        assert_eq!(market.get_sales(None, Some(255)).len(), 3);
        assert_eq!(market.get_sales(Some(9), None).len(), 0);
    }
}
