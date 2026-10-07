# Market Research — Realita Pasar NFT NEAR (riset 2026-10-07)

> **Status: BUKTI RISET — bukan keputusan.** Dokumen ini mencatat temuan pasar yang **bertentangan
> dengan premis produk** di [01-PRD.md](../01-PRD.md) §2. Per aturan repo (AGENTS.md), kontradiksi
> premis → wajib diangkat ke user sebelum keputusan; dokumen ini bahan untuk itu.
>
> **Label**: `FACT` = terverifikasi sumber primer / pengecekan langsung · `INFERENCE` = penalaran
> dari fakta · `UNVERIFIED` = klaim pihak ketiga, belum dikonfirmasi independen.
> Tanggal akses semua pengecekan langsung: **2026-10-07**.

## 1. Ringkasan (headline)

**Masalahnya bukan "belum ada marketplace NFT bagus di NEAR" — masalahnya tidak ada permintaan.**
Marketplace dominan NEAR (MITTE, klaim ~91% pangsa) tutup April 2025. Yang tersisa hanya HotCraft,
dengan volume koleksi teratasnya ratusan dolar per minggu.

Klaim di [01-PRD.md](../01-PRD.md) §2 — *"Ekosistem NEAR belum punya marketplace NFT kelas dunia"* —
secara teknis benar tapi **menyesatkan**: bukan karena belum ada yang membangun, melainkan karena
**beberapa tim berdana besar sudah membangun dan semuanya mati.**

## 2. Status marketplace (terverifikasi 2026-10-07)

| Marketplace | Status | Bukti |
|---|---|---|
| **Paras** (dulu andalan NEAR) | **Mati — pivot.** `paras.id` → 301 → `new.paras.id` → 301 → **`narrativeprotocol.com`** (engine AI world-state, bukan NFT) | **FACT** — redirect saya ikuti langsung, 2026-10-07 |
| **MITTE** (klaim ~91% pangsa NEAR, 35k MAU) | **Tutup 2025-04-10** — orderbook + meme launchpad | FACT (Learn NEAR Club) |
| **Mintbase** | **Tidak melayani** — `mintbase.xyz` menampilkan "deployment temporarily paused"; snapshot 2026-03-30 "Featured Contracts: 0" | FACT (cek langsung) |
| **Few and Far** (danai $10.5M + grant NEAR Foundation) | **Mati sebagai marketplace** — `fewandfar.xyz` → `fewfar.com`, sekarang blog konten kripto | FACT (cek langsung) |
| **TradePort** | **Drop NEAR** — homepage hanya Sui; `tradeport.xyz/near` → 404 | FACT (cek langsung) |
| **Magic Eden** | **Tidak pernah support NEAR** (Solana-native; EVM/Bitcoin ditutup 2026-03-09) | FACT |
| **Tensor** | Solana-only | FACT |
| **Ref Finance NFT** | Tidak ada marketplace NFT (Ref merge → RHEA Finance, DEX) | FACT |
| **HotCraft** (`hotcraft.art`) | **Satu-satunya yang hidup** melayani NEAR (chain-abstracted, bagian HOT Protocol) | FACT (cek langsung) |
| Apollo42, TofuNFT, nft.trade | Mati / domain tidak resolve | FACT |

## 3. Ukuran pasar (kecil dan menyusut)

- **Paras puncak (Mei 2022)**: ~$18M penjualan **kumulatif** sejak 2020, 12k artis, 50k kolektor — bukan per tahun. (FACT)
- **Paras Nov 2025**: ~$0,5 juta volume per bulan — dan ini disebut sebagai marketplace **utama** NEAR. (FACT)
- **HotCraft 2026-10-07**: koleksi teratas 7 hari = **$742 / 165 trade**; koleksi all-time teratas = **$6.811**. (FACT, cek langsung)
- **CryptoSlam**: NEAR **tidak muncul** di ranking koleksi. (FACT — saya cek langsung; NEAR absen dari daftar)
- **Global**: volume NFT 2025 ≈ **$5,5–5,6 miliar**, turun ~37% dari 2024 (~$8,8M), vs puncak **$23,7M (2022)**. Harga jual rata-rata < $100. (FACT)
- **Perhatian/capital NEAR sudah pindah** ke Intents ($29M+ kumulatif), DeFi/RHEA, dan launchpad memecoin — bukan NFT. (FACT)

## 4. Royalti: perangnya sudah selesai, dan penegakan kalah

| Tanggal | Kejadian |
|---|---|
| Okt 2022 | Blur rilis: fee 0%, **royalti opsional**, menang volume |
| Nov 2022 | OpenSea pasang Operator Filter (blokir marketplace tidak patuh) |
| Feb 2023 | OpenSea fee 0%, royalti minimum dipotong jadi 0,5% |
| Agu 2023 | OpenSea **matikan** Operator Filter; royalti jadi opsional |
| Mar 2024 | Royalti opsional untuk **semua** penjualan |
| Hasil | Pendapatan royalti kreator: **$269 juta (Jan 2023) → $2,4 juta (Sep 2023)** |

**Implikasi untuk NearSea** (INFERENCE): *"royalti dipaksa on-chain"* — yang di
[README.md](../../README.md) dan ADR-005 dijadikan **pembeda utama** — adalah posisi 2022 yang
**sudah kalah**. Blur menang dengan membuat royalti opsional; OpenSea kehilangan pangsa karena
memaksanya. Di NEAR, NEP-178 memungkinkan marketplace *kooperatif* membayar royalti, tapi tidak ada
yang bisa memaksa marketplace lain. Dengan pasar yang hampir kosong, penegakan royalti lebih
berisiko **menjauhkan** sedikit trader yang tersisa.

## 5. Pola kegagalan (INFERENCE dari fakta di atas)

1. Pasar yang kecil — volume NEAR tidak pernah melewati ratusan ribu dolar/bulan bahkan di puncak.
2. **Didanai grant lalu ditinggalkan** — MITTE, Few and Far ($10.5M + grant), Mintbase ($9.28M) semua mati.
3. Tidak ada pendapatan fee — pada ~$500k/bulan seluruh chain, fee 2% ≈ **$10k/bulan untuk SEMUA marketplace**.
4. Perhatian chain berpindah ke Intents/DeFi/memecoin/AI.
5. **Pemimpin pasar mati paling cepat** — MITTE punya ~91% pangsa dan tetap tutup ~18 bulan setelah rilis.
6. Tim pivot — Paras → Narrative Protocol; Magic Eden → Solana + casino.

## 6. Apakah ada celah? (penilaian jujur)

**Tidak ada celah dalam bentuk "marketplace lebih bagus".** Yang kurang bukan marketplace-nya, tapi
**pembeli dan penjualnya**. Bukti: pemimpin dengan 91% pangsa mati; venue yang tersisa ~$742/minggu.

Pembeda yang direncanakan NearSea **bukan hal baru** dan sebagian besar memetakan ke apa yang sudah gagal:

| Pembeda NearSea | Kenyataannya |
|---|---|
| Orderbook on-chain | = model MITTE (mati) |
| Fee 2% | Lebih murah dari apa pun tidak berarti bila volume ≈ $0 |
| Koleksi terbuka | Sudah ditawarkan Mintbase/TenK |
| Launchpad berphase | MITTE juga launchpad; energi launchpad NEAR sudah pindah ke **memecoin** |
| Royalti dipaksa on-chain | Kalah di perang royalti (lihat §4) |

**INFERENCE**: marketplace umum baru bersaing untuk kue beberapa ribu dolar per minggu, sementara
biayanya (kontrak, indexer, fetcher metadata, kerja keamanan, maintenance) tetap dan besar.
Ekspektasi pengguna ≈ 0–beberapa lusin; pendapatan ≈ nol.

## 7. Satu-satunya irisan yang mungkin

Bukan marketplace umum, tapi **kegunaan NFT yang captive dan non-spekulatif** yang *membutuhkan*
venue: game (mis. Pumpopoly — solo dev, bertahan karena produknya game, bukan karena network effect
marketplace), tiket/akses, kredensial, keanggotaan. Pembeda di situ bukan fitur marketplace, tapi
**audiens yang sudah ada**.

## 8. Yang harus diputuskan user (belum diputuskan)

1. Apakah premis produk di [01-PRD.md](../01-PRD.md) §2 diperbarui dengan bukti ini?
2. Kalau proyek dilanjutkan: apakah sebagai **produk** (perlu rencana permintaan/cold-start), atau
   sebagai **portofolio/pembelajaran** (ekspektasi pertumbuhan = nol, eksplisit)?
3. Kalau sebagai produk: apakah bentuknya tetap marketplace umum, atau diarahkan ke irisan §7?
4. Kalau sebagai pembelajaran/portofolio: apakah scope M1 dipotong (bundle & fitur yang tidak
   membuktikan inti bisa dibuang)?

> Per aturan repo: keputusan ini wajib dicatat sebagai ADR **sebelum** implementasi, karena
> bukti pasar ini bertentangan dengan premis yang saat ini tertulis di dokumen.

## 9. Keterbatasan riset ini

- Beberapa klaim bergantung pada ringkasan pihak ketiga (blog, thread forum) — ditandai `UNVERIFIED`.
- Volume HotCraft dibaca dari halaman publik, bukan API; bisa berubah.
- Tidak ada wawancara dengan mantan tim Paras/MITTE — alasan penutupan sebagian disimpulkan.
- Riset ini **tidak** menyimpulkan "jangan bangun"; ia menyimpulkan "jangan bangun **mengharap pengguna
  tanpa rencana permintaan**".
