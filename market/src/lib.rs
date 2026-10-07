//! NearSea Market — listing 2-tx, buy/settle, refund, fee platform.
//!
//! Scaffold: init + owner + pause + `MAX_FEE_BPS` agar workspace ter-compile dan teruji.
//! Method order (`list_nft_for_sale`, `buy`, …) diimplementasikan di TASK-004/005 —
//! signature kanonik: docs/contracts/market.md §1–§2.

use near_sdk::{near, AccountId, PanicOnDefault};
use near_sdk_contract_tools::owner::Owner;
use near_sdk_contract_tools::{Owner as OwnerDerive, Pause as PauseDerive};

/// Fee platform default 2% — nilai fee ≠ cap (INV-004, ADR-005).
pub const FEE_BPS_DEFAULT: u16 = 200;
/// Cap immutable fee — ditegakkan juga di `update_fee_bps` (SEC-CONTRACT-004).
pub const MAX_FEE_BPS: u16 = 500;

// Fee default tidak boleh melewati cap (INV-004) — ditegakkan saat kompilasi.
const _: () = assert!(FEE_BPS_DEFAULT <= MAX_FEE_BPS);

#[near(contract_state)]
#[derive(PanicOnDefault, OwnerDerive, PauseDerive)]
pub struct Market {}

#[near]
impl Market {
    /// Init sekali (PanicOnDefault) — SEC-CONTRACT-002 (TC-001).
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        let mut contract = Self {};
        Owner::init(&mut contract, &owner_id);
        contract
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;
    use near_sdk_contract_tools::owner::OwnerExternal;
    use near_sdk_contract_tools::pause::Pause;

    fn owner() -> AccountId {
        "owner.testnet".parse().unwrap()
    }

    // SEC-CONTRACT-002 · scaffold (TC-001 penuh menyusul di TASK-004).
    #[test]
    fn test_new_sets_owner() {
        testing_env!(VMContextBuilder::new()
            .predecessor_account_id(owner())
            .build());

        let contract = Market::new(owner());

        assert_eq!(contract.own_get_owner(), Some(owner()));
    }

    // SEC-CONTRACT-007 · scaffold (TC-012 penuh menyusul di TASK-004).
    #[test]
    fn test_pause_toggles_state() {
        testing_env!(VMContextBuilder::new()
            .predecessor_account_id(owner())
            .build());

        let mut contract = Market::new(owner());
        assert!(!Market::is_paused());

        contract.pause();
        assert!(Market::is_paused());

        contract.unpause();
        assert!(!Market::is_paused());
    }
}
