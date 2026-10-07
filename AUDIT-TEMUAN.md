# AUDIT TEMUAN — Review Mendalam Semua Dokumen (71 file)

> Metode: 11 subagent, 4 gelombang (maks 3 concurrent), setiap file dibaca PENUH dan dipahami (bukan grep).
> Tanggal audit: 2026-10-01. Semua temuan diverifikasi silang antar-dokumen dan terhadap keputusan terkunci (ronde 1–13 + ADR-001..015).
> **Status: DIEKSEKUSI penuh 2026-10-02 + verifikasi ulang 12 agent (4 gelombang) + perbaikan ronde-2 selesai — lihat bagian "STATUS: SEMUA DIEKSEKUSI" di bawah.**
> **Catatan: dokumen ini = arsip historis.** Rentang ADR (`ADR-001..015`) mencerminkan keadaan **saat audit**; ADR-016 ditambahkan ronde 17. Jangan sinkronkan angka di sini — yang berlaku selalu [docs/decisions/](./docs/decisions/).

## Ringkasan Statistik

| Kelompok | File | P0 | P1 | P2 |
|---|---|---|---|---|
| Root (README, AGENTS, RESEARCH, panduan referensi) | 4 | 0 | 6 | 16 |
| Docs produk (00–04 + DOCUMENTATION-MAP) | 6 | 0 | 6 | 24 |
| Architecture | 5 | **1** | 9 | ~15 |
| Database + API | 8 | **2** | 8 | ~14 |
| Features + Testing | 8 | 0 | 7 | 27 |
| Security bag. 1 (security, permissions, threat-model, matrix, asset, boundaries) | 6 | 0 | 5 | 17 |
| Security bag. 2 (auth/signature/order/contract/invariants/keys) | 6 | 0 | 9 | ~19 |
| Security bag. 3 (api/meta/indexer/FE/DB/fraud) | 6 | 0 | 2 | ~13 |
| Security bag. 4 (admin/infra/cicd/IR/CS/requirements/roadmap/gap) | 8 | 0 | 6 | 14 |
| Decisions (ADR-001..015) | 7 | **2** | 6 | 8 |
| Deployment + Tasks | 7 | **1** | 10 | 20 |
| **TOTAL** | **71** | **6** | **~74** | **~187** |

Yang sudah BAIK (tidak perlu dikerjakan): tidak ada file yang merekomendasikan NEAR Lake (deprecated); semua ID SEC-*/INV-*/CS-* yang dikutip terdefinisi (kecuali yang dicatat di bawah); klaim faktual NEAR inti (finality 3 tingkat, NEP-199, 1 yocto, storage 1Ⓝ/100KB, 300 Tgas, tipe akun & skema kunci, NEP-413 Final, Sputnik DAO, State Cleaner) terverifikasi akurat terhadap docs.near.org; tidak ada kontradiksi pada nilai inti (fee 2%, royalti 10%/10 penerima, offer 7 hari/0.01Ⓝ/1-per-buyer, listing 2-tx, auto-stale, Pausable).

---

## P0 — WAJIB DIPERBAIKI (merusak keputusan / menyesatkan implementasi)

### P0-1 — system-architecture.md: alur listing masih model LAMA (1-tx callback)
> "`nft_approve` (NFT contract) → callback `nft_on_approve` (market) → listing tersimpan"
Bertentangan dengan keputusan terkunci: listing = **2 transaksi** (`nft_approve` → `list_nft_for_sale`) + **dual verification** (`nft_token` + `nft_is_approved`). `list_nft_for_sale` tidak disebut sama sekali → agent implementasi akan salah membangun inti marketplace. Perbaiki bagian "Alur data kunci".

### P0-2 — api/authentication.md: mengabaikan NEP-413 (keputusan ADR-011)
Dijelaskan sepenuhnya sebagai skema custom tanpa menyebut NEP-413 (yang statusnya Final & terkunci sebagai UTAMA), dan tidak ada bagian **step-up signature admin**. Tulis ulang: NEP-413 utama (field: message, nonce 32B, recipient=domain, callbackUrl), custom challenge = fallback, + seksi step-up admin. (ADR-011 sendiri juga kontradiksi — lihat P0-5.)

### P0-3 — api/endpoints.md: endpoint profil hilang + launchpad/bundle tanpa endpoint
Kontradiksi langsung dengan api-overview.md ("API menerima data profil") dan authentication.md. Tambahkan: `GET/PATCH /api/accounts/:id/profile`, grup launchpad (phase aktif/status allowlist/progres), `GET /api/bundles/:id`, endpoint admin blocklist, dan parameter `limit` pada contoh cursor.

### P0-4 — decisions/ADR-011: kontradiksi internal Status vs Decision
Status: "NEP-413 dipilih sebagai format utama; custom = fallback". Decision: "Opsi 2 sekarang (custom challenge, DECIDED ronde 11) + evaluasi Opsi 1 (RESEARCH REQUIRED)". Tulis ulang Decision: Opsi 1 (NEP-413) = utama; Opsi 2 = fallback; Opsi 3 tetap ditolak; hapus label RESEARCH REQUIRED di Trade-offs.

### P0-5 — decisions/ADR-013: kontradiksi internal Status vs Decision
Status: "mainnet model DIPUTUSKAN = Sputnik DAO V2 council 2-of-3". Decision: "Mainnet (PROPOSED — gate M4) … RESEARCH REQUIRED tooling multisig". Samakan: "Mainnet DECIDED — implementasi PROPOSED (gate M4)"; anotasi opsi "near multisig" = DEPRECATED; perkecil daftar RESEARCH REQUIRED (sisa: integrasi Owner pattern ↔ DAO).

### P0-6 — tasks/implementation-plan.md: offers salah fase + jalur M1 pincang
"Fase 3+ — sesuai M2/M3 (offers → auctions → indexer)" menempatkan **offers di luar M1**, padahal M1 done-when = "offer→accept end-to-end via UI" dan TASK-009 = MVP. TASK-026 (SEC P0) tidak dijadwalkan di fase mana pun; launchpad (020/021) + notifikasi (015) tidak ada slot sebelum demo M1; TASK-008b bukan prasyarat Fase 2 padahal branding wajib sebelum UI. Perbaiki urutan fase + label "Fase ≠ Milestone".

---

## P1 — SEBAIKNYA DIPERBAIKI (kontradiksi / gap cakupan)

### Tema A — Kontradiksi dengan keputusan terkunci (stale)
1. **README**: klaim "fee 2% di bawah OpenSea 2,5%" **usang** — OpenSea kini ~1% (OS2, 2025); fee kita justru lebih tinggi → ganti value-prop (tekankan royalti on-chain, dst.) atau beri tanggal+sumber. Juga: "auctions" tanpa penanda fase.
2. **RESEARCH.md**: ada **dua seksi "## 12"** (Link Cepat & Security Index) → renumber; roadmap 11.3 masih draf lama (offers/bundle di fase salah) → beri catatan "draf pra-keputusan, final = implementation-plan"; "Decisions open: G2" sudah terjawab; angka "22 dokumen keamanan" (faktanya 26 file di security/); jejak editing tertinggal ("koreksi atribusi, sebelumnya keliru", "ditambahkan setelah verifikasi ulang") → rapikan jadi pernyataan final; "diulurkan" typo; "subscribe event via NearBlocks" → polling.
3. **01-PRD**: §17 Future Scope memuat **bundle** padahal fitur MVP → hapus; blok contoh §8 duplikat dengan "price > 0" (bertentangan dengan 0.01 Ⓝ) → hapus/sinkronkan; §4 "menunggu keputusan data layer (ronde 2)" stale; §13 phase tanpa atribut **waktu**.
4. **02-product-requirements**: **5 blok duplikat** (LIST/CANCEL/OFFER/SEARCH/VIEW PROFILE muncul 2× dengan variasi) → hapus satu set; log ronde 13 di DOCUMENTATION-MAP yang mengklaim "hapus duplikat" juga harus dikoreksi; header "final via tanya-jawab" stale; "media IPFS" vs keputusan IPFS ditunda.
5. **tech-stack**: "Runtime/Database/Cache — Menunggu fase 2" **kontradiksi** dengan baris "DB report: PostgreSQL di VPS" (sudah diputuskan ronde 5); kategori **auth (NEP-413) hilang**; **Pausable & MAX_FEE_BPS tidak disebut**; indexer fase 2 tanpa "**Neardata** (Lake deprecated — dilarang)".
6. **environments**: mainnet Kepemilikan "⏳ keputusan rilis" → isi Sputnik DAO V2 2-of-3 + timelock; variabel env kurang (TELEGRAM_BOT_TOKEN, DATABASE_URL, RPC failover list, secret CI); tandai IPFS vars = fase lanjut.
7. **disaster-recovery**: "mainnet M4: evaluasi multisig" → sudah DECIDED Sputnik DAO; baris RPC failover tanpa chain konkret (FASTNEAR → official → dRPC); "DECIDED (draft)" oxymoron; "MVP: seed backup akun baru + redeploy" → perjelas state tidak ikut pindah (opsi testnet saja).
8. **catastrophic-failure-scenarios**: CS-9 "**PROPOSED** ADR-013" stale → DIPUTUSKAN; CS-6 mencampur NEP-330 (verifikasi kontrak) dengan integritas bundle FE (SRI/CI) → pisahkan; alias file "smart-contract-arch §2b" salah nama.
9. **fraud-and-abuse**: "self-buy DITOLAK kontrak (FACT, ronde 9)" **tidak didukung** — tidak ada invariant-nya dan rujukan ronde salah → pilih: tambah invariant self-buy ATAU ganti dengan deterrent ekonomi + heuristik fase 2.
10. **frontend-security**: **CSP `connect-src https://rpc*` — sintaks wildcard hostname TIDAK VALID** di CSP → pakai host eksplisit; `data:` di img-src vs kebijakan data-URL yang masih OPEN QUESTION → selaraskan.

### Tema B — Fitur MVP tanpa representasi (gap cakupan terbesar)
11. **Private listing, bundle, launchpad tidak ada di database**: database-schema tak punya tabel `launchpad_phases`, `bundles`/`bundle_items`, `blocklist`, `admin_audit`, `auth_nonce` (TTL), kolom private listing (`allowed_buyer`), unique partial index offer, dan `events_raw` pakai tx_hash bukan **(receipt_id, event_index)**.
12. **data-model**: tanpa entitas Bundle & LaunchpadPhase; "Listing resolves to Sales (1..0..1)" salah notasi; sumber penjualan seharusnya event `market_sale`, bukan `nft_transfer`.
13. **endpoints/API overview**: grup launchpad/bundles/profiles hilang (lihat P0-3); status "final" vs "draft" saling bertentangan.
14. **Profil custom tidak punya spec fitur** di docs/features (hanya 2 baris di acceptance-criteria) → tambahkan seksi di users.md (flow edit, error, AC).
15. **03-user-flows**: flow hilang untuk bundle, private listing, cancel/update price, notifikasi, edit profil, onboarding; buy flow tertulis "Sign **offer**" (copy-paste); arah settlement terbalik; WalletConnect disebut sebagai wallet; urutan upload allowlist beda dengan 02.
16. **Threat-model**: autentikasi tidak menyebut NEP-413 sekali pun; **private listing & bundle tanpa baris ancaman** (bundle partial-failure = risiko nyata); Pausable tidak dirujuk sebagai mitigasi.
17. **Invariants**: tidak ada invariant **bundle all-or-nothing**, **maks 1 offer/buyer/token**, **min 0.01Ⓝ**, **private listing hanya buyer yang ditunjuk**, **cap royalti 10%**; INV-004 "constant immutable selamanya" overstated (NEAR const bisa berubah via upgrade — reformulasi dengan governance ADR-013 + NEP-330); INV-018 "≥ exact" kontradiktif; INV-021 placeholder nilai.
18. **Testing**: bundle fail-midway (revert+refund) tidak ada di skenario wajib/TC/AC; **SEC-AUTH-006 (rotasi refresh token) dirujuk 2 dokumen tapi tidak terdefinisi di register**; fuzz tooling & rujukan INV tidak masuk testing-strategy; TC-003 (mock payout) & TC-004 (time warp) tidak realistis; duplikasi seksi "Definisi selesai" di testing-strategy **masih ada**.

### Tema C — Desain kontrak belum tertutup (perlu keputusan kecil sebelum koding)
19. **Pause single-key vs owner = DAO account** (key-management §3): jika owner_id = DAO, panggilan pause single-key gagal cek owner → butuh desain guardian/daftar caller pause eksplisit.
20. **`nft_revoke` vs `nft_revoke_token`**: market sebagai approved account harus memanggil `nft_revoke_token`, bukan `nft_revoke` (owner-only) — salah di order-protocol & contract-arch.
21. **Kanonisasi nama method**: `offer`/`make_offer`/`buy` dipakai campur lintas file; `process_listing` vs `nft_on_approve`; `remove_stale` vs `remove_stale_listing` → tetapkan satu nama resmi.
22. **Bundle edge cases**: mekanisme rollback multi-token (gagal di token ke-k), token bundle yang dipindah/di-list terpisah, apakah token dalam bundle boleh di-offer; exception Pausable untuk cancel bundle.
23. **Ekonomi kecil belum ditutup**: deposit > price (refund selisih?), storage deposit saat cancel offer (dikembalikan via storage_withdraw?), cap royalti 10% tidak disebut di spec settlement.

### Tema D — Klaim tidak akurat (koreksi faktual)
24. **asset-inventory**: "refund otomatis saat gagal" menyesatkan — protokol hanya auto-refund deposit promise yang gagal; refund buyer = via resolve_purchase (SEC-ORDER-001/002); function-call key = tak bisa attach deposit **apa pun** (bukan hanya 1 yocto).
25. **trust-boundaries §7**: klaim `#[private]` tidak berlaku untuk arah NFT→market (`nft_on_approve` = call masuk dari kontrak mana pun) → pisahkan dua mekanisme per arah; penomoran duplikat "## 7".
26. **metadata-security**: `external_url` BUKAN field standar NEP-177 (tandai ekstensi); `media_hash` vs `reference_hash` digabung salah; gateway `cloudflare-ipfs.com` usang; klaim SVG-in-img "resource eksternal bisa fetch" perlu kualifikasi (browser umumnya blokir di secure static mode); typo "dekomresi".
27. **indexer-security**: `mainnet.fastnear.com` bukan endpoint publik resmi (yang resmi: `free.rpc.fastnear.com`); event identity 3-komponen vs 2-komponen (definisikan index-in-receipt vs event_index).
28. **incident-response & CS-1**: "gunakan seed backup di akun baru" berbahaya dibaca harfiah — akun baru WAJIB kunci fresh; **klaim G6 "mass-revoke ditambahkan ke playbook IR" tidak terbukti** — playbook belum memuatnya.
29. **security-requirements & gap-analysis**: SEC-CONTRACT-012 vs SEC-KEY-002 duplikasi isi; compound status "DECIDED (implementasi PROPOSED)"; "Prinsip prompt"/"Laporan prompt" (bahasa meta bocor, 3×); ringkasan gap melewatkan G12/G13; G15 duplikat; klaim roadmap vs isi ADR-013 kontradiktif.

### Tema E — Sistemik tasks/plan/monitoring
30. **Tidak ada task infra/ops** di backlog: provisioning VPS+Docker, CI/CD pipeline, domain/SSL, backup harian, alert Telegram (semua terkunci di docs tapi tak ada tasknya).
31. **TASK-020 launchpad tidak menyebut Pausable + MAX_FEE_BPS**; TASK-002 tanpa NEP-178/297; TASK-014 tanpa Neardata; kolom Depends/Spec didefinisikan tapi tak dipakai; enum area dilanggar; footer "penomoran final via tanya-jawab" stale.
32. **milestones**: M4 "Audit checklist" → harusnya "**Audit eksternal lulus (TASK-027)**"; M4 done-when tanpa transfer ownership ke Sputnik DAO + reproducible build; M3/M4 tanpa done-when; split indexer M2/M3 tumpang tindih; TASK-014 P2 vs deliverable M2.
33. **monitoring**: tidak ada alert untuk event kritis kontrak (pause/unpause, payout gagal, tx owner/DAO) — dasar deteksi drill SEC-IR-001; cert-expiry & disk check minimal tidak ada.
34. **Higienis lintas dokumen**: artefak "(ronde N)" bocor di 5+ file; "ronde" tanpa definisi di AGENTS; tanggal "2026-10-01" konsisten maju dari tanggal sistem (verifikasi jam); judul ADR-001 masih placeholder template; ADR-002..009 tanpa file; ADR-015 Neardata tercatat di Consequences bukan Decision/Status; `near-connect-hooks` perlu verifikasi nama paket npm; "SEC-AUTH-006" menggantung (lihat #18); referensi path relatif yang tidak resolve dari lokasi file (RESEARCH.md, 02-requirements, disaster-recovery dari folder security).

---

## P2 — POLISH (ringkas per file, untuk dikerjakan setelah P0/P1)

- **README**: seragamkan whitelist→allowlist; tandai `yarn` sebagai contoh; jelaskan istilah "ronde".
- **AGENTS**: perjelas rumusan SEC-P0 dalam DoD.
- **RESEARCH**: perjelas 10% (persen) vs 10 penerima; path lengkap rujukan security; istilah status seragam; "nft_transfer + payout" → market_sale (sama dengan data-model).
- **panduan referensi (arsitektur-dokumentasi…)**: file referensi — tandai struktur mana yang diikuti; heading H1→H2; "Next.js 16" hard-coded.
- **00-overview**: perjelas "10% vs 10 penerima".
- **04-ux**: rinci dropdown Create "…"; tambah komponen notifikasi/report/bundle-modal/private-listing; label "final ronde 8" stale.
- **DOCUMENTATION-MAP**: inventaris decisions (ADR-001..015); threat-model dobel di daftar; entri tanpa .md; link ADR-010..015 → folder decisions; klaim ronde 13 "hapus duplikat" dikoreksi.
- **system-architecture**: sejajarkan ASCII; pisahkan label panah; royalti cap di tabel; kurator verified badge → pointer backend; IPFS "TODO" → "ditunda".
- **frontend-architecture**: `near-connect-hooks` verifikasi; daftar wallet kutip sumber; TASK-008b reference; profil auth line; seragamkan label ronde.
- **backend-architecture**: intro indexer vs MVP; "TIDAK jalan jalur settlement" redaksi; notifikasi = client-side; path link; duplikasi daftar VPS dengan infrastructure.
- **infrastructure**: "Caddy allowlist" perjelas; origin-IP reformulasi; owner-key risk label; backup pg_dump + Telegram pointer; verifikasi domain `testnet-rpc.intea.rs`.
- **seed-data**: satu metode mint; sandbox vs testnet; seed phase + profil; path RESEARCH.md.
- **migrations**: Prisma final (hapus alternatif); "idempoten" → sekali-jalan terlacak; cakupan rebuild (app-mutable tidak ikut).
- **api-overview**: "API = service off-chain (indexer)" → proyeksi + layanan aplikasi; NEP-413 di auth ringkas.
- **endpoints**: field `allowed_buyer`/bundle di respons; semantik PATCH admin report.
- **webhooks**: pindahkan komentar keluar blok JSON; label pengaman HMAC "pasca-v1".
- **features/auth**: AC + saldo; urutan timeout reset; pointer profil.
- **features/users**: `creator_id` = ekstensi custom + fallback; AC diangkat ke acceptance-criteria; "indexer" → NearBlocks.
- **features/marketplace**: min harga 0.01 di spec; deposit > price; royalti cap; path relatif.
- **features/payments**: treasury TODO → task; hapus "toleransi bps"; error royalti >10%.
- **features/notifications**: hapus label "draft"; mekanisme deteksi per tipe notifikasi; AC per tipe.
- **testing-strategy**: tautkan skenario ↔ INV; notifikasi & profil dalam skenario.
- **test-cases**: lengkapi field Precondition/Layer; realistiskan TC-003/TC-004.
- **security.md**: "dipreview ulang" → "direview ulang"; pointer mainnet ADR-013; ronde jargon; path RESEARCH.md.
- **permissions**: konvensi sel matriks; alias role ↔ register; baris pause/withdraw/upgrade.
- **access-control-matrix**: fase baris withdraw-treasury; SECURITY/FINANCE sebagai aktor; AuthN mainnet = DAO proposal.
- **threat-model**: nonce row paralel; definisi inline G1/G4.
- **asset-inventory**: baris multisig sesuai keputusan final; sinkron withdraw-treasury.
- **api-security**: rate limit nonce +30/akun/jam; admin 30/menit beri dimensi; avatar gateway; NEP-413 di baris verify.
- **indexer-security**: definisi event_index vs index-in-receipt.
- **frontend-security**: placeholder `<gateway-allowlist>` diberi tanda; JWT in-memory vs denylist penjelasan.
- **database-security**: perjelas fase tabel cache; path DR link.
- **fraud-and-abuse**: rapikan baris sybil; link drainer → gap-analysis.
- **admin-security**: ronde jargon; sinkron IR.
- **infrastructure-security**: accepted-risk owner key eksplisit.
- **cicd-security**: TASK-001 definisi; "set force" redaksi.
- **ADR-001**: isi judul; judul tabel "Indeks ADR"; relasi ADR-002 ← ADR-007.
- **ADR-015**: Neardata masuk bagian Decision/Status.

---

## Fakta eksternal penting (hasil verifikasi agent)

1. **Klaim fee OpenSea di README sudah tidak valid** — OpenSea kini memungut ~1% (bukan 2,5%). Value proposition "fee kita lebih murah" harus dirombak: yang tetap kuat = royalti dipaksa on-chain, gas murah, finality cepat, launchpad.
2. **NEAR Lake resmi deprecated 24 Maret 2026** — sudah dikoreksi ke Neardata di semua dokumen.
3. Semua klaim NEAR lain yang dicek agent (finality, NEP-413 Final, ml-dsa-65, Sputnik DAO, State Cleaner, NEP-330, RPC providers, account types) — akurat.

## Urutan perbaikan yang disarankan

1. **6 P0** (system-architecture, api/authentication, api/endpoints, ADR-011, ADR-013, implementation-plan) — ini yang akan menyesatkan implementasi.
2. **Tema B & C** (gap cakupan launchpad/bundle/private-listing/profil + desain kontrak yang belum tertutup) — lengkapi sebelum Fase 1.
3. **Tema A & D** (stale & koreksi faktual) — bersih-bersih satu pass.
4. **Tema E + P2** (sistemik tasks + polish) — satu pass terakhir.
5. Setelah selesai → jalankan checklist DOCUMENTATION-MAP + update riwayat sinkronisasi.

---

## STATUS: SEMUA DIEKSEKUSI (2026-10-02)

Semua P0/P1/P2 di atas sudah diperbaiki (~45 file). Riwayat: DOCUMENTATION-MAP.md §5.

### Keputusan pasca-audit — diterapkan sebagai DEFAULT (bisa di-override user)

| # | Keputusan | Default | Status |
|---|---|---|---|
| Q1 | G7 kompensasi user | **Insurance fund 0.1% dari fee 2%**; aktif mainnet; pencairan hanya via proposal Sputnik DAO; tidak dijamin otomatis | Diterapkan (incident-response, gap-analysis) |
| Q2 | Launchpad phase overlap | **Berurutan wajib, tidak overlap** (1 fase aktif pada satu waktu — kontrak sederhana) | Diterapkan (01-PRD §13) |
| Q3 | Cap royalti bundle | **Per token masing-masing ≤10%**, dijumlahkan tanpa cap agregat (standar industri) | Diterapkan (INV-027) |
| Q4 | Max token per bundle | **10 token** (aman dalam gas 300 Tgas) | Diterapkan (INV-021) |

### Masih terbuka (user bisa jawab kapan saja; default sudah aman)

1. **Treasury withdraw MVP**: default = fungsi `withdraw_fees` owner-only tersedia sejak testnet (uji di testnet), mainnet via DAO. Alternatif: terkunci sampai DAO.
2. **Over-deposit mint**: default = **reject** (INV-018, deposit harus exact). Alternatif: terima + refund selisih.
3. **Package manager frontend**: default usulan **pnpm**; final ditetapkan saat TASK-001.
4. **G8 auditor**: kapan mulai kontak kandidat (OtterSec/Halborn/Block Security)? Default: defer sampai mendekati M4.

### Catatan klarifikasi

- Temuan "tanggal 2026-10-01 di masa depan" dari subagent = **false positive** (tanggal sistem resmi sesi ini memang awal Oktober 2026) — tidak perlu tindakan.
- Referensi "(ronde N)" dipertahankan di seluruh dokumen + glosarium ditambahkan di AGENTS.md (keputusan, bukan kelalaian).
