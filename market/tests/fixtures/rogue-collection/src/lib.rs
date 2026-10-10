//! Fixture **TEST SAJA** — koleksi pihak ketiga yang "nakal" (TASK-006 / TC-003).
//!
//! Bukan kontrak produksi: ia hanya mengimplementasikan tiga method yang dipanggil market
//! (`nft_token`, `nft_is_approved`, `nft_transfer_payout`) dan **selalu mengembalikan payout
//! yang bisa dikonfigurasi menjadi tidak valid**. Tujuannya membuktikan market tidak
//! mempercayai payout dari koleksi (UNTRUSTED, docs/features/payments.md §Algoritma Distribusi
//! Payout): payout invalid → refund penuh buyer + listing dipulihkan (SEC-ORDER-001, INV-002/003).
//!
//! Payout yang bisa disetel: `empty` (kosong), `zero` (amount 0), `over_cap` (> 10 penerima),
//! `over_price` (Σ melebihi harga − fee), atau `valid` (kontrol pembanding).

use near_sdk::json_types::U128;
use near_sdk::serde_json::{json, Value};
use near_sdk::{env, near, AccountId, NearToken, PanicOnDefault};
use std::collections::HashMap;

/// Payout NEP-199 — map penerima → jumlah yoctoNEAR.
pub type Payout = HashMap<AccountId, U128>;

/// Bentuk payout yang dikembalikan fixture.
#[near(serializers = [json, borsh])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PayoutMode {
    Valid,
    Empty,
    ZeroAmount,
    OverReceiverCap,
    OverPrice,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct RogueCollection {
    owner_id: AccountId,
    /// Pemilik token yang dilaporkan ke market (dipakai verifikasi kepemilikan).
    token_owner: AccountId,
    /// Penerima royalti yang dilaporkan.
    royalty_receiver: AccountId,
    mode: PayoutMode,
    /// Berapa kali `nft_transfer_payout` dipanggil (bukti market benar-benar sampai ke sini).
    payout_calls: u32,
}

#[near]
impl RogueCollection {
    #[init]
    pub fn new(owner_id: AccountId, token_owner: AccountId, royalty_receiver: AccountId) -> Self {
        Self {
            owner_id,
            token_owner,
            royalty_receiver,
            mode: PayoutMode::OverPrice,
            payout_calls: 0,
        }
    }

    /// Setel bentuk payout yang dikembalikan (owner-only).
    pub fn set_mode(&mut self, mode: PayoutMode) {
        self.assert_owner();
        self.mode = mode;
    }

    /// Ganti pemilik yang dilaporkan — dipakai menyiapkan kondisi stale bila perlu.
    pub fn set_token_owner(&mut self, token_owner: AccountId) {
        self.assert_owner();
        self.token_owner = token_owner;
    }

    /// Berapa kali `nft_transfer_payout` dipanggil.
    pub fn payout_calls(&self) -> u32 {
        self.payout_calls
    }

    // --- Permukaan yang dipanggil market ---

    /// NEP-171: bentuk `Token` yang dibaca market (near-sdk-contract-tools).
    pub fn nft_token(&self, token_id: String) -> Option<Value> {
        Some(json!({
            "token_id": token_id,
            "owner_id": self.token_owner,
        }))
    }

    /// NEP-178: fixture selalu melaporkan "approved" — market tetap wajib memvalidasi payout
    /// sendiri, jadi approval palsu tidak boleh cukup untuk menyelesaikan penjualan yang sah.
    ///
    /// Nama parameter mengikuti ABI NEP-178 apa adanya (`token_id`, `approved_account_id`,
    /// `approval_id`): derive `#[near]` memakai nama parameter sebagai kunci JSON argumen.
    pub fn nft_is_approved(
        &self,
        token_id: String,
        approved_account_id: AccountId,
        approval_id: Option<u32>,
    ) -> bool {
        let _ = (token_id, approved_account_id, approval_id);
        true
    }

    /// NEP-199: **tidak** memindahkan token (fixture tidak punya state kepemilikan nyata) dan
    /// mengembalikan payout sesuai `mode`.
    ///
    /// `#[payable]` wajib: market memanggilnya dengan attach 1 yocto (INV-011).
    #[payable]
    pub fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
        balance: U128,
        max_len_payout: Option<u32>,
    ) -> Payout {
        let _ = (receiver_id, token_id, approval_id, memo, max_len_payout);
        assert_eq!(
            env::attached_deposit(),
            NearToken::from_yoctonear(1),
            "fixture: NEP-199 wajib 1 yocto"
        );
        self.payout_calls += 1;

        match self.mode {
            PayoutMode::Valid => self.one_receiver(balance.0 / 10),
            PayoutMode::Empty => Payout::new(),
            PayoutMode::ZeroAmount => self.one_receiver(0),
            PayoutMode::OverReceiverCap => {
                let mut payout = Payout::new();
                // 11 penerima > MAX_PAYOUT_RECEIVERS (10) — INV-003.
                for index in 0..11u32 {
                    payout.insert(
                        format!("receiver{index}.testnet")
                            .parse()
                            .expect("account id"),
                        U128(1),
                    );
                }
                payout
            }
            // Σ payout melebihi plafon `harga − fee` — INV-002.
            PayoutMode::OverPrice => self.one_receiver(balance.0.saturating_add(1)),
        }
    }

    fn one_receiver(&self, amount: u128) -> Payout {
        Payout::from([(self.royalty_receiver.clone(), U128(amount))])
    }

    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "fixture: hanya owner"
        );
    }
}
