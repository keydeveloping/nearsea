# 10: FE — browse / list / buy

**What to build:** Demo M1 yang bisa dilihat orang: dua akun testnet membuka aplikasi, satu mem-mint dan memasang NFT untuk dijual lewat dua langkah yang jelas, satu lagi membelinya — dan breakdown pembayaran (royalti kreator + fee 2%) terlihat di UI. Inilah tiket yang menutup tesis slice.

**Blocked by:** 09 (wallet connect) dan 07 (kontrak lengkap). Dua jalur berbeda — FE butuh wallet, dan jalur emas butuh kontrak nyata untuk dijalankan end-to-end.

**Status:** in-progress — semua AC terimplementasi & teruji kecuali jalur emas Playwright, yang terkunci di belakang deploy testnet (wajib persetujuan user).

- [x] Grid/browse menampilkan listing yang dibaca langsung dari kontrak via view call RPC.
      → `ListingGrid` + `useListings` → `get_sales` lewat `wallet.viewFunction` (provider RPC
      near-connect, failover). Urutan deterministik `newest` (`listed_at` desc, tie-break
      `token_id` asc) karena urutan dari kontrak tidak dijamin (iterasi map).
- [x] Halaman detail NFT menampilkan metadata, harga, dan status listing.
      → `TokenPage` + `useTokenDetail` (`nft_token` + `get_sale`): judul, deskripsi, koleksi,
      owner, status (listed/not listed/stale), harga, dan media.
- [x] Pemilik token melihat tombol Sell; bukan-pemilik melihat Buy.
      → `TokenActions` membandingkan `accountId` dengan `token.ownerId`; self-buy di-disable
      (kontrak tetap menolak dengan `FORBIDDEN_SELF_BUY`).
- [x] Alur listing memakai modal **dua langkah** dengan progres yang menjelaskan langkah 1/2 (approve, lalu list) — bukan satu tombol yang tiba-tiba minta dua tanda tangan.
      → `ListModal` + `useListNft`: harga diisi di **form**, lalu `nft_approve` ("Step 1 of 2:
      Approve the marketplace") dan `list_nft_for_sale` ("Step 2 of 2: Confirm the listing").
      `approval_id` konkret dibaca dari `nft_token` di antara keduanya — mengirim `null` membuat
      listing mustahil dibeli (`nft_transfer_payout` memetakan `None` ke otorisasi owner).
      Deposit storage NEP-145 ditampilkan di modal sebelum signing (temuan UX B4).
- [x] Alur buy mengirim deposit = harga dan menampilkan **breakdown fee + royalti** sebelum konfirmasi.
      → `BuyModal` + `useBuyNft`. Deposit = harga hasil **verifikasi ulang** (bukan harga cache).
      Breakdown dari `get_fee_bps` + `royalty_config`, aritmetika BigInt (`lib/format/fees.ts`),
      dan ditandai sebagai estimasi (jumlah otoritatif = payout on-chain).
- [x] Hasil transaksi jelas: sukses dan gagal sama-sama memberi umpan balik yang terbaca (bukan diam).
      → `TxFeedback`: sukses `role="status"`, gagal `role="alert"` + Retry/Dismiss. Panic kontrak
      dipetakan ke kode registry lewat `classifyTxError` (teks mentah tidak pernah tampil).
- [x] Saat transaksi menunggu, tombol disabled + indikator progres; setelah final, data di-refresh.
      → Tombol `disabled` + `aria-busy`; modal tidak bisa ditutup (Esc/latar) selama transaksi.
      Invalidasi query dijalankan **setelah receipt final**, bukan saat submit.
- [x] Listing stale disembunyikan dari discovery; halaman token menampilkan status "tidak lagi tersedia" bila stale.
      → `discovery.ts` memeriksa **dua** kasus stale (kepemilikan pindah **atau** approval
      dicabut/diterbitkan ulang) lewat satu `nft_token` per listing. Kegagalan RPC **tidak**
      menyembunyikan listing ("tidak tahu" ≠ "basi"). Halaman token menampilkan
      "No longer available" + notice, bukan tombol Buy.
- [x] Semua angka Ⓝ diformat dari string yoctoNEAR — **tidak ada aritmetika float** pada nilai yocto.
      → `YoctoNear` bertipe; `formatNear`/`bpsOf`/`percentFromBps`/`parseNearInput` semuanya BigInt.
      Test: fee 2% dari 10 Ⓝ = `200000000000000000000000`, dan Σ(fee+royalti+sisa) = harga.
- [x] Semua string UI lewat `i18n/` (bahasa Inggris); tidak ada string hardcoded di komponen.
      → Namespace baru `marketplace` + `errors` (katalog penuh dari `error-handling.md` §8).
      Kunci bertipe: kunci salah = error build. Kode error ditulis **verbatim** (`errors.CONFLICT_SOLD`).
- [x] Metadata dari luar dirender lewat `<img>` tanpa `innerHTML`.
      → `TokenMedia` hanya `<img>` untuk URL **https**; `data:`/`javascript:`/`blob:` ditolak dan
      digantikan placeholder. Judul/deskripsi dirender sebagai teks React (escaped). Tidak ada
      `dangerouslySetInnerHTML` di seluruh repo.
- [ ] Jalur emas Playwright hijau: browse → list → buy dengan dua akun testnet.
      → **Belum.** Playwright belum ada di `frontend/` dan kontrak belum ter-deploy di testnet;
      deploy testnet wajib persetujuan user (`git-workflow.md` §3). Yang ada sekarang: jalur emas
      **Vitest + RTL** (`golden-path.test.tsx`, 20 test) yang menjalankan komponen & hook yang
      sama terhadap chain palsu — tanpa browser, tanpa wallet nyata, tanpa dua akun.

**Done-when (TASK-008):** Browse->list->buy end-to-end via UI (jalur emas Playwright).
→ Terimplementasi dan terverifikasi di level komponen/hook; bukti end-to-end dengan dua akun
testnet menyusul setelah deploy testnet disetujui.

**Bukti & catatan implementasi (ronde 26):**
- **Lokasi**: `frontend/features/marketplace/` (`ExplorePage.tsx`, `TokenPage.tsx`,
  `components/{ListingGrid,ListingCard,ListModal,BuyModal,TokenActions,TokenMedia,PriceChip,TxFeedback}.tsx`,
  `hooks/{useListings,useTokenDetail,useSaleConfig,useListNft,useBuyNft,useStorageRequirement,useMarketChain,useMountedRef}.ts`,
  `api/{market-api,discovery,sale-validity,verify-sale,query-keys}.ts`,
  `types/marketplace.types.ts`, `error-messages.ts`, `test-utils.tsx`) +
  `frontend/lib/format/{fees,storage,media}.ts`, `frontend/lib/errors/market-errors.ts`,
  `frontend/lib/near/contracts.ts`, `frontend/lib/constants/near-gas.ts`,
  `frontend/components/ui/Modal.tsx`, `frontend/app/{QueryProvider,MarketChainProvider}.tsx`,
  `frontend/app/token/[contract]/[tokenId]/page.tsx`, `i18n/en/{marketplace,errors}.json`.
- **Prasyarat baru**: `@tanstack/react-query` 5.104.1 (di-pin exact) — `frontend-architecture.md`
  §Data fetching mewajibkannya untuk server state; `zustand` belum dipakai karena belum ada state
  UI lintas-halaman yang butuh store global.
- **Pemisahan fitur ↔ wallet**: `features/marketplace` **tidak** mengimpor `features/auth`
  (dilarang `frontend-architecture.md` §1). Wallet disuntikkan dari `app/MarketChainProvider.tsx`
  lewat interface sempit `MarketChain`; `WalletApi` memenuhinya secara struktural.
- **Surface wallet diperluas**: `WalletApi` kini punya `viewFunction` + `callFunction`
  (diteruskan dari `useNearWallet()`, identitas stabil via ref seperti `getBalance`). Test double
  `near-connect.double.tsx` ikut memaparkan keduanya + pencatatan panggilan.
- **Temuan review kode (semuanya diperbaiki, dengan test):**
  1. **Royalti tak diketahui → angka seller menyesatkan.** `computeSaleBreakdown` menerima
     `royaltyBps: null` dan mengembalikan `null`, sehingga UI tidak menampilkan "seller receives"
     yang seolah royalti nol (melebihkan penerimaan seller).
  2. **Modal bisa ditutup saat transaksi berjalan** (Esc/latar) → progres & hasil hilang.
     Diperbaiki: prop `dismissible` dimatikan selama `verifying`/`signing`.
  3. **Fokus tidak dikembalikan** ke pemicu saat modal tutup, dan label dialog memakai
     `aria-label` alih-alih `aria-labelledby` (`04-ux-ui-spec.md` §Accessibility). Diperbaiki.
  4. **Error ad-hoc** (`new Error("CHAIN_REVERT: …")`) di jalur approval. Diperbaiki:
     `KnownTxError` membawa kode registry langsung, tanpa teks yang harus ditebak regex.
  5. **Duplikasi**: invalidasi query & penanda mount disalin di dua hook → `invalidateMarketQueries`
     + `useMountedRef`. Verifikasi pra-sign dipindah ke `api/verify-sale.ts` (bisa diuji sendiri).
  6. **Ekspor/kunci spekulatif** dihapus (`ListingView`, `VerifiedSale`, `notFoundFailure`,
     `marketKeys.listing`, key i18n sort/`walletHint`/`tx.pending`/`tx.failed`/`connect.hint`).
  7. **Penamaan file komponen** (`app/providers.tsx` → `QueryProvider.tsx`,
     `app/market-chain-provider.tsx` → `MarketChainProvider.tsx`) mengikuti `code-standards.md` §6;
     urutan impor eksternal→internal diperbaiki.
- **Koreksi dokumen**: klaim tiket 09 bahwa kode `function-call-key-plugin` ter-tree-shake dari
  bundle **tidak benar** — `createLocalKeyFor`/`access_key::plugin`/`ed25519:` tetap ada di chunk
  klien (diverifikasi pada build `c7e081b` **dan** build sesudahnya). Jalurnya tetap tidak aktif
  (tidak ada pemanggil), tetapi TASK-037 tidak boleh ditutup dengan alasan tree-shaking.
- **Belum diklaim**: jalur emas Playwright dua akun testnet; sort/filter/search UI (key & tipe
  sudah dihapus sampai UI-nya benar-benar ada); pagination cursor (MVP membaca satu halaman
  `get_sales` maksimum 100 entry); offers/bundles (TASK-009/010, belum ada di kontrak).

**Spec:** [docs/features/marketplace.md](../../../docs/features/marketplace.md) §UI · [docs/architecture/frontend-architecture.md](../../../docs/architecture/frontend-architecture.md) §2–§4, §8–§9 · [docs/04-ux-ui-spec.md](../../../docs/04-ux-ui-spec.md) · [tasks/milestones.md](../../../tasks/milestones.md) §Demo M1
