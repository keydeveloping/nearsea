//! NearSea NFT Collection — NEP-171/177/178/181/199/297 + state launchpad.
//!
//! Reference implementasi: docs/contracts/nft-collection.md §1–§7.
//! Fase bebas penuh (allowlist penuh, Pausable, MAX_FEE_BPS) menyusul di TASK-020;
//! `mint_price_of` (basis royalti bundle) di TASK-010.
//!
//! Prefix storage kanonik: docs/contracts/nft-collection.md §7 (SEC-CONTRACT-008).

use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::{assert_one_yocto, env, near, AccountId, BorshStorageKey, PanicOnDefault};
use near_sdk_contract_tools::nft::*;
use near_sdk_contract_tools::owner::Owner;
use near_sdk_contract_tools::standard::nep297::{Event, EventLog, ToEventLog};
use near_sdk_contract_tools::Owner as OwnerDerive;
use std::borrow::Cow;
use std::collections::HashMap;

/// Cap royalti per token 10% (INV-027).
pub const MAX_ROYALTY_BPS: u16 = 1000;
/// Royalti minimum 1 bps — payout settlement wajib ≥1 penerima ber-amount >0 (INV-003).
pub const MIN_ROYALTY_BPS: u16 = 1;
/// Batas statis loop mint (INV-021). PROPOSED — diukur di sandbox.
pub const MAX_MINT_PER_CALL: u32 = 10;
/// Batas statis jumlah fase (INV-021). PROPOSED — diukur di sandbox.
pub const MAX_PHASES: u32 = 20;
/// Batas statis baris allowlist per call (INV-021). PROPOSED — diukur di sandbox.
pub const MAX_ALLOWLIST_BATCH: u32 = 200;
/// Saldo storage minimum NEP-145 (byte × `storage_byte_cost`) — INV-020.
pub const STORAGE_MIN_BYTES: u64 = 300;

/// Teks panic kontrak = kode registry, supaya FE memetakan ke kode user yang stabil
/// (docs/development/error-handling.md §3/§4, docs/security/smart-contract-security-architecture.md §15).
mod err {
    pub const CHAIN_REVERT: &str = "CHAIN_REVERT";
    pub const INVALID_PRICE: &str = "INVALID_PRICE";
    pub const INVALID_ROYALTY: &str = "INVALID_ROYALTY";
    pub const CONFLICT_PHASE_OVERLAP: &str = "CONFLICT_PHASE_OVERLAP";
    pub const LAUNCHPAD_PHASE_INACTIVE: &str = "LAUNCHPAD_PHASE_INACTIVE";
    pub const LAUNCHPAD_NOT_ALLOWED: &str = "LAUNCHPAD_NOT_ALLOWED";
    pub const LAUNCHPAD_ALLOCATION_EXHAUSTED: &str = "LAUNCHPAD_ALLOCATION_EXHAUSTED";
    pub const LAUNCHPAD_MAX_PER_WALLET: &str = "LAUNCHPAD_MAX_PER_WALLET";
    pub const LAUNCHPAD_PRICE_MISMATCH: &str = "LAUNCHPAD_PRICE_MISMATCH";
}

/// Prefix storage milik kontrak ini — bukan `~*` (milik derive near-sdk-contract-tools).
#[derive(BorshStorageKey, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    Phases,
    Allowlist,
    MintedByWallet,
}

/// Konfigurasi fase yang dikirim creator (`set_phases`).
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseConfig {
    pub name: String,
    pub price_yocto: U128,
    pub allocation: u32,
    pub max_per_wallet: u32,
    pub allowlist_required: bool,
    pub starts_at: u64,
    pub ends_at: u64,
}

/// Fase tersimpan — `PhaseConfig` + sisa alokasi (INV-017).
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase {
    pub name: String,
    pub price_yocto: U128,
    pub allocation: u32,
    pub alloc_left: u32,
    pub max_per_wallet: u32,
    pub allowlist_required: bool,
    pub starts_at: u64,
    pub ends_at: u64,
}

/// Satu baris allowlist (CSV `account_id[, max_mints]`).
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllowlistEntry {
    pub account_id: AccountId,
    /// Override batas per wallet; efektif = min(`max_per_wallet` fase, nilai ini).
    pub max_mints: Option<u32>,
}

/// Status fase — derivasi dari `env::block_timestamp()`.
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PhaseState {
    Scheduled,
    Active,
    Ended,
}

/// Fase untuk tampilan FE (`get_launchpad`).
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseStatus {
    pub index: u16,
    pub name: String,
    pub price_yocto: U128,
    pub allocation: u32,
    pub alloc_left: u32,
    pub max_per_wallet: u32,
    pub allowlist_required: bool,
    pub starts_at: u64,
    pub ends_at: u64,
    pub status: PhaseState,
}

/// Bentuk `get_launchpad()` (docs/contracts/nft-collection.md §5).
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchpadStatus {
    pub phases: Vec<PhaseStatus>,
    pub active_phase_index: Option<u16>,
}

/// Konfigurasi royalti untuk market (INV-025 pre-validation).
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoyaltyConfig {
    pub receiver: AccountId,
    pub bps: u16,
}

/// Payout NEP-199 — map penerima → jumlah yoctoNEAR (docs/contracts/nft-collection.md §2/§4).
/// Market memperlakukan nilai ini sebagai **UNTRUSTED** dan memvalidasinya sendiri.
pub type Payout = HashMap<AccountId, U128>;

/// Envelope event NearSea — `data` selalu array satu entri (docs/api/webhooks.md §Skema payload).
/// Dipakai semua event custom `x-nearsea-market` (koleksi ini; market/factory menyusul).
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

/// Isi `data[0]` event `launchpad_mint` (docs/api/webhooks.md).
#[near(serializers = [json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchpadMint {
    pub collection: AccountId,
    pub phase_index: u16,
    pub account_id: AccountId,
    pub token_ids: Vec<String>,
    pub price_yocto: U128,
}

/// Window fase `[starts_at, ends_at)` — `starts_at` inklusif, `ends_at` eksklusif
/// (docs/contracts/nft-collection.md §3).
fn is_active(phase: &Phase, now: u64) -> bool {
    phase.starts_at <= now && now < phase.ends_at
}

/// Royalti satu token = `floor(balance × bps / 10_000)` (docs/contracts/nft-collection.md §4).
/// INV-027: `bps ≤ MAX_ROYALTY_BPS` dijaga saat init, jadi hasil selalu ≤10% `balance`.
fn royalty_amount(balance: u128, bps: u16) -> u128 {
    balance
        .checked_mul(u128::from(bps))
        .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT))
        / 10_000
}

#[near(contract_state)]
#[derive(PanicOnDefault, OwnerDerive, NonFungibleToken)]
pub struct NftCollection {
    creator_id: AccountId,
    royalty_bps: u16,
    market_id: Option<AccountId>,
    phases: Vector<Phase>,
    allowlist: LookupMap<(u16, AccountId), Option<u32>>,
    minted_by_wallet: LookupMap<(u16, AccountId), u32>,
    next_token_id: u64,
}

#[near]
impl NftCollection {
    /// Init satu-satunya (PanicOnDefault → memanggil tanpa init = panic) — SEC-CONTRACT-002.
    /// Argumen positional = ABI kontrak yang dipatok docs/contracts/nft-collection.md §1
    /// (factory TASK-012 memanggilnya), jadi tidak digabung ke struct.
    #[init]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner_id: AccountId,
        creator_id: AccountId,
        royalty_bps: u16,
        name: String,
        symbol: String,
        base_uri: Option<String>,
        market_id: Option<AccountId>,
    ) -> Self {
        assert!(
            (MIN_ROYALTY_BPS..=MAX_ROYALTY_BPS).contains(&royalty_bps),
            "{}",
            err::INVALID_ROYALTY,
        );

        let mut contract = Self {
            creator_id,
            royalty_bps,
            market_id,
            phases: Vector::new(StorageKey::Phases),
            allowlist: LookupMap::new(StorageKey::Allowlist),
            minted_by_wallet: LookupMap::new(StorageKey::MintedByWallet),
            next_token_id: 0,
        };

        Owner::init(&mut contract, &owner_id);
        contract.set_contract_metadata(&ContractMetadata::new(name, symbol, base_uri));
        Nep145Controller::set_storage_balance_bounds(
            &mut contract,
            &StorageBalanceBounds {
                min: env::storage_byte_cost().saturating_mul(u128::from(STORAGE_MIN_BYTES)),
                max: None,
            },
        );

        contract
    }

    /// Mint mengikuti fase aktif. Deposit EXACT = `phase.price × quantity` (INV-018).
    /// Storage mint dibayar pemicu mint via pre-deposit NEP-145 (INV-019).
    #[payable]
    pub fn nft_mint(&mut self, phase_index: Option<u16>, quantity: u32) {
        let minter = env::predecessor_account_id();
        let attached = env::attached_deposit().as_yoctonear();

        // 1 — fase aktif tunggal (INV-017/INV-029)
        let index = self.resolve_active_phase(phase_index);

        // 2 — batas statis per call (INV-021)
        if quantity == 0 || quantity > MAX_MINT_PER_CALL {
            env::panic_str(err::CHAIN_REVERT);
        }

        let mut phase = self
            .phases
            .get(u64::from(index))
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));

        // 3 — allowlist fase (INV-017); batas efektif per wallet = min(fase, baris allowlist)
        let wallet_cap = if phase.allowlist_required {
            match self.allowlist.get(&(index, minter.clone())) {
                Some(entry) => entry.unwrap_or(phase.max_per_wallet),
                None => env::panic_str(err::LAUNCHPAD_NOT_ALLOWED),
            }
        } else {
            phase.max_per_wallet
        };

        // 4 — sisa alokasi fase (INV-017)
        if phase.alloc_left < quantity {
            env::panic_str(err::LAUNCHPAD_ALLOCATION_EXHAUSTED);
        }

        // 5 — batas mint per wallet (INV-017)
        let minted = self
            .minted_by_wallet
            .get(&(index, minter.clone()))
            .unwrap_or(0);
        if minted + quantity > wallet_cap {
            env::panic_str(err::LAUNCHPAD_MAX_PER_WALLET);
        }

        // 6 — deposit exact-match, bukan ≥ (INV-018)
        let price = phase
            .price_yocto
            .0
            .checked_mul(u128::from(quantity))
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
        if attached != price {
            env::panic_str(err::LAUNCHPAD_PRICE_MISMATCH);
        }

        // 7 — minter wajib sudah punya saldo storage NEP-145 (INV-019)
        if self.get_storage_balance(&minter).is_err() {
            env::panic_str(err::CHAIN_REVERT);
        }

        // 8 — mint berurutan `0`,`1`,… ; tiap token menagih storage ke minter via hook NEP-145
        let mut token_ids = Vec::with_capacity(quantity as usize);
        for _ in 0..quantity {
            let token_id = self.next_token_id.to_string();
            self.next_token_id += 1;

            self.mint_with_metadata(
                &token_id,
                &minter,
                &TokenMetadata::new().title(format!("#{token_id}")),
            )
            .unwrap_or_else(|e| env::panic_str(&format!("{}: {e}", err::CHAIN_REVERT)));

            token_ids.push(token_id);
        }

        phase.alloc_left -= quantity;
        self.phases.replace(u64::from(index), &phase);
        self.minted_by_wallet
            .insert(&(index, minter.clone()), &(minted + quantity));

        SingleEvent::new(
            "launchpad_mint",
            LaunchpadMint {
                collection: env::current_account_id(),
                phase_index: index,
                account_id: minter,
                token_ids,
                price_yocto: U128(price),
            },
        )
        .emit();
    }

    /// NEP-199: pindahkan token ke `receiver_id` (otorisasi market lewat approval NEP-178)
    /// sekaligus kembalikan payout royalti untuk `balance` (harga wajar token).
    /// Market memvalidasi payout ini sendiri — nilainya UNTRUSTED
    /// (docs/features/payments.md §Algoritma Distribusi Payout).
    #[payable]
    pub fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        balance: U128,
        max_len_payout: Option<u32>,
    ) -> Payout {
        assert_one_yocto();

        let payout = self.royalty_payout(balance);

        // NEP-199: pemanggil menyatakan plafon jumlah penerima yang disanggupinya.
        if let Some(limit) = max_len_payout {
            if payout.len() > limit as usize {
                env::panic_str(err::CHAIN_REVERT);
            }
        }

        // Approval id kontrak ini `u32` (derive NEP-178). Id di luar rentang itu tidak mungkin
        // valid, jadi diperlakukan sebagai "bukan approval" → jalur owner, yang tetap gagal
        // bila pemanggil bukan pemilik (fail-closed).
        let authorization = approval_id
            .and_then(|id| u32::try_from(id).ok())
            .map(nep171::Nep171TransferAuthorization::ApprovalId)
            .unwrap_or(nep171::Nep171TransferAuthorization::Owner);

        let transfer = Nep171Transfer::new(
            token_id,
            env::predecessor_account_id(),
            receiver_id,
            authorization,
        );
        let transfer = match memo {
            Some(memo) => transfer.memo(memo),
            None => transfer,
        };

        <Self as Nep171Controller>::external_transfer(self, &transfer)
            .unwrap_or_else(|e| env::panic_str(&format!("{}: {e}", err::CHAIN_REVERT)));

        payout
    }

    /// Replace-all konfigurasi fase (owner-only). Deposit terlampir dikreditkan ke
    /// saldo storage NEP-145 creator, lalu pertumbuhan state fase ditagihkan dari sana.
    #[payable]
    pub fn set_phases(&mut self, phases: Vec<PhaseConfig>) {
        self.assert_owner();

        let count =
            u32::try_from(phases.len()).unwrap_or_else(|_| env::panic_str(err::CHAIN_REVERT));
        if count == 0 || count > MAX_PHASES {
            env::panic_str(err::CHAIN_REVERT);
        }

        for phase in &phases {
            if phase.ends_at <= phase.starts_at {
                env::panic_str(err::CHAIN_REVERT);
            }
            if phase.price_yocto.0 == 0 {
                env::panic_str(err::INVALID_PRICE);
            }
            // 0 = ambigu (selalu habis / tanpa batas) → ditolak saat konfigurasi.
            if phase.allocation == 0 || phase.max_per_wallet == 0 {
                env::panic_str(err::CHAIN_REVERT);
            }
        }

        // Fase berurutan & tidak overlap (INV-029). Window [starts_at, ends_at).
        for pair in phases.windows(2) {
            if pair[0].ends_at > pair[1].starts_at {
                env::panic_str(err::CONFLICT_PHASE_OVERLAP);
            }
        }

        let creator = env::predecessor_account_id();
        self.credit_attached_storage(&creator);

        let storage_usage_start = env::storage_usage();
        self.phases.clear();
        for phase in phases {
            self.phases.push(&Phase {
                name: phase.name,
                price_yocto: phase.price_yocto,
                allocation: phase.allocation,
                alloc_left: phase.allocation,
                max_per_wallet: phase.max_per_wallet,
                allowlist_required: phase.allowlist_required,
                starts_at: phase.starts_at,
                ends_at: phase.ends_at,
            });
        }
        self.charge_storage(&creator, storage_usage_start);
    }

    /// Tambah baris allowlist satu fase (owner-only, idempoten per akun).
    #[payable]
    pub fn allowlist_add(&mut self, phase_index: u16, entries: Vec<AllowlistEntry>) {
        self.assert_owner();

        let phase = self
            .phases
            .get(u64::from(phase_index))
            .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));

        let count =
            u32::try_from(entries.len()).unwrap_or_else(|_| env::panic_str(err::CHAIN_REVERT));
        if count > MAX_ALLOWLIST_BATCH {
            env::panic_str(err::CHAIN_REVERT);
        }

        let creator = env::predecessor_account_id();
        self.credit_attached_storage(&creator);

        let storage_usage_start = env::storage_usage();
        for entry in entries {
            if let Some(max_mints) = entry.max_mints {
                // `max_mints ≤ max_per_wallet` fase tsb (AC-COLL-4)
                if max_mints == 0 || max_mints > phase.max_per_wallet {
                    env::panic_str(err::CHAIN_REVERT);
                }
            }

            let key = (phase_index, entry.account_id);
            // Idempoten: akun terdaftar = no-op kecuali `max_mints` baru diberikan.
            let value = match (self.allowlist.get(&key), entry.max_mints) {
                (Some(existing), None) => existing,
                (_, provided) => provided,
            };
            self.allowlist.insert(&key, &value);
        }
        self.charge_storage(&creator, storage_usage_start);
    }

    /// Fase, harga, sisa alokasi, dan status tiap fase (baca lokal, tanpa cron).
    pub fn get_launchpad(&self) -> LaunchpadStatus {
        let now = env::block_timestamp();
        let mut phases = Vec::new();
        let mut active_phase_index = None;

        for index in 0..self.phases.len() {
            let phase = self
                .phases
                .get(index)
                .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
            let status = if now < phase.starts_at {
                PhaseState::Scheduled
            } else if now < phase.ends_at {
                PhaseState::Active
            } else {
                PhaseState::Ended
            };
            let index = u16::try_from(index).unwrap_or_else(|_| env::panic_str(err::CHAIN_REVERT));

            if status == PhaseState::Active && active_phase_index.is_none() {
                active_phase_index = Some(index);
            }

            phases.push(PhaseStatus {
                index,
                name: phase.name,
                price_yocto: phase.price_yocto,
                allocation: phase.allocation,
                alloc_left: phase.alloc_left,
                max_per_wallet: phase.max_per_wallet,
                allowlist_required: phase.allowlist_required,
                starts_at: phase.starts_at,
                ends_at: phase.ends_at,
                status,
            });
        }

        LaunchpadStatus {
            phases,
            active_phase_index,
        }
    }

    /// Keanggotaan allowlist satu fase — verifikasi publik on-chain (ADR-008).
    pub fn allowlist_contains(&self, phase_index: u16, account_id: AccountId) -> bool {
        self.allowlist.contains_key(&(phase_index, account_id))
    }

    /// Penerima + rate royalti (dipakai market untuk pre-validasi payout).
    pub fn royalty_config(&self) -> RoyaltyConfig {
        RoyaltyConfig {
            receiver: self.creator_id.clone(),
            bps: self.royalty_bps,
        }
    }

    /// Payout royalti satu token pada `balance` (harga wajar token) — floor ke yocto.
    /// Diturunkan dari konfigurasi **level kontrak**, bukan per-token
    /// (docs/contracts/nft-collection.md §4). INV-027: rate ≤10% sudah dijaga saat init.
    fn royalty_payout(&self, balance: U128) -> Payout {
        let amount = royalty_amount(balance.0, self.royalty_bps);

        // `amount == 0` (basis di bawah granularitas rate) tetap dikembalikan apa adanya:
        // market yang memutuskan penerima dust ini masuk merge atau tidak (INV-003).
        Payout::from([(self.creator_id.clone(), U128(amount))])
    }

    /// Referensi market yang ditampilkan FE/indexer (tidak dipakai otorisasi).
    pub fn market_id(&self) -> Option<AccountId> {
        self.market_id.clone()
    }

    /// Resolusi fase aktif: `Some(i)` memaksa fase i (tetap wajib aktif),
    /// `None` mencari satu-satunya fase aktif (INV-029).
    fn resolve_active_phase(&self, requested: Option<u16>) -> u16 {
        let now = env::block_timestamp();

        if let Some(index) = requested {
            let phase = self
                .phases
                .get(u64::from(index))
                .unwrap_or_else(|| env::panic_str(err::LAUNCHPAD_PHASE_INACTIVE));
            if !is_active(&phase, now) {
                env::panic_str(err::LAUNCHPAD_PHASE_INACTIVE);
            }
            return index;
        }

        let mut found = None;
        for index in 0..self.phases.len() {
            let phase = self
                .phases
                .get(index)
                .unwrap_or_else(|| env::panic_str(err::CHAIN_REVERT));
            if is_active(&phase, now) {
                if found.is_some() {
                    // INV-029: maksimum satu fase aktif pada satu waktu.
                    env::panic_str(err::CONFLICT_PHASE_OVERLAP);
                }
                found = Some(
                    u16::try_from(index).unwrap_or_else(|_| env::panic_str(err::CHAIN_REVERT)),
                );
            }
        }

        found.unwrap_or_else(|| env::panic_str(err::LAUNCHPAD_PHASE_INACTIVE))
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

    /// Tagih pertumbuhan storage ke pemanggil (INV-019/INV-020).
    fn charge_storage(&mut self, account_id: &AccountId, storage_usage_start: u64) {
        Nep145Controller::storage_accounting(self, account_id, storage_usage_start)
            .unwrap_or_else(|e| env::panic_str(&format!("{}: {e}", err::CHAIN_REVERT)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, Gas, NearToken};
    use near_sdk_contract_tools::owner::OwnerExternal;

    const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
    const PRICE: u128 = ONE_NEAR;
    /// Awal window fase (nanodetik) — jauh di depan supaya stabil.
    const T0: u64 = 1_800_000_000_000_000_000;
    const DAY_NS: u64 = 86_400_000_000_000;

    fn owner() -> AccountId {
        "owner.testnet".parse().unwrap()
    }

    fn creator() -> AccountId {
        "creator.testnet".parse().unwrap()
    }

    fn alice() -> AccountId {
        "alice.testnet".parse().unwrap()
    }

    fn bob() -> AccountId {
        "bob.testnet".parse().unwrap()
    }

    fn market() -> AccountId {
        "market.testnet".parse().unwrap()
    }

    fn ctx(predecessor: &AccountId, deposit: u128, now: u64) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .predecessor_account_id(predecessor.clone())
            .attached_deposit(NearToken::from_yoctonear(deposit))
            .block_timestamp(now);
        builder
    }

    fn run(predecessor: &AccountId, deposit: u128, now: u64) {
        testing_env!(ctx(predecessor, deposit, now).build());
    }

    fn new_contract() -> NftCollection {
        new_contract_with_royalty(500)
    }

    fn new_contract_with_royalty(royalty_bps: u16) -> NftCollection {
        run(&owner(), 0, T0);
        NftCollection::new(
            owner(),
            creator(),
            royalty_bps,
            "Genesis".to_string(),
            "GNS".to_string(),
            Some("https://ipfs.io/ipfs/bafy-collection".to_string()),
            None,
        )
    }

    fn phase(
        starts_at: u64,
        ends_at: u64,
        price: u128,
        allocation: u32,
        max_per_wallet: u32,
    ) -> PhaseConfig {
        PhaseConfig {
            name: "Public".to_string(),
            price_yocto: U128(price),
            allocation,
            max_per_wallet,
            allowlist_required: false,
            starts_at,
            ends_at,
        }
    }

    fn deposit_storage(contract: &mut NftCollection, account: &AccountId) {
        run(account, ONE_NEAR, T0);
        contract.storage_deposit(None, None);
    }

    /// Koleksi siap-mint: satu fase publik terbuka di `T0`, owner & minter sudah ber-storage.
    fn contract_with_public_phase(allocation: u32, max_per_wallet: u32) -> NftCollection {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());
        run(&owner(), 0, T0);
        contract.set_phases(vec![phase(
            T0,
            T0 + DAY_NS,
            PRICE,
            allocation,
            max_per_wallet,
        )]);
        contract
    }

    /// Token `0` milik alice, sudah di-approve ke market (situasi listing 2-tx).
    fn contract_with_approved_token() -> NftCollection {
        let mut contract = contract_with_public_phase(10, 2);
        deposit_storage(&mut contract, &alice());
        deposit_storage(&mut contract, &bob());

        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);

        run(&alice(), 1, T0);
        let _ = contract.nft_approve("0".to_string(), market(), None);
        contract
    }

    /// `approval_id` yang dipegang market untuk token `0`.
    fn market_approval_id(contract: &NftCollection) -> u64 {
        u64::from(
            contract
                .get_approval_id_for(&"0".to_string(), &market())
                .expect("market harus punya approval"),
        )
    }

    // --- init (SEC-CONTRACT-002, AC "Init menolak royalty_bps di luar 1..=1000") ---

    #[test]
    fn test_new_sets_owner_metadata_and_royalty() {
        let contract = new_contract();

        assert_eq!(contract.own_get_owner(), Some(owner()));

        let metadata = contract.nft_metadata();
        assert_eq!(metadata.name, "Genesis");
        assert_eq!(metadata.symbol, "GNS");
        assert_eq!(
            metadata.base_uri.as_deref(),
            Some("https://ipfs.io/ipfs/bafy-collection")
        );

        let royalty = contract.royalty_config();
        assert_eq!(royalty.receiver, creator());
        assert_eq!(royalty.bps, 500);
        assert_eq!(contract.market_id(), None);
    }

    #[test]
    #[should_panic(expected = "INVALID_ROYALTY")]
    fn test_init_rejects_royalty_below_min() {
        run(&owner(), 0, T0);
        NftCollection::new(
            owner(),
            creator(),
            0,
            "G".to_string(),
            "G".to_string(),
            None,
            None,
        );
    }

    #[test]
    #[should_panic(expected = "INVALID_ROYALTY")]
    fn test_init_rejects_royalty_above_cap() {
        run(&owner(), 0, T0);
        NftCollection::new(
            owner(),
            creator(),
            MAX_ROYALTY_BPS + 1,
            "G".to_string(),
            "G".to_string(),
            None,
            None,
        );
    }

    // --- mint + transfer + events (TC-001) ---

    #[test]
    fn test_mint_transfer_and_events() {
        let mut contract = contract_with_public_phase(10, 2);
        deposit_storage(&mut contract, &alice());
        deposit_storage(&mut contract, &bob());

        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);

        let token = contract.nft_token("0".to_string()).expect("token minted");
        assert_eq!(token.owner_id, alice());
        assert_eq!(contract.nft_total_supply(), 1.into());

        let logs = get_logs();
        assert!(
            logs.iter().any(|log| log.contains(r#""standard":"nep171""#)
                && log.contains(r#""event":"nft_mint""#)),
            "event nft_mint NEP-171 wajib ter-emit: {logs:?}"
        );
        assert!(
            logs.iter()
                .any(|log| log.contains(r#""standard":"x-nearsea-market""#)
                    && log.contains(r#""event":"launchpad_mint""#)
                    && log.contains(r#""phase_index":0"#)
                    && log.contains(r#""token_ids":["0"]"#)),
            "event launchpad_mint wajib ter-emit: {logs:?}"
        );

        run(&alice(), 1, T0);
        contract.nft_transfer(bob(), "0".to_string(), None, None);

        assert_eq!(
            contract.nft_token("0".to_string()).expect("token").owner_id,
            bob()
        );
        assert!(
            get_logs()
                .iter()
                .any(|log| log.contains(r#""standard":"nep171""#)
                    && log.contains(r#""event":"nft_transfer""#)),
            "event nft_transfer NEP-171 wajib ter-emit"
        );
    }

    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_transfer_requires_one_yocto() {
        let mut contract = contract_with_public_phase(10, 2);
        deposit_storage(&mut contract, &alice());
        deposit_storage(&mut contract, &bob());

        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);

        run(&alice(), 0, T0);
        contract.nft_transfer(bob(), "0".to_string(), None, None);
    }

    #[test]
    fn test_mint_quantity_mints_sequential_ids() {
        let mut contract = contract_with_public_phase(10, 3);
        deposit_storage(&mut contract, &alice());

        run(&alice(), PRICE * 3, T0);
        contract.nft_mint(Some(0), 3);

        assert_eq!(contract.nft_total_supply(), 3.into());
        assert_eq!(contract.nft_supply_for_owner(alice()), 3.into());
        for token_id in ["0", "1", "2"] {
            assert_eq!(
                contract
                    .nft_token(token_id.to_string())
                    .expect("token")
                    .owner_id,
                alice()
            );
        }
    }

    // --- INV-017/018/019/021/029 ---

    #[test]
    #[should_panic(expected = "LAUNCHPAD_PHASE_INACTIVE")]
    fn test_mint_without_phases_is_impossible() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &alice());

        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);
    }

    #[test]
    #[should_panic(expected = "LAUNCHPAD_PRICE_MISMATCH")]
    fn test_mint_requires_exact_deposit() {
        let mut contract = contract_with_public_phase(10, 2);
        deposit_storage(&mut contract, &alice());

        run(&alice(), PRICE - 1, T0);
        contract.nft_mint(None, 1);
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_mint_without_storage_deposit_fails() {
        let contract = contract_with_public_phase(10, 2);
        let mut contract = contract;

        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);
    }

    #[test]
    #[should_panic(expected = "LAUNCHPAD_ALLOCATION_EXHAUSTED")]
    fn test_mint_stops_at_allocation() {
        let mut contract = contract_with_public_phase(2, 5);
        deposit_storage(&mut contract, &alice());

        run(&alice(), PRICE * 2, T0);
        contract.nft_mint(None, 2);

        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);
    }

    #[test]
    #[should_panic(expected = "LAUNCHPAD_MAX_PER_WALLET")]
    fn test_mint_respects_max_per_wallet() {
        let mut contract = contract_with_public_phase(10, 1);
        deposit_storage(&mut contract, &alice());

        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);

        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_mint_rejects_quantity_above_static_limit() {
        let mut contract = contract_with_public_phase(100, 100);
        deposit_storage(&mut contract, &alice());

        run(&alice(), PRICE * u128::from(MAX_MINT_PER_CALL + 1), T0);
        contract.nft_mint(None, MAX_MINT_PER_CALL + 1);
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_mint_rejects_zero_quantity() {
        let mut contract = contract_with_public_phase(10, 2);
        deposit_storage(&mut contract, &alice());

        run(&alice(), 0, T0);
        contract.nft_mint(None, 0);
    }

    #[test]
    #[should_panic(expected = "LAUNCHPAD_PHASE_INACTIVE")]
    fn test_mint_rejected_before_window_opens() {
        let mut contract = contract_with_public_phase(10, 2);
        deposit_storage(&mut contract, &alice());

        run(&alice(), PRICE, T0 - 1);
        contract.nft_mint(None, 1);
    }

    #[test]
    #[should_panic(expected = "LAUNCHPAD_PHASE_INACTIVE")]
    fn test_mint_rejected_at_window_end_exclusive() {
        let mut contract = contract_with_public_phase(10, 2);
        deposit_storage(&mut contract, &alice());

        // `ends_at` eksklusif — tepat di batas sudah tidak aktif.
        run(&alice(), PRICE, T0 + DAY_NS);
        contract.nft_mint(None, 1);
    }

    #[test]
    #[should_panic(expected = "LAUNCHPAD_PHASE_INACTIVE")]
    fn test_mint_rejected_for_out_of_range_phase_index() {
        let mut contract = contract_with_public_phase(10, 2);
        deposit_storage(&mut contract, &alice());

        run(&alice(), PRICE, T0);
        contract.nft_mint(Some(7), 1);
    }

    // --- royalti NEP-199 (INV-003/027, TC-003) ---

    #[test]
    fn test_transfer_payout_moves_token_and_returns_creator_royalty() {
        let mut contract = contract_with_approved_token();
        let approval_id = market_approval_id(&contract);

        run(&market(), 1, T0);
        let payout = contract.nft_transfer_payout(
            bob(),
            "0".to_string(),
            Some(approval_id),
            None,
            U128(PRICE),
            Some(10),
        );

        assert_eq!(
            contract.nft_token("0".to_string()).expect("token").owner_id,
            bob(),
            "NEP-199 memindahkan token dalam panggilan yang sama"
        );
        assert_eq!(payout.len(), 1, "koleksi selalu membayar satu penerima");
        assert_eq!(payout[&creator()], U128(PRICE * 500 / 10_000));
        assert!(
            payout[&creator()].0 * 10 <= PRICE,
            "royalti per token ≤10% harga (INV-027)"
        );
    }

    #[test]
    fn test_transfer_payout_never_exceeds_ten_percent() {
        for bps in [MIN_ROYALTY_BPS, 250, 500, MAX_ROYALTY_BPS] {
            for balance in [1u128, 19, 20, 999, PRICE, PRICE * 7] {
                let amount = royalty_amount(balance, bps);

                assert_eq!(
                    amount,
                    balance * u128::from(bps) / 10_000,
                    "floor eksak (bps={bps}, balance={balance})"
                );
                assert!(
                    amount * 10 <= balance,
                    "bps={bps} balance={balance} → amount={amount} melebihi 10%"
                );
            }
        }
    }

    #[test]
    fn test_transfer_payout_floors_to_zero_on_dust_basis() {
        // 500 bps → granularitas 20 yocto; di bawah itu royalti dust = 0.
        assert_eq!(royalty_amount(19, 500), 0);
        assert_eq!(royalty_amount(20, 500), 1);
    }

    #[test]
    fn test_transfer_payout_at_cap_is_exactly_ten_percent() {
        assert_eq!(royalty_amount(PRICE, MAX_ROYALTY_BPS), PRICE / 10);
    }

    #[test]
    fn test_payout_uses_contract_level_royalty_config() {
        let contract = new_contract_with_royalty(750);
        let config = contract.royalty_config();

        assert_eq!(config.receiver, creator());
        assert_eq!(config.bps, 750);
        assert_eq!(
            contract.royalty_payout(U128(PRICE)),
            Payout::from([(config.receiver, U128(PRICE * 750 / 10_000))]),
            "payout = konfigurasi royalti level kontrak, bukan metadata per-token"
        );
    }

    #[test]
    #[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
    fn test_transfer_payout_requires_one_yocto() {
        let mut contract = contract_with_approved_token();
        let approval_id = market_approval_id(&contract);

        run(&market(), 0, T0);
        contract.nft_transfer_payout(
            bob(),
            "0".to_string(),
            Some(approval_id),
            None,
            U128(PRICE),
            Some(10),
        );
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_transfer_payout_rejected_for_unapproved_sender() {
        let mut contract = contract_with_approved_token();

        run(&bob(), 1, T0);
        contract.nft_transfer_payout(
            alice(),
            "0".to_string(),
            Some(0),
            None,
            U128(PRICE),
            Some(10),
        );
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_transfer_payout_rejected_for_unknown_token() {
        let mut contract = contract_with_approved_token();

        run(&market(), 1, T0);
        contract.nft_transfer_payout(
            bob(),
            "99".to_string(),
            Some(0),
            None,
            U128(PRICE),
            Some(10),
        );
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_transfer_payout_rejects_max_len_payout_below_payout_size() {
        let mut contract = contract_with_approved_token();
        let approval_id = market_approval_id(&contract);

        // Payout = 1 penerima; pemanggil yang hanya menyanggupi 0 penerima ditolak (NEP-199).
        run(&market(), 1, T0);
        contract.nft_transfer_payout(
            bob(),
            "0".to_string(),
            Some(approval_id),
            None,
            U128(PRICE),
            Some(0),
        );
    }

    #[test]
    fn test_transfer_payout_fits_market_gas_budget() {
        let mut contract = contract_with_approved_token();
        let approval_id = market_approval_id(&contract);

        // Mock near-sdk memakai VMLogic asli, jadi prepaid gas benar-benar ditegakkan.
        // Angka terukur ~1,70 Tgas adalah gas host function saja (tanpa gas CPU wasm), jadi
        // ia batas bawah; ambang 5 Tgas di bawah memberi ruang bagi variasi CPU.
        let mut builder = ctx(&market(), 1, T0);
        builder.prepaid_gas(Gas::from_tgas(15));
        testing_env!(builder.build());

        let payout = contract.nft_transfer_payout(
            bob(),
            "0".to_string(),
            Some(approval_id),
            None,
            U128(PRICE),
            Some(10),
        );

        assert_eq!(payout.len(), 1);
        assert_eq!(
            contract.nft_token("0".to_string()).expect("token").owner_id,
            bob()
        );
        assert!(
            env::used_gas() < Gas::from_tgas(5),
            "jalur payout harus jauh di bawah anggaran 15 Tgas market, terpakai {:?}",
            env::used_gas()
        );
    }

    #[test]
    fn test_transfer_payout_invalidates_approval() {
        let mut contract = contract_with_approved_token();
        let approval_id = market_approval_id(&contract);
        assert!(contract.nft_is_approved("0".to_string(), market(), Some(approval_id as u32)));

        run(&market(), 1, T0);
        contract.nft_transfer_payout(
            bob(),
            "0".to_string(),
            Some(approval_id),
            None,
            U128(PRICE),
            Some(10),
        );

        assert!(
            !contract.nft_is_approved("0".to_string(), market(), None),
            "approval lama invalid setelah transfer (INV-011)"
        );
    }

    // --- set_phases (INV-029) ---

    #[test]
    #[should_panic(expected = "CONFLICT_PHASE_OVERLAP")]
    fn test_set_phases_rejects_overlap() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        run(&owner(), 0, T0);
        contract.set_phases(vec![
            phase(T0, T0 + DAY_NS, PRICE, 10, 2),
            phase(T0 + DAY_NS / 2, T0 + 2 * DAY_NS, PRICE, 10, 2),
        ]);
    }

    #[test]
    fn test_set_phases_allows_adjacent_windows() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        run(&owner(), 0, T0);
        contract.set_phases(vec![
            phase(T0, T0 + DAY_NS, PRICE, 10, 2),
            phase(T0 + DAY_NS, T0 + 2 * DAY_NS, PRICE * 2, 20, 1),
        ]);

        let launchpad = contract.get_launchpad();
        assert_eq!(launchpad.phases.len(), 2);
        assert_eq!(launchpad.active_phase_index, Some(0));
        assert_eq!(launchpad.phases[0].status, PhaseState::Active);
        assert_eq!(launchpad.phases[1].status, PhaseState::Scheduled);
        assert_eq!(launchpad.phases[0].alloc_left, 10);
    }

    #[test]
    #[should_panic(expected = "INVALID_PRICE")]
    fn test_set_phases_rejects_zero_price() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        run(&owner(), 0, T0);
        contract.set_phases(vec![phase(T0, T0 + DAY_NS, 0, 10, 2)]);
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_set_phases_rejects_empty_allocation() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        run(&owner(), 0, T0);
        contract.set_phases(vec![phase(T0, T0 + DAY_NS, PRICE, 0, 2)]);
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_set_phases_rejects_inverted_window() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        run(&owner(), 0, T0);
        contract.set_phases(vec![phase(T0 + DAY_NS, T0, PRICE, 10, 2)]);
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_set_phases_rejects_too_many_phases() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        let phases = (0..=MAX_PHASES)
            .map(|i| {
                let start = T0 + u64::from(i) * DAY_NS;
                phase(start, start + DAY_NS, PRICE, 1, 1)
            })
            .collect();

        run(&owner(), 0, T0);
        contract.set_phases(phases);
    }

    #[test]
    #[should_panic]
    fn test_set_phases_is_owner_only() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &alice());

        run(&alice(), 0, T0);
        contract.set_phases(vec![phase(T0, T0 + DAY_NS, PRICE, 10, 2)]);
    }

    // --- allowlist (INV-017, AC-COLL-4) ---

    #[test]
    fn test_allowlist_gates_mint() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        let mut gated = phase(T0, T0 + DAY_NS, PRICE, 10, 2);
        gated.allowlist_required = true;
        run(&owner(), 0, T0);
        contract.set_phases(vec![gated]);

        run(&owner(), 0, T0);
        contract.allowlist_add(
            0,
            vec![AllowlistEntry {
                account_id: alice(),
                max_mints: None,
            }],
        );

        assert!(contract.allowlist_contains(0, alice()));
        assert!(!contract.allowlist_contains(0, bob()));

        deposit_storage(&mut contract, &alice());
        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);
        assert_eq!(contract.nft_supply_for_owner(alice()), 1.into());
    }

    #[test]
    #[should_panic(expected = "LAUNCHPAD_NOT_ALLOWED")]
    fn test_mint_rejected_when_not_allowlisted() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        let mut gated = phase(T0, T0 + DAY_NS, PRICE, 10, 2);
        gated.allowlist_required = true;
        run(&owner(), 0, T0);
        contract.set_phases(vec![gated]);

        deposit_storage(&mut contract, &bob());
        run(&bob(), PRICE, T0);
        contract.nft_mint(None, 1);
    }

    #[test]
    fn test_allowlist_entry_caps_per_wallet_below_phase_limit() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        let mut gated = phase(T0, T0 + DAY_NS, PRICE, 10, 5);
        gated.allowlist_required = true;
        run(&owner(), 0, T0);
        contract.set_phases(vec![gated]);

        run(&owner(), 0, T0);
        contract.allowlist_add(
            0,
            vec![AllowlistEntry {
                account_id: alice(),
                max_mints: Some(1),
            }],
        );

        deposit_storage(&mut contract, &alice());
        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);

        run(&alice(), PRICE, T0);
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            contract.nft_mint(None, 1);
        }));
        assert!(
            outcome.is_err(),
            "batas baris allowlist (1) harus menang atas max_per_wallet (5)"
        );
    }

    #[test]
    #[should_panic(expected = "CHAIN_REVERT")]
    fn test_allowlist_add_rejects_entry_above_phase_limit() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        let mut gated = phase(T0, T0 + DAY_NS, PRICE, 10, 2);
        gated.allowlist_required = true;
        run(&owner(), 0, T0);
        contract.set_phases(vec![gated]);

        run(&owner(), 0, T0);
        contract.allowlist_add(
            0,
            vec![AllowlistEntry {
                account_id: alice(),
                max_mints: Some(3),
            }],
        );
    }

    #[test]
    fn test_allowlist_add_is_idempotent() {
        let mut contract = new_contract();
        deposit_storage(&mut contract, &owner());

        run(&owner(), 0, T0);
        contract.set_phases(vec![phase(T0, T0 + DAY_NS, PRICE, 10, 2)]);

        run(&owner(), 0, T0);
        contract.allowlist_add(
            0,
            vec![AllowlistEntry {
                account_id: alice(),
                max_mints: Some(2),
            }],
        );
        contract.allowlist_add(
            0,
            vec![AllowlistEntry {
                account_id: alice(),
                max_mints: None,
            }],
        );

        // Baris kedua tidak menurunkan `max_mints` yang sudah ada (no-op).
        assert!(contract.allowlist_contains(0, alice()));
    }

    // --- NEP-178 approval ---

    #[test]
    fn test_approvals_lifecycle() {
        let mut contract = contract_with_public_phase(10, 2);
        deposit_storage(&mut contract, &alice());

        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);

        run(&alice(), 1, T0);
        let _ = contract.nft_approve("0".to_string(), bob(), None);

        assert!(contract.nft_is_approved("0".to_string(), bob(), None));
        assert_eq!(
            contract.get_approval_id_for(&"0".to_string(), &bob()),
            Some(0)
        );

        run(&alice(), 1, T0);
        contract.nft_revoke("0".to_string(), bob());
        assert!(!contract.nft_is_approved("0".to_string(), bob(), None));

        run(&alice(), 1, T0);
        let _ = contract.nft_approve("0".to_string(), bob(), None);
        run(&alice(), 1, T0);
        contract.nft_revoke_all("0".to_string());
        assert!(!contract.nft_is_approved("0".to_string(), bob(), None));
    }

    #[test]
    fn test_enumeration_for_owner() {
        let mut contract = contract_with_public_phase(10, 2);
        deposit_storage(&mut contract, &alice());

        run(&alice(), PRICE, T0);
        contract.nft_mint(None, 1);

        let tokens = contract.nft_tokens_for_owner(alice(), None, None);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_id, "0");
        assert_eq!(contract.nft_supply_for_owner(bob()), 0.into());
    }

    // --- storage NEP-145 (INV-020) ---

    #[test]
    fn test_storage_bounds_are_configured() {
        let contract = new_contract();
        let bounds = contract.storage_balance_bounds();

        assert_eq!(
            bounds.min,
            env::storage_byte_cost().saturating_mul(u128::from(STORAGE_MIN_BYTES))
        );
        assert_eq!(bounds.max, None);
    }
}
