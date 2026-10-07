//! NearSea NFT Collection — NEP-171/177/178/181/199 + state launchpad.
//!
//! Scaffold: hanya init + owner agar workspace ter-compile dan teruji. Surface lengkap
//! (`nft_mint`, approval, royalti, `set_phases`) diimplementasikan di TASK-002/003 —
//! signature kanonik: docs/contracts/nft-collection.md §1.

use near_sdk::{near, AccountId, PanicOnDefault};
use near_sdk_contract_tools::{owner::Owner, Owner as OwnerDerive};

#[near(contract_state)]
#[derive(PanicOnDefault, OwnerDerive)]
pub struct NftCollection {}

#[near]
impl NftCollection {
    /// Init sekali (PanicOnDefault → memanggil tanpa init = panic) — SEC-CONTRACT-002.
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
    use near_sdk::{testing_env, NearToken};
    use near_sdk_contract_tools::owner::OwnerExternal;

    fn owner() -> AccountId {
        "owner.testnet".parse().unwrap()
    }

    // SEC-CONTRACT-002 · scaffold (TC-001 penuh menyusul di TASK-002).
    #[test]
    fn test_new_sets_owner() {
        testing_env!(VMContextBuilder::new()
            .predecessor_account_id(owner())
            .attached_deposit(NearToken::from_yoctonear(0))
            .build());

        let contract = NftCollection::new(owner());

        assert_eq!(contract.own_get_owner(), Some(owner()));
    }
}
