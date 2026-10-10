//! NearSea Market — listing 2-tx (dual verification) + settlement buy + storage NEP-145.
//!
//! Reference implementasi: docs/contracts/market.md §1–§3b, §4–§8.
//! Tiket ini (TASK-005) menambahkan jalur settlement: `buy` + dual verification `process_purchase`,
//! callback `resolve_purchase` (validasi payout UNTRUSTED + distribusi fee/royalti/seller + refund),
//! `pending_purchases` + `recover_stuck_purchase` (INV-031), dan konfigurasi `fee_bps`/`treasury`.
//! Offer/bundle serta `remove_stale_listing` milik TASK-009/010/022.
//!
//! Prefix storage kanonik: docs/contracts/market.md §7 (SEC-CONTRACT-008).

use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::ext_contract;
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::{
    assert_one_yocto, env, near, AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault,
    Promise, PromiseError,
};
use near_sdk_contract_tools::nft::{
    ext_nep171, ext_nep178, Nep145Controller, StorageBalanceBounds, Token, TokenId,
};
use near_sdk_contract_tools::owner::Owner;
use near_sdk_contract_tools::pause::Pause;
use near_sdk_contract_tools::standard::nep297::{Event, EventLog, ToEventLog};
use near_sdk_contract_tools::{Nep145, Owner as OwnerDerive, Pause as PauseDerive};
use std::borrow::Cow;
use std::collections::HashMap;

/// Fee platform default 2% — nilai fee ≠ cap (INV-004, ADR-005).
pub const FEE_BPS_DEFAULT: u16 = 200;
/// Cap immutable fee — ditegakkan saat init **dan** di `update_fee_bps` (SEC-CONTRACT-004).
pub const MAX_FEE_BPS: u16 = 500;
/// Batas statis penerima payout dari koleksi (INV-003/021) — juga dikirim sebagai
/// `max_len_payout` ke NEP-199 sehingga koleksi yang mengembalikan lebih banyak langsung ditolak.
pub const MAX_PAYOUT_RECEIVERS: u32 = 10;
/// Jeda blok sebelum pembelian yang nyangkut boleh dipulihkan siapa pun (INV-031, §3b).
/// PROPOSED — cukup agar rantai `buy` (transfer + resolve) pasti selesai/gagal, cukup pendek
/// agar buyer tidak menunggu lama.
pub const RECOVERY_DELAY_BLOCKS: u64 = 20;

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

/// Gas callback `resolve_purchase` — FACT (docs/contracts/market.md §6; RESEARCH.md §10.2).
/// Worst case = 10 penerima payout + fee + seller + refund + event; diuji di unit
/// (`test_resolve_purchase_gas_within_budget`).
pub const GAS_FOR_RESOLVE_PURCHASE: Gas = Gas::from_tgas(115);
/// Gas `nft_transfer_payout` (NEP-199, attach 1 yocto) — FACT (docs/contracts/market.md §6).
pub const GAS_FOR_NFT_TRANSFER: Gas = Gas::from_tgas(15);
/// Gas callback `process_purchase` = anggaran transfer + callback settle + overhead sendiri.
/// PROPOSED (15 + 115 + 5); angka ini yang di-attach `buy` sehingga total jalur buy
/// = 2 view + 135 = 145 Tgas (batas dokumen <150).
pub const GAS_FOR_PROCESS_PURCHASE: Gas = Gas::from_gas(
    GAS_FOR_NFT_TRANSFER.as_gas() + GAS_FOR_RESOLVE_PURCHASE.as_gas() + Gas::from_tgas(5).as_gas(),
);
/// Gas callback `process_recovery` (refund + restore opsional + event) — PROPOSED.
pub const GAS_FOR_PROCESS_RECOVERY: Gas = Gas::from_tgas(10);

/// Teks panic kontrak = kode registry, supaya FE memetakan ke kode user yang stabil
/// (docs/development/error-handling.md §3/§4). Kegagalan dari library/derive diprefiks dengan kode
/// registry (`format!("{CHAIN_REVERT}: {e}")`) — pola yang sama dipakai kontrak koleksi
/// (docs/contracts/nft-collection.md §4): FE mencocokkan prefiks, detail library tetap terbaca di log.
mod err {
    pub const CHAIN_REVERT: &str = "CHAIN_REVERT";
    pub const CHAIN_PAUSED: &str = "CHAIN_PAUSED";
    pub const CHAIN_INSUFFICIENT_DEPOSIT: &str = "CHAIN_INSUFFICIENT_DEPOSIT";
    pub const INVALID_PRICE: &str = "INVALID_PRICE";
    pub const CONFLICT_SOLD: &str = "CONFLICT_SOLD";
    pub const CONFLICT_ALREADY_LISTED: &str = "CONFLICT_ALREADY_LISTED";
    pub const FORBIDDEN_SELF_BUY: &str = "FORBIDDEN_SELF_BUY";
    pub const FORBIDDEN_BUYER: &str = "FORBIDDEN_BUYER";
}

/// Prefix storage milik kontrak ini — bukan `~*` (milik derive near-sdk-contract-tools).
#[derive(BorshStorageKey, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    Sales,
    PendingPurchases,
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

/// Payout NEP-199 yang dikembalikan kontrak koleksi. **UNTRUSTED** — market memvalidasi
/// ulang (jumlah penerima, amount > 0, Σ ≤ harga−fee) sebelum mendistribusikan
/// (docs/features/payments.md §Algoritma Distribusi Payout).
pub type Payout = HashMap<AccountId, U128>;

/// Pembelian yang sedang berjalan (INV-031, docs/contracts/market.md §3b).
/// Ditulis **sebelum** optimistic removal; dihapus hanya oleh `resolve_purchase`
/// (sukses/gagal) atau `recover_stuck_purchase`.
///
/// `sale` = snapshot listing yang dihapus, `fee_bps` = snapshot fee saat `buy`. Keduanya di-snapshot
/// supaya split distribusi ditentukan oleh syarat yang berlaku saat pembeli menandatangani, bukan
/// oleh state yang bisa berubah di antara dua receipt (`update_fee_bps` owner-only).
/// Di-serialize JSON juga karena `get_pending_purchase` membacanya untuk ops/FE (TC-054).
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingPurchase {
    pub buyer: AccountId,
    pub deposit: u128,
    pub created_height: u64,
    pub fee_bps: u16,
    pub sale: Sale,
}

/// Interface NEP-199 (`nft_transfer_payout`). `near-sdk-contract-tools` 4.0 tidak punya derive
/// untuk NEP-199 (docs/contracts/nft-collection.md §2), jadi dideklarasikan di sini.
#[ext_contract(ext_nep199)]
pub trait Nep199 {
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        balance: U128,
        max_len_payout: Option<u32>,
    ) -> Payout;
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

/// Isi `data[0]` event `market_sale`. `payout` = map penerima **final** (royalti + seller,
/// sudah di-merge per receiver) sehingga `Σpayout == harga − fee` — fee ke treasury
/// ditransfer terpisah dan **tidak** masuk map ini (docs/api/webhooks.md §`market_sale`).
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketSale {
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub buyer: AccountId,
    pub seller: AccountId,
    pub price_yocto: U128,
    pub payout: HashMap<AccountId, U128>,
}

/// Isi `data[0]` event `market_stale_detected` — di-emit saat staleness terbukti on-chain
/// (dua kasus INV-016). `reason` ∈ `ownership_mismatch` | `approval_revoked`.
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketStaleDetected {
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub seller: AccountId,
    pub detected_owner: AccountId,
    pub reason: String,
}

/// Isi `data[0]` event `market_purchase_recovered` (INV-031).
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketPurchaseRecovered {
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub buyer: AccountId,
    pub refund_yocto: U128,
    pub sale_restored: bool,
}

/// Isi `data[0]` event `fee_update`.
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeeUpdate {
    pub old_fee_bps: u16,
    pub new_fee_bps: u16,
    pub caller: AccountId,
}

/// Isi `data[0]` event `treasury_update`.
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreasuryUpdate {
    pub old_treasury: AccountId,
    pub new_treasury: AccountId,
    pub caller: AccountId,
}

/// `fee = floor(harga × fee_bps / 10_000)` — pembagian integer, tanpa float (ADR-005, INV-004).
/// `fee_bps ≤ MAX_FEE_BPS` dijaga saat init/`update_fee_bps`, jadi `fee ≤ harga` selalu.
fn fee_amount(price: u128, fee_bps: u16) -> u128 {
    price
        .checked_mul(u128::from(fee_bps))
        .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT))
        / 10_000
}

/// Validasi payout UNTRUSTED dari kontrak koleksi (INV-002/003, SEC-CONTRACT-005):
/// `1 ≤ len ≤ MAX_PAYOUT_RECEIVERS`, setiap `amount > 0`, dan `Σ ≤ harga − fee`.
/// Sisa (`harga − fee − Σ`) jatuh ke seller sebagai residual — bukan pelanggaran.
fn valid_payout(payout: &Payout, max_pay: u128) -> bool {
    if payout.is_empty() || payout.len() > MAX_PAYOUT_RECEIVERS as usize {
        return false;
    }

    let mut sum: u128 = 0;
    for amount in payout.values() {
        if amount.0 == 0 {
            return false;
        }
        match sum.checked_add(amount.0) {
            Some(total) => sum = total,
            None => return false,
        }
    }

    sum <= max_pay
}

/// Storage minimum satu entry `Sale` (byte × `storage_byte_cost`) — INV-020.
fn storage_per_sale() -> NearToken {
    env::storage_byte_cost().saturating_mul(u128::from(STORAGE_PER_SALE_BYTES))
}

#[near(contract_state)]
#[derive(PanicOnDefault, OwnerDerive, PauseDerive, Nep145)]
pub struct Market {
    sales: UnorderedMap<SaleKey, Sale>,
    /// Fee platform (bps) — hanya dipakai settlement (ADR-005); cap ditegakkan di mutasi.
    fee_bps: u16,
    /// Penerima fee saat settlement (§3). ⏳ open-by-design: diisi saat deploy testnet.
    treasury: AccountId,
    /// Pembelian berjalan (INV-031). State transien: satu entri per sale key yang sedang settle.
    pending_purchases: LookupMap<SaleKey, PendingPurchase>,
}

#[near]
impl Market {
    /// Init sekali (PanicOnDefault) — SEC-CONTRACT-002 (TC-001).
    /// `fee_bps` default 200 (2%) dan wajib ≤ `MAX_FEE_BPS`; `treasury` default = `owner_id`
    /// (⏳ open-by-design — alamat treasury ditetapkan saat deploy testnet, docs/contracts/market.md §1).
    #[init]
    pub fn new(owner_id: AccountId, fee_bps: Option<u16>, treasury: Option<AccountId>) -> Self {
        let fee_bps = fee_bps.unwrap_or(FEE_BPS_DEFAULT);
        if fee_bps > MAX_FEE_BPS {
            env::panic_str(err::CHAIN_REVERT);
        }

        let mut contract = Self {
            sales: UnorderedMap::new(StorageKey::Sales),
            fee_bps,
            treasury: treasury.unwrap_or_else(|| owner_id.clone()),
            pending_purchases: LookupMap::new(StorageKey::PendingPurchases),
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

        // Deposit terlampir masuk saldo storage seller, lalu saldo wajib cukup untuk entry `Sale`.
        // Dicek di sini (sinkron) supaya deposit kurang = revert tx, bukan callback gagal senyap.
        self.credit_attached_storage(&seller);
        self.assert_storage_for_listing(&seller);

        self.dual_verify(&nft_contract_id, &token_id, approval_id)
            .then(
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
    ///
    /// Callback ini receipt **terpisah** dari `list_nft_for_sale`, jadi ia mengecek pause sendiri:
    /// kontrak bisa di-pause antara tx listing dan callback, dan listing baru tetap "mutasi baru"
    /// yang dilarang saat paused (INV-022).
    #[private]
    #[allow(clippy::too_many_arguments)] // 2 hasil callback + 6 konteks listing (docs/contracts/market.md §3)
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
        Self::assert_not_paused();

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
        self.settle_storage_delta(&seller_id, storage_usage_start);

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
        let seller = self.assert_sale_owner(&key);

        let storage_usage_start = env::storage_usage();
        self.sales.remove(&key);
        // Storage yang dibebaskan kembali ke saldo seller — ditarik lewat `storage_withdraw`.
        self.settle_storage_delta(&seller, storage_usage_start);

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

    /// NEP-178 receiver: notifikasi tx-1 dari kontrak koleksi ketika seller memanggil
    /// `nft_approve(market, msg)` (docs/features/marketplace.md §Flow — List langkah 2).
    /// **Bukan** `#[private]` — pemanggilnya kontrak NFT, bukan market sendiri (INV-013).
    ///
    /// Sengaja **tidak** membuat listing (ADR-002): listing hanya lahir dari `list_nft_for_sale` +
    /// dual verification. Karena method ini tidak menulis state apa pun, notifikasi palsu dari akun
    /// mana pun tidak punya efek — yang dijaga hanya bentuk payloadnya.
    pub fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u32,
        msg: String,
    ) {
        if token_id.is_empty() || owner_id == env::current_account_id() || msg.is_empty() {
            env::panic_str(err::CHAIN_REVERT);
        }
        // Market tidak menyimpan approval, jadi `approval_id` tidak bisa diverifikasi lokal —
        // nilainya baru diuji saat `list_nft_for_sale` mengecek ulang lewat `nft_is_approved`.
        let _ = approval_id;
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
        let seller = self.assert_sale_owner(&key);

        let mut sale = self
            .sales
            .get(&key)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
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

    // --- Settlement buy (docs/contracts/market.md §2/§3/§3a/§3b) ---

    /// Beli listing aktif. Deposit ≥ harga; kelebihan (harga berubah saat signing) dikembalikan
    /// di `resolve_purchase` (INV-001).
    ///
    /// Urutan wajib (INV-031): tulis `pending_purchases` **sebelum** optimistic removal, lalu
    /// verifikasi ulang kepemilikan + approval lewat dua view call (`process_purchase`) —
    /// klaim pemanggil tidak dipercaya (SEC-ORDER-004). Entry `Sale` dihapus di sini, jadi
    /// pembeli lain yang menyerbu listing yang sama revert (`CONFLICT_SOLD`) — INV-007/008.
    #[payable]
    pub fn buy(&mut self, nft_contract_id: AccountId, token_id: String) -> Promise {
        Self::assert_not_paused();

        let buyer = env::predecessor_account_id();
        let key = (nft_contract_id.clone(), token_id.clone());

        let sale = self
            .sales
            .get(&key)
            .unwrap_or_else(|| env::panic_str(err::CONFLICT_SOLD));

        if sale.owner_id == buyer {
            env::panic_str(err::FORBIDDEN_SELF_BUY);
        }
        if let Some(allowed_buyer) = &sale.allowed_buyer {
            if allowed_buyer != &buyer {
                env::panic_str(err::FORBIDDEN_BUYER);
            }
        }

        let deposit = env::attached_deposit().as_yoctonear();
        if deposit < sale.price_yocto.0 {
            env::panic_str(err::CHAIN_INSUFFICIENT_DEPOSIT);
        }

        // Satu pembelian per sale key: entri yang sudah ada = pembelian lain sedang settle.
        // Bagi pembeli hasilnya sama dengan listing yang sudah terjual — dana kembali penuh.
        if self.pending_purchases.get(&key).is_some() {
            env::panic_str(err::CONFLICT_SOLD);
        }

        self.pending_purchases.insert(
            &key,
            &PendingPurchase {
                buyer: buyer.clone(),
                deposit,
                created_height: env::block_height(),
                fee_bps: self.fee_bps,
                sale: sale.clone(),
            },
        );

        let storage_usage_start = env::storage_usage();
        self.sales.remove(&key);
        // Storage yang dibebaskan kembali ke saldo seller (ditarik lewat `storage_withdraw`).
        self.settle_storage_delta(&sale.owner_id, storage_usage_start);

        self.dual_verify(&nft_contract_id, &token_id, sale.approval_id)
            .then(
                Self::ext_self()
                    .with_static_gas(GAS_FOR_PROCESS_PURCHASE)
                    .process_purchase(nft_contract_id, token_id),
            )
    }

    /// Hasil dual verification pembelian (INV-013 — `#[private]`). Membaca konteks pembelian
    /// dari `pending_purchases`, jadi tidak ada argumen tambahan yang bisa dipalsukan.
    ///
    /// Callback ini receipt **terpisah** dari `buy`, jadi ia menegakkan pause sendiri: kontrak bisa
    /// di-pause antara tx buy dan callback, dan transfer NFT tetap "mutasi baru" yang dilarang
    /// saat paused (INV-022). Bila paused, pembelian dibatalkan **tanpa** menyentuh NFT: pending
    /// dihapus, `Sale` dipulihkan, deposit dikembalikan — pola yang sama dengan `process_listing`.
    ///
    /// Tiga hasil lain (docs/contracts/market.md §3a):
    /// - **stale** (kepemilikan pindah / approval tidak valid) → refund penuh, entry `Sale`
    ///   **tidak** dipulihkan (listing mati), event `market_stale_detected`;
    /// - **verifikasi tidak pasti** (koleksi tak bisa dihubungi / token tidak ada) → refund penuh
    ///   + `Sale` dipulihkan (kegagalan bukan stale);
    /// - **valid** → `nft_transfer_payout` (1 yocto, NEP-199) → `resolve_purchase`.
    #[private]
    pub fn process_purchase(
        &mut self,
        #[callback_result] token: Result<Option<Token>, PromiseError>,
        #[callback_result] approved: Result<bool, PromiseError>,
        nft_contract_id: AccountId,
        token_id: String,
    ) -> Promise {
        let key = (nft_contract_id.clone(), token_id.clone());
        let pending = self
            .pending_purchases
            .get(&key)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));

        if Self::is_paused() {
            self.pending_purchases.remove(&key);
            self.restore_sale(&key, &pending.sale);

            return self.refund(&pending);
        }

        let token_owner = match token {
            Ok(Some(token)) => Some(token.owner_id),
            _ => None,
        };

        let stale_reason = match (&token_owner, &approved) {
            (Some(owner), _) if owner != &pending.sale.owner_id => Some("ownership_mismatch"),
            (Some(_), Ok(false)) => Some("approval_revoked"),
            _ => None,
        };

        if let Some(reason) = stale_reason {
            self.pending_purchases.remove(&key);
            SingleEvent::new(
                "market_stale_detected",
                MarketStaleDetected {
                    nft_contract_id,
                    token_id,
                    seller: pending.sale.owner_id.clone(),
                    detected_owner: token_owner.unwrap_or_else(|| pending.sale.owner_id.clone()),
                    reason: reason.to_string(),
                },
            )
            .emit();

            return self.refund(&pending);
        }

        if token_owner.is_none() || !matches!(approved, Ok(true)) {
            self.pending_purchases.remove(&key);
            self.restore_sale(&key, &pending.sale);

            return self.refund(&pending);
        }

        ext_nep199::ext(nft_contract_id.clone())
            .with_static_gas(GAS_FOR_NFT_TRANSFER)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .nft_transfer_payout(
                pending.buyer.clone(),
                token_id.clone(),
                pending.sale.approval_id,
                None,
                pending.sale.price_yocto,
                Some(MAX_PAYOUT_RECEIVERS),
            )
            .then(
                Self::ext_self()
                    .with_static_gas(GAS_FOR_RESOLVE_PURCHASE)
                    .resolve_purchase(nft_contract_id, token_id),
            )
    }

    /// Settlement (INV-013 — `#[private]`). Payout dari koleksi diperlakukan UNTRUSTED.
    ///
    /// Sukses → distribusi: royalti → receiver, residual → seller, fee → treasury, kelebihan
    /// deposit → buyer, event `market_sale`. Gagal promise **atau** payout invalid → refund penuh
    /// buyer + `Sale` dipulihkan (SEC-ORDER-001). Semua jalur menghapus `pending_purchases`.
    #[private]
    pub fn resolve_purchase(
        &mut self,
        #[callback_result] payout: Result<Payout, PromiseError>,
        nft_contract_id: AccountId,
        token_id: String,
    ) {
        let key = (nft_contract_id.clone(), token_id.clone());
        let pending = self
            .pending_purchases
            .get(&key)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));

        let price = pending.sale.price_yocto.0;
        // Fee dari snapshot saat `buy` — split distribusi ditentukan syarat yang berlaku ketika
        // pembeli menandatangani, bukan state yang bisa berubah di antara dua receipt.
        let fee = fee_amount(price, pending.fee_bps);
        let max_pay = price
            .checked_sub(fee)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));

        let payout = match payout {
            Ok(payout) if valid_payout(&payout, max_pay) => payout,
            _ => {
                self.pending_purchases.remove(&key);
                self.restore_sale(&key, &pending.sale);
                self.refund(&pending).detach();
                return;
            }
        };

        let mut sum: u128 = 0;
        let mut final_payout: HashMap<AccountId, U128> = HashMap::new();
        for (receiver, amount) in payout {
            sum = sum
                .checked_add(amount.0)
                .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
            final_payout
                .entry(receiver)
                .and_modify(|existing| {
                    existing.0 = existing
                        .0
                        .checked_add(amount.0)
                        .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT))
                })
                .or_insert(amount);
        }

        // Sisa pembulatan jatuh ke seller → `fee + Σroyalti + seller == harga` persis (INV-001).
        // `valid_payout` menjamin `sum ≤ max_pay`, jadi pengurangan ini tidak bisa underflow.
        let seller_proceeds = max_pay
            .checked_sub(sum)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
        final_payout
            .entry(pending.sale.owner_id.clone())
            .and_modify(|existing| {
                existing.0 = existing
                    .0
                    .checked_add(seller_proceeds)
                    .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT))
            })
            .or_insert(U128(seller_proceeds));

        for (receiver, amount) in &final_payout {
            Promise::new(receiver.clone())
                .transfer(NearToken::from_yoctonear(amount.0))
                .detach();
        }
        Promise::new(self.treasury.clone())
            .transfer(NearToken::from_yoctonear(fee))
            .detach();

        // Kelebihan deposit (harga berubah saat signing) kembali ke buyer — INV-001.
        let excess = pending
            .deposit
            .checked_sub(price)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
        if excess > 0 {
            Promise::new(pending.buyer.clone())
                .transfer(NearToken::from_yoctonear(excess))
                .detach();
        }

        self.pending_purchases.remove(&key);
        SingleEvent::new(
            "market_sale",
            MarketSale {
                nft_contract_id,
                token_id,
                buyer: pending.buyer,
                seller: pending.sale.owner_id,
                price_yocto: pending.sale.price_yocto,
                payout: final_payout,
            },
        )
        .emit();
    }

    /// Pemulihan pembelian yang nyangkut — **siapa pun** boleh memanggil (INV-031, §3b).
    /// Deposit `buy` selalu punya jalur keluar tanpa bergantung pada owner/governance.
    ///
    /// Syarat tunggal: entri `pending_purchases` ada **dan** sudah lewat `RECOVERY_DELAY_BLOCKS`
    /// (memastikan rantai `buy` sudah selesai/gagal). Boleh saat paused (jalur refund, INV-022).
    #[payable]
    pub fn recover_stuck_purchase(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
    ) -> Promise {
        assert_one_yocto();

        let key = (nft_contract_id.clone(), token_id.clone());
        let pending = self
            .pending_purchases
            .get(&key)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));

        let delay_elapsed = pending
            .created_height
            .checked_add(RECOVERY_DELAY_BLOCKS)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
        if env::block_height() <= delay_elapsed {
            env::panic_str(err::CHAIN_REVERT);
        }

        // Verifikasi ulang sebelum memulihkan listing: entry hanya hidup kembali bila token masih
        // milik seller **dan** approval-nya masih valid (mencegah listing "zombie" — §3a).
        self.dual_verify(&nft_contract_id, &token_id, pending.sale.approval_id)
            .then(
                Self::ext_self()
                    .with_static_gas(GAS_FOR_PROCESS_RECOVERY)
                    .process_recovery(nft_contract_id, token_id),
            )
    }

    /// Hasil pemulihan (INV-013 — `#[private]`). Refund penuh **selalu** dijalankan; `Sale`
    /// dipulihkan hanya bila token masih milik seller, approval valid, dan storage seller menutup
    /// entry — kalau tidak, listing dibiarkan mati (`sale_restored: false`) dan dana buyer tetap kembali.
    #[private]
    pub fn process_recovery(
        &mut self,
        #[callback_result] token: Result<Option<Token>, PromiseError>,
        #[callback_result] approved: Result<bool, PromiseError>,
        nft_contract_id: AccountId,
        token_id: String,
    ) {
        let key = (nft_contract_id.clone(), token_id.clone());
        let pending = self
            .pending_purchases
            .get(&key)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));

        let still_owner = matches!(token, Ok(Some(ref t)) if t.owner_id == pending.sale.owner_id);
        let sale_restored =
            still_owner && matches!(approved, Ok(true)) && self.restore_sale(&key, &pending.sale);

        self.pending_purchases.remove(&key);
        self.refund(&pending).detach();

        SingleEvent::new(
            "market_purchase_recovered",
            MarketPurchaseRecovered {
                nft_contract_id,
                token_id,
                buyer: pending.buyer,
                refund_yocto: U128(pending.deposit),
                sale_restored,
            },
        )
        .emit();
    }

    // --- Konfigurasi fee & treasury (docs/contracts/market.md §4) ---

    /// Ubah fee platform (owner-only, 1 yocto). Cap `MAX_FEE_BPS` immutable — INV-004.
    #[payable]
    pub fn update_fee_bps(&mut self, fee_bps: u16) {
        assert_one_yocto();
        Self::assert_not_paused();
        self.assert_owner();

        if fee_bps > MAX_FEE_BPS {
            env::panic_str(err::CHAIN_REVERT);
        }

        let caller = env::predecessor_account_id();
        let old_fee_bps = self.fee_bps;
        self.fee_bps = fee_bps;

        SingleEvent::new(
            "fee_update",
            FeeUpdate {
                old_fee_bps,
                new_fee_bps: fee_bps,
                caller,
            },
        )
        .emit();
    }

    /// Ubah alamat treasury penerima fee (owner-only, 1 yocto).
    #[payable]
    pub fn update_treasury(&mut self, treasury: AccountId) {
        assert_one_yocto();
        Self::assert_not_paused();
        self.assert_owner();

        let caller = env::predecessor_account_id();
        let old_treasury = std::mem::replace(&mut self.treasury, treasury.clone());

        SingleEvent::new(
            "treasury_update",
            TreasuryUpdate {
                old_treasury,
                new_treasury: treasury,
                caller,
            },
        )
        .emit();
    }

    // --- Views listing & konfigurasi (docs/contracts/market.md §5) ---

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

    /// Fee platform aktif (bps) — dipakai ops & monitoring (docs/deployment/monitoring.md §Alur baca).
    pub fn get_fee_bps(&self) -> u16 {
        self.fee_bps
    }

    /// Alamat treasury penerima fee saat settlement.
    pub fn get_treasury(&self) -> AccountId {
        self.treasury.clone()
    }

    /// Pembelian yang sedang berjalan untuk satu listing (`None` = tidak ada).
    /// Dipakai ops/FE untuk memicu `recover_stuck_purchase` setelah jeda (INV-031, TC-054).
    pub fn get_pending_purchase(
        &self,
        nft_contract_id: AccountId,
        token_id: String,
    ) -> Option<PendingPurchase> {
        self.pending_purchases.get(&(nft_contract_id, token_id))
    }

    /// Mutasi baru diblokir saat paused (INV-022); jalur cancel/withdraw tidak memakai helper ini.
    fn assert_not_paused() {
        if Self::is_paused() {
            env::panic_str(err::CHAIN_PAUSED);
        }
    }

    /// Listing wajib ada **dan** pemanggil = `sale.owner_id`; kembalikan seller-nya (INV-012).
    /// Dipakai `remove_sale` & `update_price` — keduanya mutasi seller dengan aturan otorisasi sama.
    fn assert_sale_owner(&self, key: &SaleKey) -> AccountId {
        let sale = self
            .sales
            .get(key)
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
        let seller = env::predecessor_account_id();
        if sale.owner_id != seller {
            env::panic_str(err::CHAIN_REVERT);
        }
        seller
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

    /// Dual verification ke koleksi (SEC-ORDER-004, ADR-002): `nft_token` (kepemilikan) **dan**
    /// `nft_is_approved` (approval), digabung dengan `.and()` supaya hasilnya masuk ke satu callback.
    /// Dipakai jalur listing, buy, dan recovery — semuanya **tidak** mempercayai klaim pemanggil.
    ///
    /// `approval_id` ABI `u64` → `u32` internal NEP-178: id di luar rentang itu tidak mungkin pernah
    /// diterbitkan koleksi, jadi ditolak di sini (fail-closed), bukan dicek "tanpa id".
    fn dual_verify(
        &self,
        nft_contract_id: &AccountId,
        token_id: &str,
        approval_id: Option<u64>,
    ) -> Promise {
        let approval_u32 = approval_id
            .map(|id| u32::try_from(id).unwrap_or_else(|_| env::panic_str(err::CHAIN_REVERT)));

        let token_promise = ext_nep171::ext(nft_contract_id.clone())
            .with_static_gas(GAS_FOR_NFT_VIEW)
            .nft_token(token_id.to_string());
        let approved_promise = ext_nep178::ext(nft_contract_id.clone())
            .with_static_gas(GAS_FOR_NFT_VIEW)
            .nft_is_approved(
                token_id.to_string(),
                env::current_account_id(),
                approval_u32,
            );

        token_promise.and(approved_promise)
    }

    /// Selisihkan storage sejak `storage_usage_start` ke pemilik entry: tumbuh → ditagih,
    /// menyusut (entry dihapus) → dikreditkan kembali ke saldo (INV-020).
    fn settle_storage_delta(&mut self, account_id: &AccountId, storage_usage_start: u64) {
        Nep145Controller::storage_accounting(self, account_id, storage_usage_start)
            .unwrap_or_else(|e| env::panic_str(&format!("{}: {e}", err::CHAIN_REVERT)));
    }

    /// Refund penuh deposit ke `pending.buyer` — penerima **hardcoded dari state**, bukan alamat
    /// arbitrer (INV-014). Tidak memakai `assert_not_paused` (jalur refund tetap terbuka, INV-022).
    fn refund(&self, pending: &PendingPurchase) -> Promise {
        Promise::new(pending.buyer.clone()).transfer(NearToken::from_yoctonear(pending.deposit))
    }

    /// Pulihkan entry `Sale` dari snapshot pending. Gagal (storage seller tidak lagi menutup
    /// entry) → `false` dan tidak ada yang berubah; pemanggil tetap menjalankan refund.
    fn restore_sale(&mut self, key: &SaleKey, sale: &Sale) -> bool {
        if self.sales.get(key).is_some() {
            return false;
        }

        let storage_usage_start = env::storage_usage();
        self.sales.insert(key, sale);

        // Storage yang dibebaskan saat optimistic removal ditarik kembali dari saldo seller.
        // Saldo kurang → jangan tanggung storage user (SEC-CONTRACT-011): batalkan pemulihan.
        if Nep145Controller::storage_accounting(self, &sale.owner_id, storage_usage_start).is_err()
        {
            self.sales.remove(key);
            return false;
        }

        true
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
    /// `floor(PRICE × 200 / 10_000)` — fee 2% pada `PRICE`.
    const FEE: u128 = PRICE / 50;
    /// Plafon payout = harga − fee.
    const MAX_PAY: u128 = PRICE - FEE;
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

    fn creator() -> AccountId {
        "creator.testnet".parse().unwrap()
    }

    fn stranger() -> AccountId {
        "carol.testnet".parse().unwrap()
    }

    fn treasury() -> AccountId {
        "treasury.testnet".parse().unwrap()
    }

    fn market_id() -> AccountId {
        "market.testnet".parse().unwrap()
    }

    fn nft() -> AccountId {
        "nft.testnet".parse().unwrap()
    }

    fn run(predecessor: &AccountId, deposit: u128) {
        run_at(predecessor, deposit, 0);
    }

    fn run_at(predecessor: &AccountId, deposit: u128, height: u64) {
        testing_env!(VMContextBuilder::new()
            .current_account_id(market_id())
            .predecessor_account_id(predecessor.clone())
            .attached_deposit(NearToken::from_yoctonear(deposit))
            .block_timestamp(T0)
            .block_height(height)
            .build());
    }

    fn new_market() -> Market {
        new_market_with(None, None)
    }

    fn new_market_with(fee_bps: Option<u16>, treasury_id: Option<AccountId>) -> Market {
        run(&owner(), 0);
        Market::new(owner(), fee_bps, treasury_id)
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
    /// Treasury eksplisit supaya assert distribusi fee menunjuk akun yang jelas.
    fn listed_market() -> Market {
        let mut market = new_market_with(None, Some(treasury()));
        fund_storage(&mut market, &seller());
        let _ = list_token(&mut market, TOKEN, PRICE);
        verify_token(&mut market, TOKEN, PRICE, seller(), Ok(true));
        market
    }

    /// Market dengan listing privat untuk `buyer()`.
    fn listed_market_for(buyer_id: AccountId) -> Market {
        let mut market = new_market_with(None, Some(treasury()));
        fund_storage(&mut market, &seller());
        run(&seller(), 0);
        let _ = market.list_nft_for_sale(
            nft(),
            TOKEN.to_string(),
            Some(APPROVAL_ID),
            U128(PRICE),
            Some(buyer_id),
        );
        verify_token_with(&mut market, TOKEN, PRICE, seller(), Ok(true), Some(buyer()));
        market
    }

    /// Panggil `buy` sebagai `who` dengan `deposit`.
    fn buy_as(market: &mut Market, who: &AccountId, deposit: u128) -> Promise {
        run(who, deposit);
        market.buy(nft(), TOKEN.to_string())
    }

    fn buy(market: &mut Market, deposit: u128) -> Promise {
        buy_as(market, &buyer(), deposit)
    }

    /// Jalankan callback `process_purchase` langsung.
    fn verify_purchase(
        market: &mut Market,
        owner: AccountId,
        approved: Result<bool, PromiseError>,
    ) -> Promise {
        run(&nft(), 0);
        market.process_purchase(
            Ok(Some(token_owned_by(TOKEN, owner))),
            approved,
            nft(),
            TOKEN.to_string(),
        )
    }

    /// Jalankan callback `resolve_purchase` langsung dengan hasil payout `result`.
    fn resolve(market: &mut Market, result: Result<Payout, PromiseError>) {
        run(&nft(), 0);
        market.resolve_purchase(result, nft(), TOKEN.to_string());
    }

    /// Jalankan callback `process_recovery` langsung.
    fn verify_recovery(
        market: &mut Market,
        owner: AccountId,
        approved: Result<bool, PromiseError>,
    ) {
        run(&nft(), 0);
        market.process_recovery(
            Ok(Some(token_owned_by(TOKEN, owner))),
            approved,
            nft(),
            TOKEN.to_string(),
        );
    }

    /// Payout royalti satu penerima (bentuk yang dikembalikan koleksi NearSea, TASK-003).
    fn royalty_payout(amount: u128) -> Payout {
        Payout::from([(creator(), U128(amount))])
    }

    /// Transfer Ⓝ yang dijadwalkan sejak `testing_env!` terakhir — `(penerima, jumlah yocto)`.
    fn transfers() -> Vec<(AccountId, u128)> {
        get_created_receipts()
            .iter()
            .flat_map(|receipt| {
                receipt.actions.iter().filter_map(|action| match action {
                    MockAction::Transfer { deposit, .. } => {
                        Some((receipt.receiver_id.clone(), deposit.as_yoctonear()))
                    }
                    _ => None,
                })
            })
            .collect()
    }

    fn transferred_to(account: &AccountId) -> u128 {
        transfers()
            .into_iter()
            .filter(|(receiver, _)| receiver == account)
            .map(|(_, amount)| amount)
            .sum()
    }

    fn sale_of(market: &Market) -> Option<Sale> {
        market.get_sale(nft(), TOKEN.to_string())
    }

    fn pending_of(market: &Market) -> Option<PendingPurchase> {
        market.get_pending_purchase(nft(), TOKEN.to_string())
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

    // market.md §6
    #[test]
    fn test_dual_verify_budget_matches_its_parts() {
        // Anggaran total yang didokumentasikan harus sama dengan penjumlahan komponennya —
        // supaya tidak ada angka kedua yang bisa menyimpang (nilainya sendiri PROPOSED, TASK-006).
        assert_eq!(
            GAS_FOR_DUAL_VERIFY.as_gas(),
            2 * GAS_FOR_NFT_VIEW.as_gas() + GAS_FOR_PROCESS_LISTING.as_gas()
        );
        assert_eq!(GAS_FOR_DUAL_VERIFY, Gas::from_tgas(20));
    }

    // SEC-CONTRACT-002, INV-020
    #[test]
    fn test_new_sets_owner_and_storage_bounds() {
        let market = new_market();

        assert_eq!(market.own_get_owner(), Some(owner()));
        assert_eq!(market.storage_balance_bounds().min, storage_per_sale());
        assert_eq!(market.storage_balance_bounds().max, None);
        assert_eq!(market.get_supply_sales(), 0);
    }

    // INV-004 · SEC-CONTRACT-004 · docs/contracts/market.md §1
    #[test]
    fn test_new_defaults_fee_and_treasury() {
        let market = new_market();

        assert_eq!(market.get_fee_bps(), FEE_BPS_DEFAULT);
        assert_eq!(market.get_fee_bps(), 200);
        assert_eq!(
            market.get_treasury(),
            owner(),
            "default treasury = owner (§1)"
        );
    }

    // INV-004 · SEC-CONTRACT-004
    #[test]
    fn test_new_accepts_fee_at_cap_and_custom_treasury() {
        let market = new_market_with(Some(MAX_FEE_BPS), Some(treasury()));

        assert_eq!(market.get_fee_bps(), MAX_FEE_BPS);
        assert_eq!(market.get_fee_bps(), 500);
        assert_eq!(market.get_treasury(), treasury());
    }

    // INV-004 · SEC-CONTRACT-004
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_new_rejects_fee_above_cap() {
        let _ = new_market_with(Some(MAX_FEE_BPS + 1), None);
    }

    // INV-004 · SEC-CONTRACT-004
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_update_fee_bps_rejects_above_cap() {
        let mut market = new_market();

        run(&owner(), 1);
        market.update_fee_bps(MAX_FEE_BPS + 1);
    }

    // SEC-CONTRACT-004 · docs/api/webhooks.md §`fee_update`
    #[test]
    fn test_update_fee_bps_changes_value_and_emits_event() {
        let mut market = new_market();

        run(&owner(), 1);
        market.update_fee_bps(MAX_FEE_BPS);

        assert_eq!(market.get_fee_bps(), MAX_FEE_BPS);
        let logs = get_logs();
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""event":"fee_update""#)
                    && log.contains(r#""old_fee_bps":200"#)
                    && log.contains(r#""new_fee_bps":500"#)
                    && log.contains(r#""caller":"owner.testnet""#)),
            "event fee_update wajib memuat nilai lama & baru: {logs:?}"
        );
    }

    // SEC-CONTRACT-004 · SEC-CONTRACT-012 (owner-only MVP)
    #[test]
    #[should_panic]
    fn test_update_fee_bps_is_owner_only() {
        let mut market = new_market();

        run(&stranger(), 1);
        market.update_fee_bps(MAX_FEE_BPS);
    }

    // SEC-CONTRACT-001
    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_update_fee_bps_requires_one_yocto() {
        let mut market = new_market();

        run(&owner(), 0);
        market.update_fee_bps(MAX_FEE_BPS);
    }

    // docs/api/webhooks.md §`treasury_update`
    #[test]
    fn test_update_treasury_changes_value_and_emits_event() {
        let mut market = new_market();

        run(&owner(), 1);
        market.update_treasury(treasury());

        assert_eq!(market.get_treasury(), treasury());
        let logs = get_logs();
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""event":"treasury_update""#)
                    && log.contains(r#""old_treasury":"owner.testnet""#)
                    && log.contains(r#""new_treasury":"treasury.testnet""#)
                    && log.contains(r#""caller":"owner.testnet""#)),
            "event treasury_update wajib memuat alamat lama & baru: {logs:?}"
        );
    }

    // SEC-CONTRACT-012 (owner-only MVP)
    #[test]
    #[should_panic]
    fn test_update_treasury_is_owner_only() {
        let mut market = new_market();

        run(&stranger(), 1);
        market.update_treasury(treasury());
    }

    // SEC-CONTRACT-007
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

    // TC-002 · INV-013 · ADR-002
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

    // TC-002 · INV-013
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_list_rejected_when_caller_is_not_token_owner() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());
        let _ = list_token(&mut market, TOKEN, PRICE);

        // Koleksi melaporkan pemilik lain → klaim sepihak seller ditolak.
        verify_token(&mut market, TOKEN, PRICE, buyer(), Ok(true));
    }

    // TC-002 · INV-013
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_list_rejected_when_market_is_not_approved() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());
        let _ = list_token(&mut market, TOKEN, PRICE);

        verify_token(&mut market, TOKEN, PRICE, seller(), Ok(false));
    }

    // INV-013
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

    // INV-013
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

    // TC-013 · INV-030
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

    // TC-013 · INV-030
    #[test]
    #[should_panic(expected = "INVALID_PRICE")]
    fn test_list_rejects_price_below_minimum() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());

        let _ = list_token(&mut market, TOKEN, MIN_PRICE_YOCTO - 1);
    }

    // INV-007
    #[test]
    #[should_panic(expected = "CONFLICT_ALREADY_LISTED")]
    fn test_list_rejects_duplicate_listing() {
        let mut market = listed_market();

        let _ = list_token(&mut market, TOKEN, PRICE);
    }

    // TC-020 · INV-020
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_list_rejected_without_storage_deposit() {
        let mut market = new_market();

        let _ = list_token(&mut market, TOKEN, PRICE);
    }

    // TC-020 · INV-020
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

    // INV-013
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

    // TC-012 · INV-022
    #[test]
    #[should_panic(expected = "CHAIN_PAUSED")]
    fn test_list_rejected_when_paused() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());
        Pause::set_is_paused(&mut market, true);

        let _ = list_token(&mut market, TOKEN, PRICE);
    }

    // INV-026
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

    // TC-012 · INV-022
    #[test]
    #[should_panic(expected = "CHAIN_PAUSED")]
    fn test_process_listing_rejected_when_paused() {
        let mut market = new_market();
        fund_storage(&mut market, &seller());
        let _ = list_token(&mut market, TOKEN, PRICE);

        // Pause terjadi antara tx listing dan callback — callback adalah receipt terpisah, jadi ia
        // wajib menegakkan INV-022 sendiri.
        Pause::set_is_paused(&mut market, true);
        verify_token(&mut market, TOKEN, PRICE, seller(), Ok(true));
    }

    // --- nft_on_approve — NEP-178 receiver (INV-013, ADR-002) ---

    // INV-013 · ADR-002
    #[test]
    fn test_nft_on_approve_accepts_valid_payload() {
        let mut market = new_market();

        run(&nft(), 0);
        market.nft_on_approve(TOKEN.to_string(), seller(), 7, PRICE.to_string());

        assert_eq!(market.get_supply_sales(), 0);
    }

    // ADR-002
    #[test]
    fn test_nft_on_approve_does_not_create_listing() {
        let mut market = new_market();

        // Tx-1 mengirim msg harga — market **tidak** boleh membuat listing dari notifikasi ini.
        run(&nft(), 0);
        market.nft_on_approve(TOKEN.to_string(), seller(), 7, PRICE.to_string());

        assert!(
            market.get_sale(nft(), TOKEN.to_string()).is_none(),
            "nft_on_approve tidak boleh membuat listing (ADR-002)"
        );
    }

    // INV-013
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_nft_on_approve_rejects_empty_payload() {
        let mut market = new_market();

        run(&nft(), 0);
        market.nft_on_approve(TOKEN.to_string(), seller(), 7, String::new());
    }

    // INV-013
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_nft_on_approve_rejects_self_as_owner() {
        let mut market = new_market();

        run(&nft(), 0);
        market.nft_on_approve(TOKEN.to_string(), market_id(), 7, PRICE.to_string());
    }

    // --- remove_sale (TC-044, INV-012/022) ---

    // TC-044 · INV-012
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

    // TC-044 · INV-020
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

    // TC-044 · SEC-CONTRACT-001
    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_remove_sale_requires_one_yocto() {
        let mut market = listed_market();

        run(&seller(), 0);
        market.remove_sale(nft(), TOKEN.to_string());
    }

    // TC-044 · INV-012
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_remove_sale_rejected_for_non_owner() {
        let mut market = listed_market();

        run(&buyer(), 1);
        market.remove_sale(nft(), TOKEN.to_string());
    }

    // INV-012
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_remove_sale_rejected_for_unknown_listing() {
        let mut market = new_market();

        run(&seller(), 1);
        market.remove_sale(nft(), TOKEN.to_string());
    }

    // TC-012 · INV-022
    #[test]
    fn test_remove_sale_allowed_while_paused() {
        let mut market = listed_market();
        Pause::set_is_paused(&mut market, true);

        run(&seller(), 1);
        market.remove_sale(nft(), TOKEN.to_string());

        assert!(market.get_sale(nft(), TOKEN.to_string()).is_none());
    }

    // --- update_price (INV-030, INV-012) ---

    // INV-012
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

    // INV-030
    #[test]
    #[should_panic(expected = "INVALID_PRICE")]
    fn test_update_price_rejects_below_minimum() {
        let mut market = listed_market();

        run(&seller(), 1);
        market.update_price(nft(), TOKEN.to_string(), U128(MIN_PRICE_YOCTO - 1));
    }

    // INV-012
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_update_price_rejected_for_non_owner() {
        let mut market = listed_market();

        run(&buyer(), 1);
        market.update_price(nft(), TOKEN.to_string(), U128(PRICE * 2));
    }

    // TC-012 · INV-022
    #[test]
    #[should_panic(expected = "CHAIN_PAUSED")]
    fn test_update_price_rejected_when_paused() {
        let mut market = listed_market();
        Pause::set_is_paused(&mut market, true);

        run(&seller(), 1);
        market.update_price(nft(), TOKEN.to_string(), U128(PRICE * 2));
    }

    // --- view listing (§5) ---

    // market.md §5
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

    // --- buy: happy path (TC-002 paruh buy, INV-001/002/003/011/014/031) ---

    // TC-002 · INV-001/002/003/031 · docs/contracts/market.md §2
    #[test]
    fn test_buy_settles_and_distributes_fee_royalty_and_proceeds() {
        let mut market = listed_market();
        let royalty = MAX_PAY / 10;

        let promise = buy(&mut market, PRICE);
        drop(promise);

        // Optimistic removal + entri pending ditulis SEBELUM XCC (INV-031).
        assert!(
            sale_of(&market).is_none(),
            "Sale dihapus optimistic saat buy"
        );
        let pending = pending_of(&market).expect("entri pending_purchases ditulis saat buy");
        assert_eq!(pending.buyer, buyer());
        assert_eq!(pending.deposit, PRICE);
        assert_eq!(pending.sale.owner_id, seller());

        // Jalur buy = dual verification (2 view) → transfer payout → callback settle.
        let methods = created_method_names();
        assert!(
            methods.contains(&"nft_token".to_string())
                && methods.contains(&"nft_is_approved".to_string()),
            "buy wajib dual verification sebelum transfer: {methods:?}"
        );
        assert!(
            !methods.contains(&"nft_transfer_payout".to_string()),
            "transfer baru boleh setelah verifikasi lolos: {methods:?}"
        );

        let promise = verify_purchase(&mut market, seller(), Ok(true));
        drop(promise);

        let methods = created_method_names();
        assert!(
            methods.contains(&"nft_transfer_payout".to_string()),
            "verifikasi lolos → nft_transfer_payout (NEP-199): {methods:?}"
        );

        // Payout dari koleksi = royalti kreator (UNTRUSTED, divalidasi market).
        resolve(&mut market, Ok(royalty_payout(royalty)));

        // INV-001/002: fee + Σroyalti + seller == harga persis; sisa jatuh ke seller.
        assert_eq!(transferred_to(&treasury()), FEE, "fee 2% → treasury");
        assert_eq!(transferred_to(&creator()), royalty, "royalti → kreator");
        assert_eq!(
            transferred_to(&seller()),
            MAX_PAY - royalty,
            "seller menerima residual (plafon − royalti)"
        );
        assert_eq!(
            transferred_to(&treasury()) + transferred_to(&creator()) + transferred_to(&seller()),
            PRICE,
            "Σ keluar == Σ masuk (INV-001)"
        );
        assert_eq!(
            transferred_to(&buyer()),
            0,
            "deposit pas harga → tidak ada refund"
        );

        // Entri pending dihapus setelah settle (INV-031 jalur keluar pertama).
        assert!(pending_of(&market).is_none());
        assert!(sale_of(&market).is_none(), "listing tetap terjual (SOLD)");

        let logs = get_logs();
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""event":"market_sale""#)
                    && log.contains(r#""buyer":"bob.testnet""#)
                    && log.contains(r#""seller":"alice.testnet""#)
                    && log.contains(&format!(r#""price_yocto":"{PRICE}""#))
                    && log.contains(&format!(r#""{}":"{royalty}""#, creator()))
                    && log.contains(&format!(r#""{}":"{}""#, seller(), MAX_PAY - royalty))),
            "event market_sale wajib memuat payout map final: {logs:?}"
        );
    }

    // TC-022 · INV-001 · docs/features/payments.md §Refund Path
    #[test]
    fn test_buy_refunds_excess_deposit_to_buyer() {
        let mut market = listed_market();
        let excess = 500_000_000_000_000_000_000_000;
        let royalty = MAX_PAY / 10;

        let promise = buy(&mut market, PRICE + excess);
        drop(promise);
        let promise = verify_purchase(&mut market, seller(), Ok(true));
        drop(promise);
        resolve(&mut market, Ok(royalty_payout(royalty)));

        assert_eq!(
            transferred_to(&buyer()),
            excess,
            "kelebihan deposit kembali ke buyer (harga berubah saat signing)"
        );
        assert_eq!(
            transferred_to(&treasury())
                + transferred_to(&creator())
                + transferred_to(&seller())
                + transferred_to(&buyer()),
            PRICE + excess,
            "Σ keluar == Σ masuk termasuk refund (INV-001)"
        );
    }

    // INV-014 · SEC-ORDER-002
    #[test]
    fn test_buy_transfers_only_to_state_derived_accounts() {
        let mut market = listed_market();

        let promise = buy(&mut market, PRICE);
        drop(promise);
        let promise = verify_purchase(&mut market, seller(), Ok(true));
        drop(promise);
        resolve(&mut market, Ok(royalty_payout(MAX_PAY / 10)));

        // Tujuan transfer hanya turunan state: treasury + payout + seller (+ buyer saat refund).
        let allowed = [treasury(), creator(), seller(), buyer(), market_id()];
        for (receiver, _) in transfers() {
            assert!(
                allowed.contains(&receiver),
                "transfer ke alamat non-state: {receiver}"
            );
        }
    }

    // INV-011 · SEC-ORDER-004 · docs/contracts/market.md §6
    #[test]
    fn test_buy_attaches_one_yocto_and_payout_cap_to_transfer() {
        let mut market = listed_market();

        let promise = buy(&mut market, PRICE);
        drop(promise);
        let promise = verify_purchase(&mut market, seller(), Ok(true));
        drop(promise);

        let calls: Vec<(String, u128)> = get_created_receipts()
            .iter()
            .flat_map(|receipt| {
                receipt.actions.iter().filter_map(|action| match action {
                    MockAction::FunctionCallWeight {
                        method_name,
                        attached_deposit,
                        ..
                    } => Some((
                        String::from_utf8(method_name.clone()).expect("utf8 method name"),
                        attached_deposit.as_yoctonear(),
                    )),
                    _ => None,
                })
            })
            .collect();
        let (_, deposit) = calls
            .iter()
            .find(|(method, _)| method == "nft_transfer_payout")
            .expect("nft_transfer_payout dipanggil");

        assert_eq!(
            *deposit, 1,
            "NEP-199 dipanggil dengan attach 1 yocto (INV-011)"
        );
    }

    // INV-002 · SEC-CONTRACT-005
    #[test]
    fn test_buy_accepts_one_yocto_dust_remainder() {
        let mut market = listed_market();
        // Σpayout = plafon − 1 → sisa 1 yocto diterima (batas inklusif INV-002), dust → seller.
        let royalty = MAX_PAY - 1;

        let promise = buy(&mut market, PRICE);
        drop(promise);
        let promise = verify_purchase(&mut market, seller(), Ok(true));
        drop(promise);
        resolve(&mut market, Ok(royalty_payout(royalty)));

        assert_eq!(transferred_to(&creator()), royalty);
        assert_eq!(transferred_to(&seller()), 1, "sisa 1 yocto jatuh ke seller");
        assert!(
            pending_of(&market).is_none(),
            "dust tetap menyelesaikan pembelian"
        );
    }

    // --- buy: penolakan (INV-023/026, CONFLICT_SOLD/STALE, deposit) ---

    // TC-002 · INV-023
    #[test]
    #[should_panic(expected = "FORBIDDEN_SELF_BUY")]
    fn test_buy_rejected_for_self_buy() {
        let mut market = listed_market();

        let _ = buy_as(&mut market, &seller(), PRICE);
    }

    // TC-011 · INV-026
    #[test]
    #[should_panic(expected = "FORBIDDEN_BUYER")]
    fn test_buy_rejected_for_non_allowed_buyer_on_private_listing() {
        let mut market = listed_market_for(buyer());

        let _ = buy_as(&mut market, &stranger(), PRICE);
    }

    // TC-011 · INV-026
    #[test]
    fn test_buy_allowed_buyer_settles_private_listing() {
        let mut market = listed_market_for(buyer());
        let royalty = MAX_PAY / 10;

        let promise = buy_as(&mut market, &buyer(), PRICE);
        drop(promise);
        let promise = verify_purchase(&mut market, seller(), Ok(true));
        drop(promise);
        resolve(&mut market, Ok(royalty_payout(royalty)));

        assert_eq!(transferred_to(&seller()), MAX_PAY - royalty);
        assert_eq!(transferred_to(&treasury()), FEE);
    }

    // TC-022 · INV-001
    #[test]
    #[should_panic(expected = "CHAIN_INSUFFICIENT_DEPOSIT")]
    fn test_buy_rejected_when_deposit_below_price() {
        let mut market = listed_market();

        let _ = buy(&mut market, PRICE - 1);
    }

    // TC-016/017 · INV-007/008 · SEC-ORDER-005/006
    #[test]
    #[should_panic(expected = "CONFLICT_SOLD")]
    fn test_buy_rejected_when_sale_already_removed() {
        let mut market = listed_market();

        let promise = buy(&mut market, PRICE);
        drop(promise);

        // Pembeli kedua menyerbu listing yang sale-nya sudah dihapus optimistic.
        let _ = buy_as(&mut market, &stranger(), PRICE);
    }

    // INV-031 · §3b
    #[test]
    #[should_panic(expected = "CONFLICT_SOLD")]
    fn test_buy_rejected_while_another_purchase_pending() {
        let mut market = listed_market();

        let promise = buy(&mut market, PRICE);
        drop(promise);

        // Entry `Sale` dipulihkan (mis. callback settle belum selesai) tapi pending masih hidup:
        // pembeli kedua tidak boleh menumpuk deposit di atas pembelian yang sedang berjalan.
        let sale = pending_of(&market).expect("pending ada").sale;
        assert!(market.restore_sale(&(nft(), TOKEN.to_string()), &sale));

        let _ = buy_as(&mut market, &stranger(), PRICE);
    }

    // --- buy: stale (dua kasus, INV-016 §3a) ---

    // TC-006 · INV-016 kasus A (ownership mismatch)
    #[test]
    fn test_buy_refunds_and_marks_stale_when_ownership_moved() {
        let mut market = listed_market();

        let promise = buy(&mut market, PRICE);
        drop(promise);

        // Token sudah pindah ke orang lain di luar market → stale kasus A.
        let promise = verify_purchase(&mut market, stranger(), Ok(true));
        drop(promise);

        assert_eq!(transferred_to(&buyer()), PRICE, "refund penuh buyer");
        assert!(
            pending_of(&market).is_none(),
            "entry pending dihapus pada jalur stale"
        );
        assert!(
            sale_of(&market).is_none(),
            "listing mati — tidak dipulihkan saat stale"
        );

        let logs = get_logs();
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""event":"market_stale_detected""#)
                    && log.contains(r#""reason":"ownership_mismatch""#)
                    && log.contains(r#""detected_owner":"carol.testnet""#)),
            "event market_stale_detected (ownership_mismatch) wajib ter-emit: {logs:?}"
        );
    }

    // TC-053 · INV-016 kasus B (approval dicabut)
    #[test]
    fn test_buy_refunds_and_marks_stale_when_approval_revoked() {
        let mut market = listed_market();

        let promise = buy(&mut market, PRICE);
        drop(promise);

        // Token tetap milik seller, tapi approval sudah dicabut → stale kasus B.
        let promise = verify_purchase(&mut market, seller(), Ok(false));
        drop(promise);

        assert_eq!(transferred_to(&buyer()), PRICE, "refund penuh buyer");
        assert!(sale_of(&market).is_none(), "listing mati saat stale");
        assert!(pending_of(&market).is_none());

        let logs = get_logs();
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""event":"market_stale_detected""#)
                    && log.contains(r#""reason":"approval_revoked""#)),
            "event market_stale_detected (approval_revoked) wajib ter-emit: {logs:?}"
        );
    }

    // INV-016 · §3a
    #[test]
    fn test_buy_restores_sale_when_verification_is_inconclusive() {
        let mut market = listed_market();

        let promise = buy(&mut market, PRICE);
        drop(promise);

        // Koleksi tidak bisa dihubungi → kegagalan BUKAN stale → listing dipulihkan.
        let promise = verify_purchase(&mut market, seller(), Err(PromiseError::Failed));
        drop(promise);

        assert_eq!(transferred_to(&buyer()), PRICE, "refund penuh buyer");
        assert!(pending_of(&market).is_none());
        let sale = sale_of(&market).expect("listing dipulihkan (bukan stale)");
        assert_eq!(sale.owner_id, seller());
        assert_eq!(sale.price_yocto, U128(PRICE));
        assert_eq!(market.get_supply_sales(), 1);
    }

    // INV-013
    #[test]
    fn test_process_purchase_rejected_without_pending() {
        let mut market = listed_market();

        // Tanpa `buy` lebih dulu, tidak ada konteks pembelian → callback menolak (INV-013).
        run(&nft(), 0);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = market.process_purchase(
                Ok(Some(token_owned_by(TOKEN, seller()))),
                Ok(true),
                nft(),
                TOKEN.to_string(),
            );
        }));
        assert!(result.is_err(), "callback tanpa entri pending wajib panic");
    }

    // --- resolve_purchase: validasi payout UNTRUSTED (INV-002/003, SEC-CONTRACT-005) ---

    /// Jalankan `buy` + `process_purchase` sampai titik `resolve_purchase`.
    fn market_at_resolve() -> Market {
        let mut market = listed_market();
        let promise = buy(&mut market, PRICE);
        drop(promise);
        let promise = verify_purchase(&mut market, seller(), Ok(true));
        drop(promise);
        market
    }

    // TC-003 · INV-002 · SEC-ORDER-001
    #[test]
    fn test_resolve_refunds_when_payout_exceeds_price_minus_fee() {
        let mut market = market_at_resolve();

        resolve(&mut market, Ok(royalty_payout(MAX_PAY + 1)));

        assert_eq!(
            transferred_to(&buyer()),
            PRICE,
            "payout invalid → refund penuh"
        );
        assert_eq!(transferred_to(&treasury()), 0, "tidak ada fee saat gagal");
        assert_eq!(
            transferred_to(&seller()),
            0,
            "tidak ada distribusi saat gagal"
        );
        let sale = sale_of(&market).expect("listing dipulihkan saat payout invalid");
        assert_eq!(sale.owner_id, seller());
        assert!(pending_of(&market).is_none());
    }

    // INV-002 · SEC-CONTRACT-005 · docs/features/payments.md §Aturan pembulatan
    #[test]
    fn test_resolve_accepts_small_royalty_and_pays_residual_to_seller() {
        let mut market = market_at_resolve();
        // Bentuk payout yang sebenarnya dikembalikan koleksi NearSea: royalti saja, jauh di
        // bawah plafon. Sisa (plafon − royalti) = proceeds seller — bukan dust, bukan penolakan.
        let royalty = PRICE * 5 / 1_000;

        resolve(&mut market, Ok(royalty_payout(royalty)));

        assert_eq!(transferred_to(&creator()), royalty);
        assert_eq!(
            transferred_to(&seller()),
            MAX_PAY - royalty,
            "seller menerima residual besar — aritmetika internal tetap exact"
        );
        assert_eq!(transferred_to(&treasury()), FEE);
        assert_eq!(
            transferred_to(&treasury()) + transferred_to(&creator()) + transferred_to(&seller()),
            PRICE,
            "fee + royalti + seller == harga persis (INV-001)"
        );
    }

    // INV-003 · SEC-CONTRACT-005
    #[test]
    fn test_resolve_refunds_on_empty_payout() {
        let mut market = market_at_resolve();

        resolve(&mut market, Ok(Payout::new()));

        assert_eq!(transferred_to(&buyer()), PRICE);
        assert!(sale_of(&market).is_some());
    }

    // INV-003 · SEC-CONTRACT-005
    #[test]
    fn test_resolve_refunds_on_zero_amount_receiver() {
        let mut market = market_at_resolve();

        resolve(&mut market, Ok(royalty_payout(0)));

        assert_eq!(
            transferred_to(&buyer()),
            PRICE,
            "amount 0 ditolak (INV-003)"
        );
        assert!(sale_of(&market).is_some());
    }

    // INV-003/021 · SEC-CONTRACT-010
    #[test]
    fn test_resolve_refunds_when_receivers_above_cap() {
        let mut market = market_at_resolve();
        let mut payout = Payout::new();
        for index in 0..=MAX_PAYOUT_RECEIVERS {
            payout.insert(format!("receiver{index}.testnet").parse().unwrap(), U128(1));
        }

        resolve(&mut market, Ok(payout));

        assert_eq!(
            transferred_to(&buyer()),
            PRICE,
            "> 10 penerima ditolak (INV-003)"
        );
        assert!(sale_of(&market).is_some());
    }

    // INV-013 · SEC-ORDER-001
    #[test]
    fn test_resolve_refunds_when_transfer_promise_failed() {
        let mut market = market_at_resolve();

        resolve(&mut market, Err(PromiseError::Failed));

        assert_eq!(
            transferred_to(&buyer()),
            PRICE,
            "promise gagal → refund penuh"
        );
        assert!(
            sale_of(&market).is_some(),
            "listing dipulihkan (kegagalan bukan stale)"
        );
        assert!(pending_of(&market).is_none());
    }

    // INV-003 · SEC-CONTRACT-005
    #[test]
    fn test_resolve_accepts_multiple_receivers() {
        let mut market = market_at_resolve();
        let creator_cut = MAX_PAY / 20;
        let second_cut = MAX_PAY / 40;
        let mut payout = Payout::new();
        payout.insert(creator(), U128(creator_cut));
        payout.insert(stranger(), U128(second_cut));

        resolve(&mut market, Ok(payout));

        assert_eq!(transferred_to(&creator()), creator_cut);
        assert_eq!(transferred_to(&stranger()), second_cut);
        assert_eq!(
            transferred_to(&seller()),
            MAX_PAY - creator_cut - second_cut,
            "seller = residual setelah semua royalti"
        );
    }

    // --- recover_stuck_purchase (TC-054, INV-031) ---

    /// Market dengan pembelian nyangkut: `buy` jalan, callback settle **tidak** (disimulasikan
    /// dengan tidak memanggil `resolve_purchase`).
    fn market_with_stuck_purchase() -> Market {
        let mut market = listed_market();
        let promise = buy(&mut market, PRICE);
        drop(promise);
        market
    }

    // TC-054 · INV-031
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_recover_rejected_before_delay() {
        let mut market = market_with_stuck_purchase();

        run_at(&stranger(), 1, 0);
        let _ = market.recover_stuck_purchase(nft(), TOKEN.to_string());
    }

    // TC-054 · INV-031
    #[test]
    fn test_recover_refunds_and_restores_after_delay_by_third_party() {
        let mut market = market_with_stuck_purchase();

        // Akun ketiga (bukan buyer/seller) memicu pemulihan setelah jeda.
        run_at(&stranger(), 1, RECOVERY_DELAY_BLOCKS + 1);
        let promise = market.recover_stuck_purchase(nft(), TOKEN.to_string());
        drop(promise);

        verify_recovery(&mut market, seller(), Ok(true));

        assert_eq!(transferred_to(&buyer()), PRICE, "refund penuh ke buyer");
        assert!(pending_of(&market).is_none(), "entry pending dihapus");
        let sale = sale_of(&market).expect("listing dipulihkan");
        assert_eq!(sale.owner_id, seller());

        let logs = get_logs();
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""event":"market_purchase_recovered""#)
                    && log.contains(r#""buyer":"bob.testnet""#)
                    && log.contains(&format!(r#""refund_yocto":"{PRICE}""#))
                    && log.contains(r#""sale_restored":true"#)),
            "event market_purchase_recovered wajib ter-emit: {logs:?}"
        );
    }

    // INV-031 · §3a
    #[test]
    fn test_recover_refunds_without_restoring_when_token_moved() {
        let mut market = market_with_stuck_purchase();

        run_at(&stranger(), 1, RECOVERY_DELAY_BLOCKS + 1);
        let promise = market.recover_stuck_purchase(nft(), TOKEN.to_string());
        drop(promise);

        // Token sudah pindah → jangan pulihkan listing "zombie"; dana tetap kembali penuh.
        verify_recovery(&mut market, stranger(), Ok(true));

        assert_eq!(transferred_to(&buyer()), PRICE);
        assert!(
            sale_of(&market).is_none(),
            "listing tidak dipulihkan bila token pindah"
        );
        assert!(pending_of(&market).is_none());

        let logs = get_logs();
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""event":"market_purchase_recovered""#)
                    && log.contains(r#""sale_restored":false"#)),
            "sale_restored:false saat token tidak lagi milik seller: {logs:?}"
        );
    }

    // TC-054 · SEC-CONTRACT-001
    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_recover_requires_one_yocto() {
        let mut market = market_with_stuck_purchase();

        run_at(&stranger(), 0, RECOVERY_DELAY_BLOCKS + 1);
        let _ = market.recover_stuck_purchase(nft(), TOKEN.to_string());
    }

    // INV-031
    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_recover_rejected_without_pending() {
        let mut market = listed_market();

        run_at(&stranger(), 1, RECOVERY_DELAY_BLOCKS + 1);
        let _ = market.recover_stuck_purchase(nft(), TOKEN.to_string());
    }

    // --- paused & anggaran gas (INV-022, §3b) ---

    // TC-012 · INV-022
    #[test]
    #[should_panic(expected = "CHAIN_PAUSED")]
    fn test_buy_rejected_when_paused() {
        let mut market = listed_market();
        Pause::set_is_paused(&mut market, true);

        let _ = buy(&mut market, PRICE);
    }

    // TC-012 · INV-022
    #[test]
    fn test_recover_allowed_while_paused() {
        let mut market = market_with_stuck_purchase();
        Pause::set_is_paused(&mut market, true);

        // Refund adalah jalur pengembalian aset — tetap terbuka saat paused (INV-022).
        run_at(&stranger(), 1, RECOVERY_DELAY_BLOCKS + 1);
        let promise = market.recover_stuck_purchase(nft(), TOKEN.to_string());
        drop(promise);
        verify_recovery(&mut market, seller(), Ok(true));

        assert_eq!(transferred_to(&buyer()), PRICE);
    }

    // TC-012 · INV-022 — callback = receipt terpisah, jadi pause ditegakkan di sana juga
    #[test]
    fn test_process_purchase_aborts_when_paused_between_buy_and_callback() {
        let mut market = listed_market();

        let promise = buy(&mut market, PRICE);
        drop(promise);

        // Pause terjadi setelah `buy` tapi sebelum callback settle selesai. NFT **tidak boleh**
        // pindah; pembelian dibatalkan tanpa biaya — pending dihapus, listing dipulihkan, refund.
        Pause::set_is_paused(&mut market, true);
        let promise = verify_purchase(&mut market, seller(), Ok(true));
        drop(promise);

        assert_eq!(transferred_to(&buyer()), PRICE, "refund penuh buyer");
        assert!(pending_of(&market).is_none(), "pending dihapus");
        assert!(
            sale_of(&market).is_some(),
            "listing dipulihkan — pembelian batal karena paused"
        );
        assert_eq!(
            transferred_to(&treasury()) + transferred_to(&seller()),
            0,
            "tidak ada distribusi saat paused"
        );
    }

    // docs/contracts/market.md §3 — split ditentukan syarat saat `buy`, bukan state saat settle
    #[test]
    fn test_resolve_uses_fee_bps_snapshot_from_buy() {
        let mut market = listed_market();
        let royalty = MAX_PAY / 10;

        let promise = buy(&mut market, PRICE);
        drop(promise);
        let promise = verify_purchase(&mut market, seller(), Ok(true));
        drop(promise);

        // Owner menaikkan fee setelah buyer menandatangani; split tetap memakai fee saat buy (2%).
        run(&owner(), 1);
        market.update_fee_bps(MAX_FEE_BPS);
        resolve(&mut market, Ok(royalty_payout(royalty)));

        assert_eq!(
            transferred_to(&treasury()),
            FEE,
            "fee yang berlaku = snapshot saat buy (2%), bukan 5%"
        );
        assert_eq!(transferred_to(&seller()), MAX_PAY - royalty);
    }

    // docs/contracts/market.md §3b — anggaran gas resolve_purchase vs worst case
    #[test]
    fn test_resolve_purchase_gas_within_budget() {
        let mut market = market_at_resolve();
        let mut payout = Payout::new();
        for index in 0..MAX_PAYOUT_RECEIVERS {
            payout.insert(format!("receiver{index}.testnet").parse().unwrap(), U128(1));
        }

        // Worst case: 10 penerima + fee + seller + refund + event, di bawah anggaran 115 Tgas.
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(market_id())
            .predecessor_account_id(nft())
            .block_timestamp(T0)
            .prepaid_gas(GAS_FOR_RESOLVE_PURCHASE);
        testing_env!(builder.build());

        market.resolve_purchase(Ok(payout), nft(), TOKEN.to_string());

        assert!(
            env::used_gas() < GAS_FOR_RESOLVE_PURCHASE,
            "resolve_purchase wajib muat di anggaran 115 Tgas, terpakai {:?}",
            env::used_gas()
        );
    }
}
