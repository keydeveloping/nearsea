# 07: Market buy — settlement, split royalti + fee, refund

**What to build:** Buyer bisa membeli NFT yang ter-list. Market memindahkan NFT, membagi pembayaran (royalti kreator, fee platform 2%, sisanya ke seller), dan **otomatis me-refund buyer bila ada yang gagal**. Setiap deposit yang masuk punya jalan keluar — termasuk bila callback settlement mati.

**Blocked by:** 06

**Status:** done

- [x] `buy` dengan deposit ≥ harga memindahkan NFT ke buyer dan membayar kreator + treasury + seller dalam satu settlement.
- [x] Payout dari kontrak koleksi diperlakukan **untrusted**: jumlah penerima (`1..=10`), amount > 0, dan `Σpayout ≤ harga − fee` semuanya divalidasi. **Catatan**: klausa "sisa ≤ 1 yocto" dibatalkan (ronde 23) — sisa = proceeds seller, bukan dust.
- [x] Fee = `floor(harga × fee_bps / 10_000)`; `fee_bps` default 200 dan tidak bisa melampaui cap 500.
- [x] Payout tidak valid (receiver > batas, jumlah melebihi harga, amount 0, kosong) → **refund penuh buyer** dan listing dipulihkan.
- [x] Kelebihan deposit di-refund ke buyer.
- [x] Self-buy ditolak.
- [x] Listing stale ditolak saat buy — **kedua** kasus: kepemilikan sudah pindah **atau** approval sudah tidak valid.
- [x] Pembelian yang nyangkut (callback settlement gagal) bisa dipulihkan **oleh siapa pun** setelah jeda — buyer di-refund penuh, tanpa bergantung pada governance.
- [x] Event `market_sale` ter-emit dengan payout map final.
- [x] Anggaran gas `resolve_purchase` diverifikasi terhadap worst case (10 penerima payout + fee + seller + refund + event).
- [ ] Sandbox hijau: buy sukses, payout tidak valid → refund, dan **race 20 pembeli → tepat 1 menang, 19 refund penuh** (TC-002 paruh buy, TC-003, TC-016, TC-017, TC-022, TC-054). → **sebagian**: paruh buy + TC-003 + TC-054 dibuktikan di **unit** (39 test baru); race TC-016/017, TC-022, TC-048 versi sandbox = **tiket 08** (butuh dua kontrak nyata).

**Done-when (TASK-005):** Buy sukses + refund + race 20 pembeli lolos (INV-001/016, TC-003/016).
→ Buy sukses + refund **lolos di unit** (116 test workspace, `fmt`/`clippy -D warnings` bersih, wasm 239 KB). Race 20 pembeli = tiket 08.

**Bukti & catatan implementasi (ronde 23):**
- `buy` → `process_purchase` (dual verification saat settle) → `nft_transfer_payout` → `resolve_purchase`. Verifikasi stale/ownership **tidak bisa sinkron** (view XCC = receipt terpisah), jadi stale = refund penuh + event `market_stale_detected`, bukan panic tx — dana buyer tidak pernah tertahan.
- **Dua koreksi dokumen** (temuan saat implementasi): (1) `withdraw_fees` dihapus — fee ditransfer langsung ke treasury saat settlement; (2) aturan `sisa ≤ 1 yocto` (INV-002) dibatalkan — residual = proceeds seller. Detail: [docs/contracts/market.md](../../../docs/contracts/market.md) §3/§4a, [docs/features/payments.md](../../../docs/features/payments.md) §Algoritma.
- Test unit menutup: happy path dengan angka exact (Σ keluar == Σ masuk), kelebihan deposit refund, private listing, self-buy, deposit kurang, `CONFLICT_SOLD` (sale/pending), stale dua kasus (TC-006/TC-053), verifikasi tak pasti, 6 jalur payout invalid → refund, recovery permissionless dengan/tanpa restore (TC-054), paused, anggaran gas worst case.

**Spec:** [docs/contracts/market.md](../../../docs/contracts/market.md) §3, §3a, §3b · [docs/features/payments.md](../../../docs/features/payments.md) · [docs/development/concurrency-and-races.md](../../../docs/development/concurrency-and-races.md) · INV-001/002/003/009/016/031 · SEC-ORDER-001
