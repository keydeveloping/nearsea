# Key Management

> Strategi otoritas signing produksi. Prinsip prompt: **"private key di environment variable" BUKAN model produksi** — dipakai hanya sebagai model interim MVP dengan risiko eksplisit + kontrol kompensasi, dan WAJIB digantikan sebelum mainnet.

## 1. Kunci yang ada

| Kunci | Kegunaan | Kritisitas |
|---|---|---|
| Owner key market/factory (full-access) | pause, fee, treasury, upgrade, withdraw | **Tertinggi** — setara kendali seluruh dana kontrak |
| Deploy key (bisa = owner key) | `cargo near deploy` | Tinggi (identik owner di MVP) |
| SSH deploy key (GitHub → VPS) | deploy aplikasi | Sedang — akses server, bukan dana |
| Session signing (JWT secret) | API report | Sedang — spoof identitas off-chain |

## 2. MVP (testnet) — interim model: DECIDED dengan risiko eksplisit

```text
GENERATION : keypair dibuat via near-cli di mesin lokal (TIDAK via layanan online)
STORAGE    : full-access key di VPS `~/.near-keys/` chmod 600, disimpan terenkripsi di disk (LUKS/fscrypt — RESEARCH REQUIRED apakah layanan VPS mendukung)
             seed phrase dicetak offline (kertas/steel), disimpan terpisah dari VPS
ACCESS     : hanya 1 orang (PLATFORM_OWNER) tahu lokasi; VPS SSH key-based saja (password auth off)
ROTATION   : tidak diotomasi; rotasi manual = buat akun baru + migrasi owner via kontrak (jika ada fungsi) ATAU deploy ulang
BACKUP     : seed offline 2 salinan lokasi berbeda; VPS key TIDAK dibackup ke cloud
```

**Risiko diterima (documented)**: VPS compromise = owner key compromise = kendali penuh kontrak.
**Kompensasi**: (a) dana escrow tetap refund-able oleh user via cancel_offer (bukan hanya owner), (b) `MAX_FEE_BPS` immutable membatasi kerusakan finansial via fee, (c) testnet = tanpa dana nyata, (d) alert Telegram untuk tx owner tak terduga — via polling NearBlocks API (diputuskan; TASK-026).

## 3. Mainnet — target model: DECIDED (riset 2026-10-01) — implementasi PROPOSED

| Opsi | Kelebihan | Kekurangan | Keputusan |
|---|---|---|---|
| **Sputnik DAO V2 council** (multisig kontrak) | Standar multisig aktif & maintained di NEAR (near-daos/sputnik-dao-contract); bisa deploy via global contract (docs primitives/dao); UI **NEAR Treasury** untuk request→approve→execute (Astro UI **deprecated** — ADR-013) | Setup + kurang fleksibel untuk logika kustom | **DIPILIH** — council 2-of-3 untuk aksi owner |
| MPC (chain signatures / provider) | Tanpa single point | Pihak ketiga, cost, trust | Alternatif jangka panjang |
| KMS/HSM | Hardware-grade | Tidak native NEAR, cost/ops tinggi | Overkill tahap ini |
| Dedicated signer service | Fleksibel | Backend tidak pernah sign (ADR-012) — kontradiktif | Ditolak |

**FACT (riset 2026-10-01)**: kontrak 2FA multisig legacy bawaan NEAR Wallet lama **sudah deprecated** (wallet deprecated; ekosistem migrasi ke Sputnik DAO/NEAR Treasury/smart accounts) — JANGAN dipakai sebagai dasar.

**Struktur mainnet (DECIDED — finalisasi teknis di M4)**:
- Owner actions finansial (fee, treasury, upgrade): **Sputnik DAO V2 council 2-of-3** (via **NEAR Treasury**; Astro UI deprecated — ADR-013) + timelock 24 jam untuk param/upgrade (proposal DAO memberi delay bawaan).
- **Pause tetap single-key (reaksi cepat) — butuh desain guardian**: jika owner_id = DAO account, panggilan pause single-key akan gagal cek owner. Solusi: kontrak memiliki **daftar `pause_callers` terpisah** (guardian accounts, role SECURITY) yang hanya berhak atas `pause` — bukan owner penuh. Item RESEARCH REQUIRED kecil: integrasi Owner pattern near-sdk-contract-tools ↔ Sputnik DAO + desain `pause_callers` (lihat ADR-013).
- Rotation/recovery: tambah/hapus council member via proposal DAO (mekanisme bawaan) — prosedur lengkap §3a.

## 3a. Prosedur rotasi council & guardian (mainnet)

> Dirujuk oleh [ADR-013](../decisions/ADR-013-key-management.md). Dijalankan sebelum mainnet (drill: SEC-IR-001) dan setiap ada perubahan anggota.

```text
RUTIN (anggota keluar/masuk):
1. Anggota baru membuat akun + kunci (generation offline §4); seed offline 2 lokasi terpisah.
2. Proposal DAO: add_member(new_member) — butuh 2-of-3 approve.
3. Verifikasi: view DAO get_policy → council = 3 anggota benar.
4. Anggota lama keluar: proposal remove_member(old_member) — 2-of-3 approve.
5. Cabut akses lain anggota lama: SSH key VPS, GitHub deploy key, Telegram admin, akses DB.
6. Catat di log insiden/perubahan (tanggal, aktor, proposal id).

DARURAT (dugaan kompromi satu key — CS-1):
1. Ganti pause: guardian `pause_callers` memanggil pause SEGERA (single-key, tanpa DAO).
2. Proposal DAO remove_member(compromised) — 2-of-3 approve (2 anggota bersih cukup).
3. Rotasi kunci: buat key baru, add_member, verifikasi, unpause.
4. Rotasi secret terkait (SSH, deploy key, JWT, DB) — incident-response.md.
5. Postmortem + perbarui SEC-* status.

GUARDIAN PAUSE (mainnet):
1. Daftar `pause_callers` ditetapkan saat deploy (mis. 2 akun terpisah dari owner-DAO).
2. Guardian HANYA bisa pause (tidak unpause, tidak tarik dana) — unpause via DAO 2-of-3.
3. Drill pause/unpause wajib di testnet sebelum mainnet (SEC-IR-001).
```

**Prinsip**: rotasi rutin = 2-of-3 (satu anggota tidak cukup); darurat = pause dulu (cepat, single-key guardian) lalu perbaiki via DAO. Tidak ada jalur satu-akun yang bisa memindahkan dana.

## 4. Kebijakan

- **Generation**: offline; entropy OS CSPRNG.
- **Signing policy**: owner key TIDAK pernah dipakai oleh aplikasi/web; hanya CLI interaktif oleh manusia.
- **Revocation/rotation**: Sputnik DAO council mendukung tambah/hapus member via proposal (mainnet); MVP: rotasi = redeploy owner config atau akun baru.
- **Separation of duties**: MVP tidak ada (satu orang); mainnet: FINANCE ≠ SECURITY ≠ DEVELOPER (access-control-matrix.md).
- **Tanpa pengecualian**: seed/private key TIDAK ADA di git, chat, screenshot, `.env` frontend, log.

## 5. Upacara pembuatan kunci (key-generation ceremony)

> Dijalankan untuk **setiap kunci kritis** (owner/DAO member, deploy, backup) — bukan hanya saat bootstrap. Tujuan: kunci dibuat offline dengan entropy tepercaya, backup terverifikasi, dan **tidak ada salinan tak terduga**. Status: PROPOSED (drill wajib sebelum mainnet — SEC-IR-001).

```text
PERSIAPAN (sebelum hari-H)
1. Tentukan role kunci & siapa pemegangnya (1 kunci = 1 pemegang; tidak ada sharing).
2. Siapkan mesin OFFLINE yang belum pernah terhubung internet (air-gapped); OS bersih, tanpa
   kamera/mikrofon aktif, tanpa sinkronisasi cloud.
3. Siapkan media backup: ≥2 lokasi terpisah (mis. brankas berbeda); opsi steel plate tahan api/air.
4. Catat rencana di log perubahan (tanggal, aktor, role, akun target) — TANPA secret.

PELAKSANAAN (hari-H, minimal 1 saksi bila tersedia)
5. Boot mesin offline; verifikasi hash image/OS (reproducible; catat hash di log).
6. Generate keypair via near-cli (atau tool kripto setara) — entropy dari OS CSPRNG.
   - Untuk DAO member: buat akun + kunci; jangan pakai kunci yang pernah dipakai role lain.
7. Verifikasi kunci: sign/verify pesan uji di mesin offline; pastikan public key benar.
8. Tulis seed phrase HANYA dengan tangan (tanpa printer jaringan, tanpa foto/screenshot).
9. Buat 2 salinan seed → 2 lokasi terpisah; segel + catat lokasi (bukan isinya) di log.
10. Impor public key / akun ke lingkungan yang membutuhkan (onboarding §7); TIDAK pernah seed.

PENUTUPAN
11. Hapus file kunci sementara di mesin offline (shred); reboot bila perlu.
12. Verifikasi akses: kunci baru bisa melakukan aksi yang diizinkan; kunci lama (bila rotasi) sudah tidak.
13. Catat di log perubahan + incident-response.md (bila bagian insiden); update status SEC-*.
```

- **Larangan**: generate kunci di VPS produksi, mesin ber-internet, layanan online, atau lingkungan CI.
- **Verifikasi backup**: sebelum dianggap valid, seed dibaca ulang di mesin offline **lain** dan menghasilkan public key yang sama (backup invalid sampai diuji).

## 6. Rotasi kunci langkah-demi-langkah (detail operasional)

> Melengkapi §3a (rotasi council & guardian). Bagian ini memerinci rotasi **semua kunci**, termasuk kunci off-chain. Prinsip: **rotasi dulu, baru bersihkan jejak**; rotasi dijadwalkan (tahunan) dan insidental (anggota keluar / dugaan bocor).

### 6a. Rotasi owner/DAO council key

```text
1. Buat kunci baru (upacara §5); verifikasi public key.
2. Proposal DAO add_member(new_key) → approve 2-of-3 → execute.
3. Verifikasi: view get_policy → council berisi kunci baru.
4. Uji aksi (mis. proposal no-op) dari kunci baru → sukses.
5. Proposal remove_member(old_key) → approve 2-of-3 → execute.
6. Verifikasi kunci lama TIDAK bisa lagi mengusulkan/menyetujui.
7. Bila rotasi karena dugaan kompromi: pause dulu via guardian (§3a DARURAT) sebelum langkah 2.
8. Catat log (proposal id, tanggal, aktor) + update incident-response.md.
```

### 6b. Rotasi deploy key (GitHub → VPS / `cargo near deploy`)

```text
1. Buat keypair baru di mesin operator (offline bila memungkinkan).
2. Pasang public key di server (`~/.ssh/authorized_keys`) atau akun deploy tujuan.
3. Uji deploy/SSH dengan kunci baru (jalur terpisah, tanpa mengganggu yang lama).
4. Hapus public key lama dari server + hapus GitHub Secret lama.
5. Verifikasi kunci lama ditolak.
6. Rotasi secret CI terkait (bila ada) + catat.
```

### 6c. Rotasi secret off-chain (JWT, DB, Telegram, backup S3)

| Secret | Langkah kunci | Efek |
|---|---|---|
| `JWT_SECRET` | generate baru → update env → restart API | semua sesi invalid (user login ulang) |
| `DATABASE_URL` / password DB | buat role/password baru → update `.env.server` → restart → hapus role lama | koneksi lama mati; app pakai kredensial baru |
| `TELEGRAM_BOT_TOKEN` | `/revoke` di BotFather → token baru → update env | alert kembali normal |
| `DEPLOY_SSH_KEY` | lihat §6b | akses deploy berpindah |
| `BACKUP_S3_*` | rotasi access key di object storage → update env cron backup | backup jalan dengan kredensial baru |

- Prosedur lengkap + urutan contain/rotate/verify ada di [disaster-recovery.md](../deployment/disaster-recovery.md) § Playbook kompromi secret.
- **Verifikasi wajib** setiap rotasi: secret lama tidak valid (uji akses harus gagal) + `/api/health` 200.
- Rotasi insidental **tidak** menunggu jadwal; rotasi rutin minimal **tahunan** atau saat anggota keluar (drill tercatat).

## 7. Onboarding anggota council (mainnet)

> Mainnet: council Sputnik DAO 2-of-3; anggota baru harus melewati onboarding terkontrol agar tidak menambah titik lemah. Status: PROPOSED (bagian dari SEC-KEY-002/SEC-CONTRACT-012).

```text
1. KEPUTUSAN: anggota baru disetujui council (minimal 2-of-3) — catat peran (FINANCE/SECURITY/DEVELOPER,
   access-control-matrix: tidak ada role catch-all).
2. IDENTITAS & PERAN: verifikasi identitas + konfirmasi pemisahan tugas (tidak ada satu orang memegang
   dua peran yang seharusnya terpisah).
3. KUNCI: anggota membuat akun + kunci via upacara §5 (offline); seed 2 lokasi terpisah.
4. ADD MEMBER: proposal DAO add_member(new_member) → approve 2-of-3 → execute.
5. VERIFIKASI: view get_policy → council = jumlah benar; uji anggota baru bisa menyetujui proposal uji.
6. AKSES OFF-CHAIN: bila relevan, berikan akses terpisah (VPS/DM/Telegram) dengan least privilege.
7. DOKUMENTASI: catat tanggal, aktor, proposal id; jangan pernah mencatat seed.
```

- Anggota **keluar** → jalankan §3a RUTIN (remove_member + cabut SSH/deploy/Telegram/DB) — akses off-chain ikut dicabut, bukan hanya on-chain.
- Onboarding MVP (testnet, 1 orang) = tidak berlaku; hanya owner tunggal + seed offline.

## 8. Shamir / split-seed (diskusi)

> Pertanyaan: apakah seed owner/DAO member dipecah dengan **Shamir Secret Sharing** (mis. SLIP-39) alih-alih satu seed utuh? Status: **PROPOSED / ⏳ open-by-design** — keputusan final saat finalisasi mainnet (M4), sejalan ADR-013.

| Pendekatan | Kelebihan | Kekurangan | Catatan |
|---|---|---|---|
| **Seed utuh, 2 salinan fisik** (model MVP) | sederhana; pemulihan langsung | satu salinan ditemukan → kunci penuh; bergantung disiplin pemegang | dipakai di MVP/testnet |
| **Shamir split-seed (k-of-n)** | tidak ada satu titik; threshold recovery | butuh ≥k pihak saat recovery; tooling & SOP tambahan; risiko kehilangan k share | kandidat mainnet |
| **Multisig on-chain (Sputnik DAO)** | threshold native NEAR; audit trail; tidak ada seed tunggal | slow UX; butuh ≥3 pemegang | **DIPILIH** untuk aksi finansial (ADR-013) |
| **MPC / chain signatures** | tanpa seed tunggal di satu mesin | pihak ketiga, cost, trust | alternatif jangka panjang |

- **Posisi saat ini**: aksi finansial mainnet sudah diproteksi **multisig on-chain 2-of-3** (lebih kuat daripada split-seed karena threshold diverifikasi kontrak, bukan manusia). Split-seed dipertimbangkan **tambahan** untuk kunci per-anggota (mengurangi risiko satu seed anggota bocor), bukan pengganti multisig.
- **Rekomendasi (PROPOSED)**: untuk tiap anggota council, pecah seed dengan **Shamir k-of-n** (mis. 2-of-3) dan simpan share di lokasi berbeda; SOP recovery wajib dilatih (drill) sebelum mainnet.
- **Risiko split-seed**: kehilangan share melebihi threshold = kunci hilang permanen; karena itu **wajib** ada jalur recovery DAO (§9) agar satu anggota hilang tidak mengunci seluruh council.
- **Larangan**: tidak ada share yang disimpan di git, cloud, chat, atau screenshot — sama seperti seed utuh.

## 9. Runbook pemulihan kunci (recovery)

> Skenario pemulihan kunci/otoritas. Setiap skenario punya pemicu, langkah, dan bukti. Dijalankan bersama [incident-response.md](./incident-response.md); drill sebelum mainnet (SEC-IR-001).

| Skenario | Pemicu | Langkah | Bukti |
|---|---|---|---|
| **Seed anggota hilang** (bukan bocor) | anggota kehilangan akses kunci | 1) anggota lain mengonfirmasi; 2) proposal DAO `remove_member(lost)` 2-of-3; 3) `add_member(new_key)` (upacara §5); 4) verifikasi policy; 5) catat | policy view + log proposal |
| **Kunci anggota bocor** (CS-1) | dugaan kompromi | 1) guardian `pause_callers` pause SEGERA; 2) `remove_member(compromised)` 2-of-3 (2 anggota bersih cukup); 3) rotasi semua secret terkait; 4) `add_member(fresh)`; 5) unpause via DAO; 6) postmortem | tx pause + policy view + log rotasi |
| **Owner key MVP bocor (testnet)** | dugaan kompromi | 1) pause owner; 2) buat akun owner baru; 3) transfer ownership bila fungsi tersedia ATAU redeploy + `migrate`; 4) rotasi SSH/deploy; 5) catat | tx + state kontrak |
| **Threshold DAO hilang** (< 2 anggota bisa approve) | ≥2 anggota hilang permanen | 1) **break-glass**: tidak ada jalur on-chain — pemulihan = deploy kontrak baru ke akun baru + migrasi state (jika memungkinkan) / re-list NFT; 2) kompensasi user via G7; 3) postmortem + perbaiki desain threshold | keputusan council + catatan insiden |
| **VPS hilang** (kunci ada di seed offline) | VPS mati/compromised | 1) provision VPS baru (TASK-028); 2) restore kunci dari seed offline ke mesin aman (bukan ke VPS bila tak perlu); 3) restore DB dari backup (disaster-recovery.md); 4) rotasi secret | log re-provision |
| **Seed phrase rusak/hilang (satu salinan)** | salinan pertama rusak | 1) gunakan salinan kedua; 2) verifikasi public key cocok; 3) buat salinan pengganti + segel ulang | verifikasi public key |

- **Prinsip**: untuk aksi finansial, tidak ada jalur satu-akun; recovery selalu lewat **threshold** (DAO 2-of-3) atau deploy ulang. Pause darurat boleh single-key (guardian) tetapi **unpause** lewat DAO.
- **Catatan penting**: redeploy ke akun **baru** tidak memindahkan state/NFT referensi lama (FACT) — opsi ini hanya untuk testnet; mainnet mengandalkan DAO + timelock.
- Setiap recovery = **postmortem wajib** + update status SEC-* + test regresi sebelum unpause.

## 10. Timeline migrasi mainnet (key management)

> Key management bukan task sekali jalan — gate per milestone. Milestone: [milestones.md](../../tasks/milestones.md); item di bawah = pekerjaan key management yang **wajib** selesai sebelum mainnet.

| Milestone | Pekerjaan key management | Gate / bukti |
|---|---|---|
| **M1–M2 (MVP/testnet)** | owner single-key + seed offline 2 lokasi (§2); alert Telegram tx owner tak terduga (TASK-026); dokumentasi rotasi manual | SEC-KEY-001; drill rotasi manual |
| **M3 (pra-mainnet)** | desain `pause_callers` (guardian) terintegrasi Owner pattern; setup Sputnik DAO V2 council 2-of-3 via NEAR Treasury; timelock 24 jam param/upgrade | SEC-CONTRACT-012 / SEC-KEY-002 → READY-FOR-IMPLEMENTATION |
| **M3 (pra-mainnet)** | drill: rotasi council (§3a), onboarding anggota (§7), pause/unpause guardian, recovery (§9) — di testnet | SEC-IR-001 bukti drill |
| **M4 (mainnet gate)** | migrasi owner → DAO; treasury → DAO-guarded; guardian `pause_callers` aktif; reproducible build dipublikasikan (NEP-330); decision Shamir/split-seed (§8) difinalkan | SEC-CONTRACT-012, SEC-CONTRACT-006, ADR-013 |
| **Pasca-mainnet** | rotasi rutin tahunan; rotasi insidental saat anggota keluar; audit akses berkala | log rotasi + access-control-matrix |

- **Urutan terkunci**: jangan migrasi ke mainnet sebelum (a) DAO council teruji di testnet, (b) drill pause/rotasi/recovery hijau, (c) semua SEC P0 mainnet gate berstatus READY-FOR-IMPLEMENTATION.
- Keputusan yang masih terbuka: integrasi teknis Owner pattern ↔ DAO account + desain `pause_callers` (RESEARCH REQUIRED kecil, ADR-013); kebijakan penarikan treasury (threshold) — lihat access-control-matrix.
- Referensi: [ADR-013](../decisions/ADR-013-key-management.md), [disaster-recovery.md](../deployment/disaster-recovery.md), [incident-response.md](./incident-response.md).
