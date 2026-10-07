//! NearSea Factory — deploy koleksi ke sub-akun + wiring.
//!
//! Scaffold: init + owner + pause agar workspace ter-compile dan teruji. `create_collection`
//! diimplementasikan di TASK-012 — signature kanonik: docs/contracts/factory.md §1–§2.

use near_sdk::{near, AccountId, PanicOnDefault};
use near_sdk_contract_tools::owner::Owner;
use near_sdk_contract_tools::{Owner as OwnerDerive, Pause as PauseDerive};

#[near(contract_state)]
#[derive(PanicOnDefault, OwnerDerive, PauseDerive)]
pub struct Factory {}

#[near]
impl Factory {
    /// Init sekali (PanicOnDefault) — SEC-CONTRACT-002.
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

    fn owner() -> AccountId {
        "owner.testnet".parse().unwrap()
    }

    // SEC-CONTRACT-002 · scaffold (TC-001 penuh menyusul di TASK-012).
    #[test]
    fn test_new_sets_owner() {
        testing_env!(VMContextBuilder::new()
            .predecessor_account_id(owner())
            .build());

        let contract = Factory::new(owner());

        assert_eq!(contract.own_get_owner(), Some(owner()));
    }
}
