//! Suite sandbox dua-kontrak (slice M1) — TASK-006 / tiket 08.
//!
//! Membuktikan tesis slice di chain lokal sungguhan dengan **dua kontrak nyata** (koleksi +
//! market), bukan mock: `mint → list (2-tx, non-custodial) → orang lain beli → royalti + fee
//! terbayar → NFT tidak pernah dipegang kontrak`, plus jalur refund/race/recovery.
//!
//! Cakupan (docs/testing/test-cases.md §Cakupan slice M1):
//! TC-001, TC-002, TC-003, TC-006, TC-013, TC-016, TC-017, TC-020, TC-022, TC-044, TC-047,
//! TC-048, TC-053, TC-054 · INV-001..004, 007, 008, 011..016, 019, 020, 023, 026, 030, 031.
//!
//! **Dijalankan hanya di Linux/macOS** (binary sandbox nearcore tidak dipublikasikan untuk
//! Windows). Di Windows file ini dikompilasi menjadi nol test supaya `cargo test` lokal tetap
//! hijau; suite penuh berjalan di CI (`ubuntu-latest`) dan di WSL.
#![cfg(unix)]

mod common;

use common::*;
use near_sdk::borsh::BorshSerialize;
use near_sdk::AccountId;
use near_workspaces::types::NearToken;
use serde_json::json;
use tokio::sync::MutexGuard;

/// Serialisasi test dalam satu binary (sandbox + root account dibagi).
async fn guard() -> MutexGuard<'static, ()> {
    SERIAL.lock().await
}

/// Fixture slice siap-pakai: fase publik terbuka + akun test terdaftar storage.
async fn slice() -> Slice {
    let slice = Slice::new().await.expect("fixture slice");
    slice
        .open_public_phase(10, 5)
        .await
        .expect("fase publik terbuka");
    slice
}

// --- TC-001 · mint (INV-018/019) ---------------------------------------------

/// TC-001 · INV-019 — mint membuat token milik minter dan meng-emit event kanonik.
/// Storage mint dibayar pemicu mint (invoker), bukan kontrak.
#[tokio::test]
async fn tc_001_mint_creates_token_for_minter() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;

    let before = slice
        .collection_storage_available(slice.seller.id())
        .await?;

    let result = slice
        .seller
        .call(slice.collection.id(), "nft_mint")
        .args_json(json!({ "phase_index": null, "quantity": 1 }))
        .deposit(NearToken::from_yoctonear(PRICE))
        .max_gas()
        .transact()
        .await?;
    result.clone().into_result()?;

    assert_eq!(
        slice.token_owner("0").await?,
        Some(slice.seller.id().clone()),
        "mint menaruh token di wallet minter"
    );

    let launchpad = event_payload(&result, "launchpad_mint");
    assert_eq!(launchpad["phase_index"], json!(0));
    assert_eq!(launchpad["token_ids"], json!(["0"]));
    assert_eq!(launchpad["price_yocto"], json!(PRICE.to_string()));

    // INV-019: kontrak koleksi tidak menanggung storage — saldo NEP-145 minter yang turun.
    let after = slice
        .collection_storage_available(slice.seller.id())
        .await?;
    assert!(
        after < before,
        "storage mint ditagih ke minter (INV-019): {before} → {after}"
    );

    // INV-018: deposit non-exact ditolak.
    let mismatched = slice
        .seller
        .call(slice.collection.id(), "nft_mint")
        .args_json(json!({ "phase_index": null, "quantity": 1 }))
        .deposit(NearToken::from_yoctonear(PRICE - 1))
        .max_gas()
        .transact()
        .await?;
    assert!(
        mismatched.is_failure(),
        "deposit ≠ harga fase wajib ditolak (INV-018)"
    );
    assert!(
        failure_message(&mismatched).contains("LAUNCHPAD_PRICE_MISMATCH"),
        "kode registry INV-018: {}",
        failure_message(&mismatched)
    );
    Ok(())
}

// --- TC-002 · list + buy happy path (INV-001/002/003/011/014/015) -------------

/// TC-002 · INV-001/002/003/011/014/015 — jalur bahagia slice dengan angka **exact**.
///
/// Bukti non-custodial: selama window listing token tetap di wallet seller (diasersi on-chain).
/// Bukti settlement: `fee + royalti + proceeds == harga` persis, dan Σ keluar == Σ masuk.
#[tokio::test]
async fn tc_002_list_then_buy_settles_with_exact_split() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let buyer = slice.new_buyer().await?;

    let token_id = slice.mint_one(&slice.seller).await?;
    slice.approve_market(&slice.seller, &token_id).await?;
    let listed = slice.list(&slice.seller, &token_id, PRICE, None).await?;
    listed.clone().into_result()?;

    // Non-custodial (ADR-007): market hanya memegang approval, NFT tetap di seller.
    assert_eq!(
        slice.token_owner(&token_id).await?,
        Some(slice.seller.id().clone()),
        "NFT tetap di wallet seller selama listing (non-custodial)"
    );
    assert!(
        slice.approval_id_of(&token_id).await?.is_some(),
        "market memegang approval saat listing"
    );
    let sale = slice.sale(&token_id).await?.expect("listing ACTIVE");
    assert_eq!(sale["owner_id"], json!(slice.seller.id().as_str()));
    assert_eq!(sale["price_yocto"], json!(PRICE.to_string()));

    let before = slice.balances(&buyer).await?;
    let result = slice.buy(&buyer, &token_id, PRICE).await?;
    result.clone().into_result()?;
    let deltas = BalanceDelta::between(before, slice.balances(&buyer).await?);

    // INV-011/015: transfer hanya lewat settlement sah, approval lama invalid setelahnya.
    assert_eq!(
        slice.token_owner(&token_id).await?,
        Some(buyer.id().clone()),
        "NFT pindah ke buyer"
    );
    assert!(
        slice.approval_id_of(&token_id).await?.is_none(),
        "approval lama invalid setelah transfer (INV-011)"
    );
    assert!(
        slice.sale(&token_id).await?.is_none(),
        "listing SOLD (entry hilang)"
    );

    let sale_event = event_payload(&result, "market_sale");
    assert_eq!(sale_event["buyer"], json!(buyer.id().as_str()));
    assert_eq!(sale_event["seller"], json!(slice.seller.id().as_str()));
    assert_eq!(sale_event["price_yocto"], json!(PRICE.to_string()));

    // INV-003: 1..=10 penerima unik, tidak ada amount 0.
    let receivers = payout_receivers(&sale_event);
    assert!(
        (1..=10).contains(&receivers.len()),
        "payout 1..=10 penerima (INV-003), dapat {}",
        receivers.len()
    );
    assert!(
        receivers.contains(&slice.creator.id().to_string())
            && receivers.contains(&slice.seller.id().to_string()),
        "payout memuat kreator + seller: {receivers:?}"
    );
    let payout = sale_event["payout"].as_object().expect("payout map");
    assert!(
        payout.values().all(|amount| amount.as_str() != Some("0")),
        "tidak ada amount 0 di payout (INV-003)"
    );
    // INV-002: Σpayout (royalti + seller) == harga − fee persis.
    let payout_sum: u128 = payout
        .values()
        .map(|amount| {
            amount
                .as_str()
                .expect("string yocto")
                .parse::<u128>()
                .expect("u128")
        })
        .sum();
    assert_eq!(payout_sum, PRICE - FEE, "Σpayout == harga − fee (INV-002)");

    assert_eq!(deltas.treasury, FEE as i128, "fee 2% → treasury");
    assert_eq!(deltas.creator, ROYALTY as i128, "royalti → kreator");
    assert_eq!(
        deltas.seller, SELLER_PROCEEDS as i128,
        "proceeds seller = residual (INV-002)"
    );

    // INV-001: Σ keluar == Σ masuk (buyer kehilangan harga + gas).
    assert_eq!(
        deltas.treasury + deltas.creator + deltas.seller,
        PRICE as i128,
        "Σ keluar == Σ masuk (INV-001)"
    );
    assert!(
        deltas.buyer <= -(PRICE as i128) && deltas.buyer > -((PRICE + PRICE / 100) as i128),
        "buyer membayar harga + gas, tanpa refund: {}",
        deltas.buyer
    );
    assert_eq!(
        FEE + ROYALTY + SELLER_PROCEEDS,
        PRICE,
        "fee + royalti + proceeds == harga (aritmetika internal exact)"
    );
    Ok(())
}

/// INV-014 · SEC-ORDER-002 — transfer settlement hanya ke alamat turunan state.
///
/// Penerima yang sah = treasury (fee), kreator (royalti), seller (residual), buyer (refund).
/// Tidak ada alamat arbitrer — diperiksa dari saldo akun luar, bukan dari klaim.
#[tokio::test]
async fn inv_014_settlement_credits_only_state_derived_accounts() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let buyer = slice.new_buyer().await?;
    let outsider = slice.worker.dev_create_account().await?;

    let token_id = slice.mint_and_list(&slice.seller, PRICE).await?;

    let outsider_before = slice.balance_of(outsider.id()).await?;
    let before = slice.balances(&buyer).await?;

    let result = slice.buy(&buyer, &token_id, PRICE).await?;
    result.clone().into_result()?;

    let deltas = BalanceDelta::between(before, slice.balances(&buyer).await?);
    assert!(deltas.treasury > 0 && deltas.creator > 0 && deltas.seller > 0);
    assert_eq!(
        slice.balance_of(outsider.id()).await?,
        outsider_before,
        "akun di luar state tidak menerima apa pun (INV-014)"
    );
    Ok(())
}

// --- TC-022 · harga berubah saat signing (INV-001/030) -----------------------

/// TC-022 · INV-001/030 — deposit < harga ditolak; kelebihan deposit di-refund saat settle.
#[tokio::test]
async fn tc_022_price_change_rejects_stale_deposit_and_refunds_excess() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let buyer = slice.new_buyer().await?;

    let token_id = slice.mint_and_list(&slice.seller, PRICE).await?;

    // Seller menaikkan harga setelah buyer melihat harga lama.
    let new_price = PRICE * 3 / 2;
    let repriced = slice
        .seller
        .call(slice.market.id(), "update_price")
        .args_json(json!({
            "nft_contract_id": slice.collection.id(),
            "token_id": token_id,
            "new_price": new_price.to_string(),
        }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    repriced.clone().into_result()?;
    assert_eq!(
        slice.sale(&token_id).await?.expect("listing tetap ada")["price_yocto"],
        json!(new_price.to_string())
    );

    // (2) deposit harga lama → revert, dana tidak berpindah.
    let stale_deposit = slice.buy(&buyer, &token_id, PRICE).await?;
    assert!(stale_deposit.is_failure(), "deposit < harga wajib ditolak");
    assert!(
        failure_message(&stale_deposit).contains("CHAIN_INSUFFICIENT_DEPOSIT"),
        "kode registry: {}",
        failure_message(&stale_deposit)
    );
    assert!(
        slice.sale(&token_id).await?.is_some(),
        "listing tetap ACTIVE setelah buy gagal"
    );

    // (3) deposit lebih → sukses, kelebihan kembali ke buyer.
    let before = slice.balances(&buyer).await?;
    let result = slice.buy(&buyer, &token_id, PRICE * 2).await?;
    result.clone().into_result()?;
    let deltas = BalanceDelta::between(before, slice.balances(&buyer).await?);

    let fee = new_price * u128::from(FEE_BPS) / 10_000;
    let royalty = new_price * u128::from(ROYALTY_BPS) / 10_000;
    assert_eq!(deltas.treasury, fee as i128, "fee dihitung dari harga baru");
    assert_eq!(deltas.creator, royalty as i128);
    assert_eq!(deltas.seller, (new_price - fee - royalty) as i128);
    // Buyer membayar harga baru, kelebihan deposit kembali (selisih = gas).
    assert!(
        deltas.buyer < -(new_price as i128)
            && deltas.buyer > -(new_price as i128) - (PRICE as i128),
        "kelebihan deposit kembali ke buyer (INV-001): {}",
        deltas.buyer
    );
    Ok(())
}

// --- TC-013 · harga minimum (INV-030) ----------------------------------------

/// TC-013 · INV-030 — listing di bawah 0.01 Ⓝ ditolak; tepat di batas diterima.
#[tokio::test]
async fn tc_013_price_below_minimum_is_rejected() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;

    let token_id = slice.mint_one(&slice.seller).await?;
    slice.approve_market(&slice.seller, &token_id).await?;

    let too_cheap = slice
        .list(&slice.seller, &token_id, MIN_PRICE - 1, None)
        .await?;
    assert!(
        too_cheap.is_failure(),
        "harga < 0.01 Ⓝ wajib ditolak (INV-030)"
    );
    assert!(
        failure_message(&too_cheap).contains("INVALID_PRICE"),
        "kode registry: {}",
        failure_message(&too_cheap)
    );
    assert!(
        slice.sale(&token_id).await?.is_none(),
        "tidak ada listing tersimpan"
    );

    // Batas inklusif: tepat MIN_PRICE diterima.
    slice
        .list(&slice.seller, &token_id, MIN_PRICE, None)
        .await?
        .into_result()?;
    assert!(
        slice.sale(&token_id).await?.is_some(),
        "harga tepat minimum diterima"
    );
    Ok(())
}

// --- TC-003 · payout tidak valid dari koleksi pihak ketiga (INV-002/003) -----

/// TC-003 · INV-002/003 · SEC-ORDER-001 — payout dari koleksi diperlakukan **UNTRUSTED**.
///
/// Koleksi NearSea selalu mengembalikan payout yang sah (royalti ≤10%), jadi jalur ini diuji
/// dengan **kontrak koleksi pihak ketiga** (`market/tests/fixtures/rogue-collection`) yang
/// mengembalikan payout tidak valid: kosong, amount 0, > 10 penerima, atau Σ melebihi
/// `harga − fee`. Semuanya wajib berakhir **refund penuh buyer + listing dipulihkan** —
/// tidak ada distribusi yang salah dan tidak ada dana tertahan.
#[tokio::test]
async fn tc_003_invalid_payout_from_third_party_collection_refunds_buyer() -> anyhow::Result<()> {
    let _guard = guard().await;

    for mode in ["Empty", "ZeroAmount", "OverReceiverCap", "OverPrice"] {
        let rogue = RogueSlice::new(mode).await?;
        let buyer = rogue.slice.new_buyer().await?;
        rogue.list(PRICE).await?;
        assert!(
            rogue.sale().await?.is_some(),
            "[{mode}] listing aktif sebelum buy"
        );

        let before = rogue.balances(&buyer).await?;
        let result = rogue.buy(&buyer, PRICE).await?;
        result.clone().into_result()?;
        let after = rogue.balances(&buyer).await?;

        assert_eq!(
            rogue.payout_calls().await?,
            1,
            "[{mode}] market wajib memanggil NEP-199; validasi payout terjadi setelahnya"
        );
        assert_eq!(
            after.1, before.1,
            "[{mode}] royalti TIDAK boleh dibayar saat payout invalid (INV-002/003)"
        );
        assert_eq!(
            after.2, before.2,
            "[{mode}] fee TIDAK boleh dibayar saat payout invalid"
        );
        // Refund penuh: buyer hanya kehilangan gas.
        let spent = before.0.saturating_sub(after.0);
        assert!(
            spent < PRICE / 100,
            "[{mode}] buyer wajib refund penuh (hanya gas hangus), terpakai {spent}"
        );
        assert!(
            rogue.sale().await?.is_some(),
            "[{mode}] listing dipulihkan setelah payout invalid (SEC-ORDER-001)"
        );
        assert!(
            rogue.slice.pending_purchase("0").await?.is_none(),
            "[{mode}] tidak ada pembelian tertinggal (INV-031)"
        );
    }

    // Kontrol pembanding: payout `Valid` benar-benar mendistribusikan — membuktikan jalur di
    // atas gagal karena validasi payout, bukan karena fixture tidak pernah dipakai.
    let control = RogueSlice::new("Valid").await?;
    let buyer = control.slice.new_buyer().await?;
    control.list(PRICE).await?;
    let before = control.balances(&buyer).await?;
    control.buy(&buyer, PRICE).await?.into_result()?;
    let after = control.balances(&buyer).await?;
    assert_eq!(
        control.payout_calls().await?,
        1,
        "kontrol: NEP-199 dipanggil"
    );
    assert!(
        after.1 > before.1,
        "kontrol: payout valid wajib benar-benar membayar royalti"
    );
    assert_eq!(
        after.2 - before.2,
        FEE,
        "kontrol: fee dibayar saat payout valid"
    );
    Ok(())
}

// --- TC-020 · storage NEP-145 saat listing (INV-020) -------------------------

/// TC-020 · INV-020 — listing butuh saldo storage NEP-145; kurang = revert, bukan listing gagal senyap.
///
/// Angka konkret diambil dari view `storage_balance_bounds` (bukan hardcode):
/// `required − 1 yocto`, `0 yocto`, dan `required` persis.
#[tokio::test]
async fn tc_020_listing_requires_storage_deposit() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let required = slice.market_storage_minimum().await?;

    // Seller baru **tanpa** saldo storage di market.
    let fresh = slice.worker.dev_create_account().await?;
    slice.register_collection(&fresh).await?;
    let token_id = slice.mint_one(&fresh).await?;
    slice.approve_market(&fresh, &token_id).await?;

    let short = slice
        .list_with_deposit(&fresh, &token_id, PRICE, None, required - 1)
        .await?;
    assert!(
        short.is_failure(),
        "deposit < storage minimum wajib revert (INV-020)"
    );
    assert!(
        slice.sale(&token_id).await?.is_none(),
        "tidak ada listing tersimpan"
    );

    let zero = slice
        .list_with_deposit(&fresh, &token_id, PRICE, None, 0)
        .await?;
    assert!(zero.is_failure(), "deposit 0 wajib revert (INV-020)");
    assert!(slice.sale(&token_id).await?.is_none());

    slice
        .list_with_deposit(&fresh, &token_id, PRICE, None, required)
        .await?
        .into_result()?;
    assert!(
        slice.sale(&token_id).await?.is_some(),
        "deposit tepat `required` → listing ACTIVE"
    );
    Ok(())
}

// --- TC-044 · remove_sale (INV-012/016/020) ----------------------------------

/// TC-044 · INV-012/016/020 — `remove_sale` menghapus listing, mengembalikan storage,
/// dan **tidak** mencabut approval (NEP-178 owner-only — market.md §2a).
#[tokio::test]
async fn tc_044_remove_sale_delists_and_releases_storage() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let stranger = slice.new_buyer().await?;

    let token_id = slice.mint_and_list(&slice.seller, PRICE).await?;
    let locked = slice.market_storage_available(slice.seller.id()).await?;

    // Non-owner tidak boleh menghapus listing (INV-012).
    let by_stranger = stranger
        .call(slice.market.id(), "remove_sale")
        .args_json(json!({
            "nft_contract_id": slice.collection.id(),
            "token_id": token_id,
        }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    assert!(
        by_stranger.is_failure(),
        "remove_sale non-owner wajib revert (INV-012)"
    );

    let removed = slice
        .seller
        .call(slice.market.id(), "remove_sale")
        .args_json(json!({
            "nft_contract_id": slice.collection.id(),
            "token_id": token_id,
        }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    removed.clone().into_result()?;

    assert!(slice.sale(&token_id).await?.is_none(), "listing hilang");
    assert_eq!(
        event_payload(&removed, "market_delist")["seller"],
        json!(slice.seller.id().as_str())
    );
    // INV-020: storage entry yang dibebaskan kembali ke saldo seller.
    let released = slice.market_storage_available(slice.seller.id()).await?;
    assert!(
        released > locked,
        "storage dibebaskan ke saldo seller ({released} > {locked})"
    );
    // §2a: approval market TIDAK dicabut oleh remove_sale.
    assert!(
        slice.approval_id_of(&token_id).await?.is_some(),
        "remove_sale tidak mencabut approval (market.md §2a)"
    );
    // Listing yang sudah dihapus tidak bisa dibeli.
    let after_delist = slice.buy(&stranger, &token_id, PRICE).await?;
    assert!(
        after_delist.is_failure(),
        "buy listing yang sudah dihapus wajib revert"
    );
    assert!(
        failure_message(&after_delist).contains("CONFLICT_SOLD"),
        "kode registry: {}",
        failure_message(&after_delist)
    );
    Ok(())
}

// --- TC-047 · owner-only fee & treasury (INV-004) ----------------------------

/// TC-047 · INV-004 · SEC-CONTRACT-012 — konfigurasi fee/treasury owner-only + cap immutable.
#[tokio::test]
async fn tc_047_fee_and_treasury_are_owner_only_and_capped() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;

    // (1) non-owner `update_fee_bps` → revert.
    let by_stranger = slice
        .seller
        .call(slice.market.id(), "update_fee_bps")
        .args_json(json!({ "fee_bps": 500 }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    assert!(
        by_stranger.is_failure(),
        "update_fee_bps non-owner wajib revert"
    );

    // (2) owner sukses sampai cap.
    let by_owner = slice
        .owner
        .call(slice.market.id(), "update_fee_bps")
        .args_json(json!({ "fee_bps": 500 }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    by_owner.clone().into_result()?;
    assert_eq!(
        event_payload(&by_owner, "fee_update")["new_fee_bps"],
        json!(500)
    );
    let fee_bps: u16 = slice
        .owner
        .view(slice.market.id(), "get_fee_bps")
        .await?
        .json()?;
    assert_eq!(fee_bps, 500, "get_fee_bps() == 500");

    // (3) di atas cap → revert (INV-004).
    let above_cap = slice
        .owner
        .call(slice.market.id(), "update_fee_bps")
        .args_json(json!({ "fee_bps": 501 }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    assert!(
        above_cap.is_failure(),
        "fee > MAX_FEE_BPS wajib revert (INV-004)"
    );

    // (4) non-owner `update_treasury` → revert; (5) owner sukses.
    let treasury_stranger = slice
        .seller
        .call(slice.market.id(), "update_treasury")
        .args_json(json!({ "treasury": slice.seller.id() }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    assert!(
        treasury_stranger.is_failure(),
        "update_treasury non-owner wajib revert"
    );

    let treasury_owner = slice
        .owner
        .call(slice.market.id(), "update_treasury")
        .args_json(json!({ "treasury": slice.treasury.id() }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    treasury_owner.clone().into_result()?;
    assert_eq!(
        event_payload(&treasury_owner, "treasury_update")["new_treasury"],
        json!(slice.treasury.id().as_str())
    );
    let treasury: String = slice
        .owner
        .view(slice.market.id(), "get_treasury")
        .await?
        .json()?;
    assert_eq!(treasury, slice.treasury.id().as_str());
    Ok(())
}

// --- INV-004 / INV-027 · split pada fee cap & royalti cap --------------------

/// INV-004 · INV-027 — fee di cap (5%) dan royalti di cap (10%) tetap menghasilkan split
/// konsisten: `fee + royalti + proceeds == harga` persis, royalti ≤ 10% harga.
///
/// Menutup jalur "parameter non-default" yang tidak dibuktikan TC-002: split dihitung dari nilai
/// yang benar-benar tersimpan di kontrak, bukan konstanta test.
#[tokio::test]
async fn inv_004_and_027_split_holds_at_caps() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = Slice::with_caps(500, 1000)
        .await
        .expect("fixture fee+royalti cap");
    slice.open_public_phase(10, 5).await?;
    let buyer = slice.new_buyer().await?;

    let token_id = slice.mint_and_list(&slice.seller, PRICE).await?;

    let before = slice.balances(&buyer).await?;
    let result = slice.buy(&buyer, &token_id, PRICE).await?;
    result.clone().into_result()?;
    let deltas = BalanceDelta::between(before, slice.balances(&buyer).await?);

    let fee = PRICE * 500 / 10_000;
    let royalty = PRICE * 1000 / 10_000;
    assert_eq!(deltas.treasury, fee as i128, "fee 5% (cap) → treasury");
    assert_eq!(
        deltas.creator, royalty as i128,
        "royalti 10% (cap) → kreator (INV-027)"
    );
    assert_eq!(deltas.seller, (PRICE - fee - royalty) as i128);
    assert_eq!(
        deltas.treasury + deltas.creator + deltas.seller,
        PRICE as i128,
        "fee + royalti + proceeds == harga (INV-001)"
    );
    assert!(
        royalty * 10 <= PRICE,
        "royalti per token ≤ 10% harga (INV-027)"
    );
    Ok(())
}

// --- INV-023 / INV-026 · penolakan buy --------------------------------------

/// INV-023 — self-buy ditolak (pemanggil = seller).
#[tokio::test]
async fn inv_023_self_buy_is_rejected() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let token_id = slice.mint_and_list(&slice.seller, PRICE).await?;

    let result = slice.buy(&slice.seller, &token_id, PRICE).await?;
    assert!(result.is_failure(), "self-buy wajib revert (INV-023)");
    assert!(
        failure_message(&result).contains("FORBIDDEN_SELF_BUY"),
        "kode registry: {}",
        failure_message(&result)
    );
    assert!(
        slice.sale(&token_id).await?.is_some(),
        "listing tetap ACTIVE setelah self-buy ditolak"
    );
    Ok(())
}

/// INV-026 — private listing hanya untuk `allowed_buyer`.
#[tokio::test]
async fn inv_026_private_listing_rejects_other_buyers() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let allowed = slice.new_buyer().await?;
    let stranger = slice.new_buyer().await?;

    let token_id = slice.mint_one(&slice.seller).await?;
    slice.approve_market(&slice.seller, &token_id).await?;
    slice
        .list(&slice.seller, &token_id, PRICE, Some(allowed.id()))
        .await?
        .into_result()?;

    let rejected = slice.buy(&stranger, &token_id, PRICE).await?;
    assert!(rejected.is_failure(), "buyer lain wajib ditolak (INV-026)");
    assert!(
        failure_message(&rejected).contains("FORBIDDEN_BUYER"),
        "kode registry: {}",
        failure_message(&rejected)
    );

    slice.buy(&allowed, &token_id, PRICE).await?.into_result()?;
    assert_eq!(
        slice.token_owner(&token_id).await?,
        Some(allowed.id().clone())
    );
    Ok(())
}

// --- TC-053 / TC-006 · stale dua kasus (INV-016) -----------------------------

/// TC-053 · INV-016 kasus B — approval dicabut tanpa memindahkan token.
///
/// Token tetap milik seller, tapi approval ke market hilang → settlement **berhenti** dan
/// buyer di-refund penuh (bukan NFT berpindah tanpa bayaran).
#[tokio::test]
async fn tc_053_stale_when_approval_revoked_refunds_buyer() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let buyer = slice.new_buyer().await?;

    let token_id = slice.mint_and_list(&slice.seller, PRICE).await?;

    // Seller mencabut approval — token TIDAK pindah.
    slice
        .seller
        .call(slice.collection.id(), "nft_revoke")
        .args_json(json!({ "token_id": token_id, "account_id": slice.market.id() }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    assert!(
        slice.approval_id_of(&token_id).await?.is_none(),
        "approval tercabut"
    );

    let before = slice.balances(&buyer).await?;
    let result = slice.buy(&buyer, &token_id, PRICE).await?;
    result.clone().into_result()?;
    let deltas = BalanceDelta::between(before, slice.balances(&buyer).await?);

    // Buyer menerima refund penuh: kerugiannya hanya gas, jauh di bawah harga.
    assert!(
        deltas.buyer > -(PRICE as i128) / 100,
        "refund penuh ke buyer, hanya gas yang hangus (INV-016): {}",
        deltas.buyer
    );
    assert_eq!(deltas.seller, 0, "tidak ada distribusi saat stale");
    assert_eq!(deltas.treasury, 0, "tidak ada fee saat stale");
    assert_eq!(
        slice.token_owner(&token_id).await?,
        Some(slice.seller.id().clone()),
        "NFT tidak pindah saat stale"
    );
    assert!(
        slice.sale(&token_id).await?.is_none(),
        "listing mati saat stale"
    );

    let stale = event_payload(&result, "market_stale_detected");
    assert_eq!(stale["reason"], json!("approval_revoked"));
    assert_eq!(stale["detected_owner"], json!(slice.seller.id().as_str()));
    Ok(())
}

/// TC-006 · INV-016 kasus A — kepemilikan pindah di luar market.
///
/// Sejak **TASK-036** diperbaiki (ronde 29), `nft_transfer` atas token yang masih di-approve
/// berjalan normal — jadi test ini memakai jalur yang sesungguhnya: seller memindahkan token
/// **tanpa** mencabut approval lebih dulu, persis seperti di rantai nyata. Yang diuji tetap
/// kasus A: **kepemilikan token berpindah ke akun lain** sementara entry `Sale` masih ada, dan
/// market menolak settle.
#[tokio::test]
async fn tc_006_stale_when_ownership_moved_refunds_buyer() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let buyer = slice.new_buyer().await?;
    let new_owner = slice.new_buyer().await?;

    let token_id = slice.mint_and_list(&slice.seller, PRICE).await?;

    // Seller memindahkan token di luar market — mis. menjualnya lewat jalur lain. Approval ke
    // market masih terpasang; kontrak yang mencabutnya saat transfer (TASK-036).
    slice
        .seller
        .call(slice.collection.id(), "nft_transfer")
        .args_json(json!({
            "receiver_id": new_owner.id(),
            "token_id": token_id,
            "approval_id": null,
            "memo": null,
        }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    assert_eq!(
        slice.token_owner(&token_id).await?,
        Some(new_owner.id().clone()),
        "kepemilikan sudah pindah sebelum buy"
    );
    assert!(
        slice.sale(&token_id).await?.is_some(),
        "entry Sale masih ada — inilah kondisi stale"
    );

    let before = slice.balances(&buyer).await?;
    let result = slice.buy(&buyer, &token_id, PRICE).await?;
    result.clone().into_result()?;
    let deltas = BalanceDelta::between(before, slice.balances(&buyer).await?);

    assert!(
        deltas.buyer > -(PRICE as i128) / 100,
        "buyer tidak kehilangan harga (refund penuh, INV-016): {}",
        deltas.buyer
    );
    assert_eq!(deltas.seller, 0);
    assert_eq!(
        slice.token_owner(&token_id).await?,
        Some(new_owner.id().clone()),
        "NFT tetap di pemilik baru — market tidak memindahkannya"
    );

    let stale = event_payload(&result, "market_stale_detected");
    assert_eq!(
        stale["reason"],
        json!("ownership_mismatch"),
        "ownership mismatch diputuskan lebih dulu daripada approval (INV-016)"
    );
    assert_eq!(stale["detected_owner"], json!(new_owner.id().as_str()));
    Ok(())
}

// --- TC-016 / TC-017 · race & double-submit (INV-007/008/009) ----------------

/// TC-016 · INV-007/008/009 — 20 pembeli merebut satu listing: tepat 1 menang, 19 dana kembali.
///
/// Pemenang ditentukan **urutan eksekusi on-chain** (satu shard akun market), bukan RPC
/// (docs/development/concurrency-and-races.md §1–§3). Yang kalah revert → deposit Ⓝ tidak
/// berpindah, hanya gas yang hangus.
#[tokio::test]
async fn tc_016_race_twenty_buyers_single_winner() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;

    let token_id = slice.mint_and_list(&slice.seller, PRICE).await?;

    let mut buyers = Vec::new();
    for _ in 0..20 {
        buyers.push(slice.new_buyer().await?);
    }

    let before = slice
        .snapshot_accounts(buyers.iter().map(|b| b.id()))
        .await?;

    let attempts = futures::future::join_all(
        buyers
            .iter()
            .map(|buyer| slice.buy(buyer, &token_id, PRICE)),
    )
    .await;

    let mut winners = Vec::new();
    for (index, attempt) in attempts.into_iter().enumerate() {
        let result = attempt?;
        if result.is_success() && result.receipt_failures().is_empty() {
            winners.push(index);
        } else {
            assert!(
                failure_message(&result).contains("CONFLICT_SOLD"),
                "pembeli kalah harus revert CONFLICT_SOLD, dapat: {}",
                failure_message(&result)
            );
        }
    }

    assert_eq!(winners.len(), 1, "tepat satu pembeli menang (INV-007/008)");

    let winner = &buyers[winners[0]];
    assert_eq!(
        slice.token_owner(&token_id).await?,
        Some(winner.id().clone()),
        "NFT pindah ke pemenang"
    );
    assert!(slice.sale(&token_id).await?.is_none(), "listing terjual");
    assert!(
        slice.pending_purchase(&token_id).await?.is_none(),
        "tidak ada pembelian tertinggal (INV-031)"
    );

    // Yang kalah: deposit kembali — kerugian hanya gas, jauh di bawah harga.
    let after = slice
        .snapshot_accounts(buyers.iter().map(|b| b.id()))
        .await?;
    let gas_ceiling = PRICE / 100;
    for (index, buyer) in buyers.iter().enumerate() {
        if index == winners[0] {
            continue;
        }
        let loss = before[index].saturating_sub(after[index]);
        assert!(
            loss < gas_ceiling,
            "pembeli kalah #{} ({}) kehilangan {loss} yocto — deposit wajib kembali penuh",
            index,
            buyer.id()
        );
    }
    Ok(())
}

/// TC-017 · INV-007/008 — double-submit pembeli yang sama: tx kedua revert tanpa efek ganda.
#[tokio::test]
async fn tc_017_double_submit_second_purchase_reverts() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let buyer = slice.new_buyer().await?;

    let token_id = slice.mint_and_list(&slice.seller, PRICE).await?;

    slice.buy(&buyer, &token_id, PRICE).await?.into_result()?;
    assert_eq!(
        slice.token_owner(&token_id).await?,
        Some(buyer.id().clone())
    );

    let second = slice.buy(&buyer, &token_id, PRICE).await?;
    assert!(second.is_failure(), "buy kedua wajib revert (INV-007/008)");
    assert!(
        failure_message(&second).contains("CONFLICT_SOLD"),
        "kode registry: {}",
        failure_message(&second)
    );

    // Tidak ada efek ganda: token tetap satu pemilik, tidak ada pending tertinggal.
    assert_eq!(
        slice.token_owner(&token_id).await?,
        Some(buyer.id().clone())
    );
    assert!(slice.pending_purchase(&token_id).await?.is_none());
    Ok(())
}

// --- TASK-036 · transfer token ter-approve (temuan F1) -----------------------

/// TASK-036 · regression test sandbox — `nft_transfer` atas token yang **masih di-approve**
/// ke penerima terdaftar yang belum memegang token. Dulu gagal `ExcessiveUnlockError`
/// (storage-accounting NEP-145 di koleksi). AC TASK-036 menuntut bukti di rantai sungguhan,
/// bukan hanya unit — karena itu test ini ada di suite sandbox, bukan di `contract/src/lib.rs`.
#[tokio::test]
async fn task_036_transfer_approved_token_to_fresh_receiver_succeeds() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let receiver = slice.new_buyer().await?;

    let token_id = slice.mint_one(&slice.seller).await?;
    // Token sekarang punya approval aktif ke market; receiver belum pernah pegang token.
    let _approval_id = slice.approve_market(&slice.seller, &token_id).await?;
    assert_eq!(
        slice.approval_id_of(&token_id).await?,
        Some(_approval_id),
        "prasyarat: token benar-benar punya approval aktif"
    );

    let result = slice
        .seller
        .call(slice.collection.id(), "nft_transfer")
        .args_json(json!({
            "receiver_id": receiver.id(),
            "token_id": token_id,
            "approval_id": null,
            "memo": null,
        }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    result.clone().into_result()?;

    assert_eq!(
        slice.token_owner(&token_id).await?,
        Some(receiver.id().clone()),
        "transfer token ter-approve ke penerima baru berhasil (TASK-036)"
    );
    // Approval lama ikut dicabut oleh transfer (NEP-178/INV-011).
    assert_eq!(
        slice.approval_id_of(&token_id).await?,
        None,
        "approval dicabut setelah transfer (INV-011)"
    );
    Ok(())
}

// --- TC-054 · recovery dana nyangkut (INV-031) ------------------------------

/// TC-054 · INV-031 — pembelian nyangkut dipulihkan **siapa pun** setelah jeda blok.
///
/// Kondisi nyangkut di-*inject* dengan menulis entri `pending_purchases` langsung ke state
/// sandbox (precondition TC-054: "inject: callback revert / gas habis") — jalur ini tidak bisa
/// dicapai lewat kontrak NearSea sendiri karena `resolve_purchase` selalu menutup pending
/// (docs/contracts/market.md §3b). Yang diuji: **kontrak pemulihannya** — gerbang jeda blok,
/// refund penuh tanpa governance, dan pemulihan listing.
#[tokio::test]
async fn tc_054_stuck_purchase_is_recovered_permissionlessly() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let buyer = slice.new_buyer().await?;
    let rescuer = slice.worker.dev_create_account().await?;

    let token_id = slice.mint_and_list(&slice.seller, PRICE).await?;
    let approval_id = slice.sale(&token_id).await?.expect("listing ACTIVE")["approval_id"].as_u64();

    // Reproduksi keadaan setelah `buy` ditulis: Sale dihapus, pending hidup, dana di kontrak.
    slice
        .seller
        .call(slice.market.id(), "remove_sale")
        .args_json(json!({
            "nft_contract_id": slice.collection.id(),
            "token_id": token_id,
        }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    buyer
        .transfer_near(slice.market.id(), NearToken::from_yoctonear(PRICE))
        .await?
        .into_result()?;

    let height = slice.worker.view_block().await?.height();
    inject_stuck_purchase(&slice, &token_id, buyer.id(), approval_id, height).await?;

    assert!(
        slice.pending_purchase(&token_id).await?.is_some(),
        "entri pending ter-inject"
    );
    assert!(
        slice.sale(&token_id).await?.is_none(),
        "Sale terhapus (optimistic removal)"
    );

    // (1) Sebelum jeda → revert (terlalu dini).
    let too_early = rescuer
        .call(slice.market.id(), "recover_stuck_purchase")
        .args_json(json!({
            "nft_contract_id": slice.collection.id(),
            "token_id": token_id,
        }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    assert!(
        too_early.is_failure(),
        "recovery sebelum jeda wajib revert (INV-031)"
    );

    // (2) Setelah jeda → akun ketiga memulihkan; buyer refund penuh, listing hidup lagi.
    slice.worker.fast_forward(30).await?;
    let before = slice.balance_of(buyer.id()).await?;

    let recovered = rescuer
        .call(slice.market.id(), "recover_stuck_purchase")
        .args_json(json!({
            "nft_contract_id": slice.collection.id(),
            "token_id": token_id,
        }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    recovered.clone().into_result()?;
    slice.quiesce().await?;
    let after = slice.balance_of(buyer.id()).await?;

    assert_eq!(
        after - before,
        PRICE,
        "buyer menerima refund penuh tanpa bergantung governance (INV-031)"
    );
    assert!(
        slice.pending_purchase(&token_id).await?.is_none(),
        "entri pending dihapus setelah recovery"
    );

    let event = event_payload(&recovered, "market_purchase_recovered");
    assert_eq!(event["buyer"], json!(buyer.id().as_str()));
    assert_eq!(event["refund_yocto"], json!(PRICE.to_string()));
    assert_eq!(
        event["sale_restored"],
        json!(true),
        "listing dipulihkan (token masih milik seller)"
    );
    assert!(
        slice.sale(&token_id).await?.is_some(),
        "listing bisa dijual lagi setelah recovery"
    );
    Ok(())
}

// --- TC-048 · nft_on_approve (INV-013) ---------------------------------------

/// TC-048 · INV-013 — notifikasi `nft_on_approve` palsu tidak boleh menciptakan listing.
///
/// Yang dijamin kontrak (ADR-002): listing **hanya** lahir dari `list_nft_for_sale` + dual
/// verification, dan `nft_on_approve` tidak menulis state apa pun — jadi penelepon palsu tidak
/// punya efek. Payload yang bentuknya tidak sah tetap ditolak.
#[tokio::test]
async fn tc_048_spoofed_nft_on_approve_creates_no_listing() -> anyhow::Result<()> {
    let _guard = guard().await;
    let slice = slice().await;
    let attacker = slice.worker.dev_create_account().await?;

    let token_id = slice.mint_one(&slice.seller).await?;

    // Payload NEP-178 berbentuk sah, dikirim akun biasa (bukan koleksi).
    let spoofed = attacker
        .call(slice.market.id(), "nft_on_approve")
        .args_json(json!({
            "token_id": token_id,
            "owner_id": slice.seller.id(),
            "approval_id": 0,
            "msg": PRICE.to_string(),
        }))
        .max_gas()
        .transact()
        .await?;

    assert!(
        slice.sale(&token_id).await?.is_none(),
        "notifikasi palsu TIDAK boleh menciptakan listing (ADR-002)"
    );
    let supply: u64 = slice
        .owner
        .view(slice.market.id(), "get_supply_sales")
        .await?
        .json()?;
    assert_eq!(
        supply, 0,
        "tidak ada listing tercipta dari notifikasi palsu"
    );
    assert!(
        spoofed.receipt_failures().is_empty() || spoofed.is_failure(),
        "panggilan notifikasi dievaluasi"
    );

    // Payload yang bentuknya tidak sah wajib ditolak.
    let empty_msg = attacker
        .call(slice.market.id(), "nft_on_approve")
        .args_json(json!({
            "token_id": token_id,
            "owner_id": slice.seller.id(),
            "approval_id": 0,
            "msg": "",
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(empty_msg.is_failure(), "msg kosong wajib revert (INV-013)");
    Ok(())
}

// --- helper: injeksi state untuk TC-054 --------------------------------------

/// Tulis entri `pending_purchases` langsung ke state sandbox (keadaan "`buy` sudah jalan tapi
/// callback settle mati").
///
/// Kunci `LookupMap` = prefix diskriminan enum `StorageKey` + borsh `SaleKey`; layout ini
/// di-verifikasi terhadap state nyata sandbox sebelum dipakai (market.md §7).
async fn inject_stuck_purchase(
    slice: &Slice,
    token_id: &str,
    buyer: &AccountId,
    approval_id: Option<u64>,
    created_height: u64,
) -> anyhow::Result<()> {
    let key = (slice.collection.id().clone(), token_id.to_string());
    let mut pending_key = vec![PREFIX_PENDING_PURCHASES];
    key.serialize(&mut pending_key)?;

    let pending = nearsea_market::PendingPurchase {
        buyer: buyer.clone(),
        deposit: PRICE,
        created_height,
        fee_bps: FEE_BPS,
        sale: nearsea_market::Sale {
            nft_contract_id: slice.collection.id().clone(),
            token_id: token_id.to_string(),
            owner_id: slice.seller.id().clone(),
            approval_id,
            price_yocto: PRICE.into(),
            allowed_buyer: None,
            listed_at: 0,
        },
    };
    let mut pending_value = Vec::new();
    pending.serialize(&mut pending_value)?;

    slice
        .worker
        .patch(slice.market.id())
        .state(&pending_key, &pending_value)
        .transact()
        .await?;
    Ok(())
}

/// Helper baca saldo banyak akun pada satu titik waktu (setelah rantai ditenangkan).
impl Slice {
    pub async fn snapshot_accounts<'a, I>(&self, accounts: I) -> anyhow::Result<Vec<u128>>
    where
        I: IntoIterator<Item = &'a AccountId>,
    {
        self.quiesce().await?;
        let mut balances = Vec::new();
        for account in accounts {
            balances.push(self.balance_of(account).await?);
        }
        Ok(balances)
    }
}
