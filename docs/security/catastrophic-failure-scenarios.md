# Catastrophic Failure Scenarios

> Skenario bencana paling berbahaya + jalur serangan + pencegahan berlapis.
> **Format wajib per skenario (9 field)**: Attack Path → Preconditions → Assets → Impact → Detection → Prevention → Mitigation → Emergency → Recovery.
> Setiap skenario punya **Likelihood** (Rendah/Sedang/Tinggi) dan **Impact** (Rendah/Sedang/Kritis) agar prioritas jelas.
> Terkait: [threat-model.md](./threat-model.md), [incident-response.md](./incident-response.md), [key-management.md](./key-management.md).

---

## CS-1 — Penyerang memperoleh otoritas owner (full-access key)

- **Attack path**: VPS compromise (RCE via app bug → membaca `~/.near-keys`) / seed phrase bocor (phishing tim) / CI secret leak jika key pernah lewat pipeline.
- **Preconditions**: satu saja dari jalur di atas berhasil.
- **Assets**: semua kemampuan owner — pause, `fee_bps` (dibatasi cap), `update_treasury` (arahkan fee), **upgrade kontrak ke kode berbahaya**. (Tidak ada penarikan treasury dari kontrak — fee masuk akun treasury saat settlement, ronde 23.)
- **Impact**: **Kritis**. Worst case = deploy kontrak jahat + drain escrow offer + arahkan payout. NFT user TETAP AMAN (tidak pernah dipegang kontrak — approval model ADR-007). Escrow offer bisa dirampas via upgrade.
- **Detection**: monitor tx dari owner account (polling NearBlocks → alert Telegram); tx upgrade tak terjadwal; perubahan `treasury` tak terduga.
- **Prevention**: key tidak pernah di app process; hardening VPS; tidak pernah lewat CI; seed offline 2 lokasi ([key-management.md](./key-management.md)).
- **Mitigation**: escrow kecil (offer expire 7 hari + user bisa cancel sendiri); `MAX_FEE_BPS` immutable membatasi kerusakan via fee; **mainnet**: Sputnik DAO V2 council 2-of-3 + timelock (ADR-013) → upgrade tidak instan.
- **Emergency**: pause race → bila key sudah di tangan attacker, buat akun/deploy baru dengan **kunci FRESH yang tidak pernah terekspos** (BUKAN seed backup yang mungkin ikut bocor). Attacker bisa menyalahgunakan approval yang sudah ada → jalankan playbook mass-revoke: user `nft_revoke_all` per token + market `remove_stale_listing` (G6).
- **Recovery**: kontrak baru + re-list (2 tx per item — mahal tapi mungkin); escrow refund otomatis via `cancel_offer` user-side.
- **Likelihood**: Rendah (MVP testnet: tanpa dana nyata). **Drill**: SEC-IR-001.

## CS-2 — Penyerang bypass validasi order (execute cancelled/expired/stale)

- **Attack path**: bug pada lifecycle check → beli listing yang sudah cancel/stale, atau accept offer yang sudah expired.
- **Preconditions**: bug lolos dari test invariant.
- **Assets**: konsistensi dana/NFT (bukan dana kontrak); dampak terbatas per-tx (state masih on-chain).
- **Impact**: Sedang — satu tx salah settle; bisa dibalikkan lewat kompensasi, bukan kehilangan sistemik.
- **Detection**: test suite invariant (INV-008/009/016) + monitoring anomali (`SOLD` ganda, settle setelah cancel).
- **Prevention**: invariant + property test; **satu fungsi settle bersama** untuk semua jalur (buy/accept_offer/bundle) sehingga cek lifecycle tidak terduplikasi.
- **Mitigation**: optimistic removal + revert (state dipulihkan); dual verification saat settle (SEC-ORDER-004).
- **Emergency**: pause; analisis; patch; redeploy.
- **Recovery**: refund manual/off-chain kompensasi — kebijakan **G7** (DEFAULT: insurance fund 0.1% dari fee, pencairan via proposal DAO).

## CS-3 — Manipulasi akuntansi settlement (payout)

- **Attack path**: bug di payout split — overflow/underflow, receiver duplikat, fee dihitung dua kali/kurang, pembulatan tak konsisten.
- **Preconditions**: bug aritmetika lolos dari unit test & fuzz.
- **Assets**: dana hasil penjualan (proceeds seller, royalti, fee treasury).
- **Impact**: **Kritis** — dana salah distribusi; transfer on-chain **final** (FACT) → tidak bisa dibalikkan; korban = seller/royalti/trader.
- **Detection**: reconcile `sum(payout) == harga − fee` tiap settle; unit test matriks royalti (0%, 1000 bps, bulat tak rata); alert bila selisih > 1 yocto.
- **Prevention**: INV-001..004; **checked arithmetic** (`overflow-checks` di release); batas 10 penerima; validasi Σpayout ≤ harga − fee dan amount > 0.
- **Mitigation**: payout divalidasi **sebelum** transfer (bukan sesudah); gagal validasi → revert + refund penuh.
- **Emergency**: pause sampai patch; identifikasi tx terdampak dari event `market_sale`.
- **Recovery**: kompensasi via G7 insurance fund untuk kasus terverifikasi; perbaikan + test regresi sebelum unpause.

## CS-4 — Drain escrow offer

- **Attack path**: bug refund/accept → escrow dikirim ke alamat salah, atau `accept_offer` dobel pada offer yang sama.
- **Preconditions**: bug pada jalur refund/accept.
- **Assets**: escrow Ⓝ dari offer aktif (dana user yang ditahan sementara).
- **Impact**: **Kritis** — dana buyer hilang/terkirim ke pihak salah.
- **Detection**: reconcile saldo escrow kontrak vs `sum(offers.amount)`; monitoring accept ganda.
- **Prevention**: INV-005/006/009; refund tujuan **hardcoded `offer.buyer_id`** (tidak bisa diarahkan attacker); entry offer dihapus atomik sebelum settle.
- **Mitigation**: escrow hanya untuk offer aktif dan selalu user-recoverable (cancel/expire) — kerusakan dibatasi per-offer.
- **Emergency**: pause; audit jalur refund; patch.
- **Recovery**: refund manual + kompensasi G7; test regresi `accept_offer`/refund sebelum unpause.

## CS-5 — Metadata SSRF / FE script injection

- **Attack path**: (MVP) metadata jahat berisi SVG/HTML ber-script → XSS di browser korban. (Fase 2) URL metadata menunjuk alamat internal → server fetcher dipakai menembus jaringan internal (SSRF).
- **Preconditions**: MVP — FE merender metadata tanpa sanitasi; Fase 2 — fetcher tanpa guard SSRF.
- **Assets**: sesi browser user (MVP); jaringan internal + kredensial server (Fase 2).
- **Impact**: Sedang (MVP: XSS sesi user, bukan dana langsung); Tinggi (Fase 2: SSRF bisa mencapai layanan internal).
- **Detection**: CSP report-only logs; anomali request dari fetcher; audit URL yang di-fetch.
- **Prevention**: MVP — **server tidak fetch apa pun** (FACT, ADR-014) → permukaan SSRF kosong; render media **hanya `<img>`**, dilarang `innerHTML`/`<svg>` inline; CSP strict. Fase 2 — fetcher terisolasi dengan guard SSRF ([metadata-security.md](./metadata-security.md) §3).
- **Mitigation**: gateway allowlist; verifikasi hash media (`media_hash`) client-side; interstitial untuk link eksternal.
- **Emergency**: matikan fitur render thumbnail/fetcher; rotasi kredensial bila SSRF terbukti.
- **Recovery**: tambal guard, uji ulang dengan test SSRF (private/loopback/metadata-endpoint), baru aktifkan kembali.

## CS-6 — Frontend supply chain compromise

- **Attack path**: npm dependency malicious / akun GitHub takeover → inject kode yang menukar receiver pada transaksi.
- **Preconditions**: dependency jahat lolos audit, atau akun maintainer diambil alih.
- **Assets**: transaksi user (receiver bisa ditukar), sesi user.
- **Impact**: **Kritis** — user menandatangani tx jahat dari UI yang tampak asli; wallet menampilkan receiver berbeda.
- **Detection**: CSP report logs; anomali Dependabot; review manual diff; laporan user.
- **Prevention**: tanpa third-party script; CSP strict; lockfile + `npm audit`; review wajib untuk perubahan workflow; **verifikasi kontrak yang dipanggil FE** = reproducible build NEP-330.
- **Mitigation**: **SRI** untuk aset eksternal + integritas bundle FE sendiri (lockfile + CI) — dua hal berbeda, jangan dicampur.
- **Emergency**: rollback FE ke build terakhir yang terverifikasi; cabut token deploy; umumkan.
- **Recovery**: audit dependency, ganti yang terkompromi, rebuild + re-verifikasi hash, baru deploy.

## CS-7 — CI/CD compromise → deploy kontrak jahat

- **Attack path**: edit workflow tanpa review / secret CI bocor → pipeline mendeploy artifact jahat.
- **Preconditions**: akses tulis ke workflow, atau secret deploy bocor.
- **Assets**: artifact deploy, kontrak produksi (bila gate mainnet dilewati).
- **Impact**: **Kritis** (mainnet) — kontrak jahat ter-deploy; Sedang (testnet) — tanpa dana nyata.
- **Detection**: audit log GitHub (perubahan workflow, run tak wajar); verifikasi hash pasca-deploy.
- **Prevention**: branch protection (**perubahan file workflow = review wajib**); deploy mainnet **manual-trigger** + environment approval; `pull_request_target` dilarang dengan checkout PR.
- **Mitigation**: **post-deploy hash verification** (SEC-CONTRACT-006) — deploy gagal bila hash ≠ artifact; mainnet timelock (ADR-013).
- **Emergency**: batalkan deploy, revoke deploy key + rotasi secret CI, rollback ke versi terverifikasi.
- **Recovery**: audit provenance, perbaiki pipeline, re-deploy hanya setelah hash cocok.

## CS-8 — DNS hijack / phishing mirror

- **Attack path**: akun registrar diambil alih → domain diarahkan ke FE jahat (wallet-drainer UI).
- **Preconditions**: akses registrar/registrar account compromise.
- **Assets**: reputasi domain, user yang menandatangani di mirror.
- **Impact**: Sedang — bukan dana langsung (kontrak tidak tersentuh); user yang sign di mirror menanggung risiko sendiri; reputasi rusak.
- **Detection**: monitoring DNS = cron mingguan `dig` banding record → alert Telegram (G9); laporan komunitas.
- **Prevention**: registrar **2FA + transfer-lock**; catat registrar + kontak darurat; edukasi user (domain resmi + bookmark).
- **Mitigation**: publikasikan domain resmi di kanal; DMARC/SPF untuk email; sertifikat CT monitoring.
- **Emergency**: hubungi registrar segera (lock/restore), ganti DNS, umumkan peringatan.
- **Recovery**: pulihkan kontrol domain, rotasi kredensial registrar, verifikasi DNS konsisten, postmortem.

## CS-9 — Upgrade ke implementasi jahat (owner)

- **Attack path**: sama seperti CS-1, tapi "sah" secara transaksi (owner memanggil `upgrade`) — tidak ada tx mencurigakan dari sisi format.
- **Preconditions**: akses owner key (atau DAO council diretas).
- **Assets**: seluruh logika kontrak (termasuk jalur dana).
- **Impact**: **Kritis** — logika bisa diubah untuk mengalihkan dana/escrow.
- **Detection**: pemantauan hash implementasi kontrak (bandingkan dengan rilis resmi); alert pada perubahan code hash tak terjadwal.
- **Prevention**: **mainnet: Sputnik DAO V2 2-of-3 + timelock** (upgrade tidak instan, ada jeda review); publish + review wasm hash sebelum approve proposal.
- **Mitigation**: NEP-330 reproducible build → hash terverifikasi; komunitas bisa memverifikasi build.
- **Emergency**: pause via guardian; batalkan proposal upgrade; bila sudah ter-upgrade → CS-1 recovery path.
- **Recovery**: deploy ulang implementasi yang terverifikasi; dokumentasikan + perbarui prosedur upgrade.
- **Catatan MVP**: diterima di testnet (tanpa dana nyata) — ADR-013.

## CS-10 — Poisoning indexer (fase 2)

- **Attack path**: event palsu dari kontrak lain, atau bug ingest → data pencarian/statistik salah.
- **Preconditions**: indexer fase 2 aktif; filter emitter/validasi lemah.
- **Assets**: data proyeksi (discovery, floor, volume) — **bukan** dana/ownership.
- **Impact**: Sedang — tampilan salah (harga/statistik menyesatkan); tidak bisa memindahkan dana (SEC-INDEX-001).
- **Detection**: reconcile jumlah event vs chain; anomali lonjakan volume; cek hash blok final.
- **Prevention**: **filter emitter = kontrak kita saja**; ingest **hanya blok final**; dedup `(receipt_id, event_index)`; skema `events_raw`.
- **Mitigation**: indexer tidak pernah menjadi otoritas (FE re-verify on-chain — SEC-ORDER-003); rebuild dari `events_raw` bila korup.
- **Emergency**: hentikan ingest, tandai data "tidak terpercaya", fallback FE ke RPC langsung.
- **Recovery**: rebuild indexer dari checkpoint, validasi rekonsiliasi, baru aktifkan kembali.

---

## Ringkasan prioritas

| Skenario | Likelihood (MVP) | Impact | Prioritas |
|---|---|---|---|
| CS-1 owner key compromise | Rendah | Kritis | Tertinggi (gate mainnet) |
| CS-3 akuntansi payout | Rendah | Kritis | Tinggi (test INV-001..004 + fuzz) |
| CS-4 drain escrow | Rendah | Kritis | Tinggi (test INV-005/009) |
| CS-6 supply chain FE | Sedang | Kritis | Tinggi |
| CS-7 CI/CD | Rendah | Kritis | Tinggi |
| CS-9 upgrade jahat | Rendah | Kritis | Tinggi (mainnet DAO) |
| CS-2 bypass lifecycle | Sedang | Sedang | Sedang (property test) |
| CS-5 metadata SSRF/XSS | Sedang | Sedang | Sedang |
| CS-8 DNS hijack | Sedang | Sedang | Sedang |
| CS-10 indexer poisoning | — (fase 2) | Sedang | Fase 2 |

**Prinsip lintas-skenario**: NFT user tidak pernah dipegang kontrak (ADR-007) → bahkan worst case CS-1 tidak menghilangkan kepemilikan NFT. Setiap skenario punya jalur **deteksi → pause (jika perlu) → patch → verifikasi → unpause**.

## Peta drill per skenario (validasi kesiapan)

> Setiap skenario punya drill/test yang membuktikan kemampuan respons — bukan sekadar dokumen. Drill operasional dijadwalkan di [incident-response.md](./incident-response.md) §12; test kontrak di [testing/test-cases.md](../testing/test-cases.md).

| Skenario | Drill / test yang memvalidasi | Jenis | Fase | Owner |
|---|---|---|---|---|
| CS-1 owner key compromise | Drill pause/unpause testnet (**SEC-IR-001**) + uji skrip mass-revoke (G6: `nft_revoke_all` + `remove_stale_listing`) + drill rotasi council (key-management §3a) | drill | Phase D / sebelum M4 | SECURITY |
| CS-2 bypass validasi order | Property/invariant test INV-008/009/016 + TC-006 (stale) + TC-022 (harga berubah) | test | Phase D | contract |
| CS-3 manipulasi akuntansi payout | Fuzz + unit matriks royalti INV-001..004 + TC-003/TC-005/TC-015 | test | Phase D | contract |
| CS-4 drain escrow offer | Sandbox test INV-005/009 + TC-004/TC-005/TC-018 (double-accept) | test | Phase D | contract |
| CS-5 metadata SSRF / FE XSS | DAST: header/CSP scan + uji SSRF (private/loopback/metadata-endpoint) — SEC-META-001/002 | test (DAST) | Phase D (MVP) / fase 2 (SSRF) | frontend/backend |
| CS-6 frontend supply chain | Drill rollback FE ke build terverifikasi + verifikasi hash NEP-330 | drill | Phase D | infra/frontend |
| CS-7 CI/CD compromise → deploy jahat | Drill simulasi kompromi pipeline: revoke secret + rebuild dari commit bersih + verifikasi hash (SEC-CICD-002, SEC-CONTRACT-006) | drill | Phase D / sebelum M4 | infra |
| CS-8 DNS hijack / mirror | Drill pemulihan DNS + cron `dig` banding record (G9) → alert; uji transfer-lock registrar | drill | Phase D | infra |
| CS-9 upgrade ke implementasi jahat | Drill proposal DAO 2-of-3 + timelock + publish/verifikasi wasm hash (SEC-CONTRACT-012) | drill | sebelum M4 | SECURITY |
| CS-10 poisoning indexer (fase 2) | Drill rebuild indexer dari `events_raw` + rekonsiliasi jumlah event vs chain (SEC-INDEX-002) | drill | fase 2 (M2+) | indexer |

- **Aturan**: skenario tanpa drill yang lulus = **blocker mainnet** (khususnya CS-1, CS-7, CS-9).
- Hasil drill dicatat di [incident-response.md](./incident-response.md) + memperbarui status SEC-* terkait.
- Peta ini melengkapi (bukan mengganti) format 9-field dan tabel prioritas di atas.
