# Incident Response

> Prosedur DESAIN sebelum produksi. Tim MVP = 1 orang + agent → prosedur disederhanakan tapi struktur tetap lengkap. Channel utama: Telegram alert (ronde 12).

## 1. Tingkat insiden

| Sev | Definisi | Contoh | Target respons |
|---|---|---|---|
| S1 | Dana/aset user berisiko AKTIF | exploit kontrak dieksploitasi, owner key dicuri | segera (menit): pause/break-glass |
| S2 | Kontrol rusak, dana belum hilang | key leak terdeteksi, CI compromised, API auth bypass | jam: rotasi/patch |
| S3 | Degraded / data off-chain rusak | API down, indexer korup, spam massal | hari: perbaikan normal |

## 2. Playbook per skenario (ringkas; detil langkah di catastrophic-failure-scenarios.md)

| Skenario | Detection | Containment | Eradication/Recovery | Komunikasi | SEC terkait |
|---|---|---|---|---|---|
| Owner key compromise | tx owner tak dikenal (monitor NearBlocks), anon warning | **Pause segera** (owner key — jika penyerang sudah pakai key, pause race condition: buat akun baru dengan **kunci FRESH yang tidak pernah terekspos**, bukan seed yang mungkin bocor, + deploy baru) | deploy kontrak baru + migrasi listing; rotasi semua kunci; **playbook mass-revoke: informasikan user `nft_revoke_all` per token + jalankan `remove_stale_listing`** (G6); forensik VPS | status page + Telegram community; jujur cakupan dampak | SEC-KEY-002, SEC-CONTRACT-012, SEC-IR-001 |
| Market exploit aktif | pola settlement aneh (volume/payout anomaly, manual + alert) | pause; simpan bukti tx | root cause → patch → redeploy; refund manual bila perlu | sama; kompensasi user = **G7 DEFAULT** (insurance fund 0.1% fee, pencairan via proposal DAO) | SEC-CONTRACT-007, SEC-IR-001 |
| API/Frontend compromise | defacement, CSP violation spike, alert uptime | take offline (Caddy 503), revoke SSH/deploy keys | rebuild dari clean artifact + rotate secrets | wajib umumkan bila user data tersentuh (profiles) | SEC-FE-001, SEC-AUTH-003 |
| CI/CD compromise | workflow asing, secret access log | revoke GH secrets + deploy key | audit commit history sejak kemungkinan titik kompromi; rebuild | internal dulu, publik bila artifact terdistribusi | SEC-CICD-001/002/003 |
| DB compromise | anomali query, akses asing | stop app; assess (audit table integrity) | restore backup; rotate DB creds | profiles/report data minim — evaluasi kewajiban notifikasi | SEC-DB-001/003, SEC-INFRA-002 |
| Loss of off-chain admin access (semua allowlist hilang) | admin tidak bisa login panel | gunakan break-glass SQL via SSH | rebuild allowlist + audit entri manual | internal | SEC-ADMIN-001 |
| RPC compromise/malicious | data inkonsisten antar provider | switch provider fallback | lapor provider | internal | SEC-INDEX-002 |
| Indexer korup (fase 2) | reconciliation mismatch | tandai indexer degraded (UI badge) | rebuild dari events_raw | internal | SEC-INDEX-001/002 |

## 3. Prosedur umum (semua insiden)

```text
DETECT      : alert Telegram / laporan user / monitor manual
CONTAIN     : matikan vektor (pause/takedown/offline) — dulu, cepat
COMMUNICATE : status ke user (channel resmi) — jujur, tanpa jaminan palsu
ERADICATE   : root cause, patch, rotasi kredensial
RECOVER     : restore/verify (restore drill = bukti bisa pulih)
FORENSICS   : simpan log/tx hash/snapshot SEBELUM dibersihkan
POSTMORTEM  : blameless doc → action items → update dokumen keamanan ini
```

## 4. Persiapan yang wajib ada SEBELUM mainnet (SEC-IR-*)

- [ ] Kontak darurat + akses kedua ke VPS/DM (jika pemegang kunci utama tak tersedia)
- [ ] Dokumen "siapa boleh apa saat S1" (access-control-matrix)
- [ ] Status page / kanal komunikasi resmi siap
- [ ] Simulasi pause + unpause sudah dilakukan di testnet (bukti di test-cases)
- [ ] Restore drill database sukses (SEC-DB-004)
- [ ] Prosedur kompensasi user — **G7: DEFAULT diterapkan (2026-10-01)**: *insurance fund* 0.1% dari fee 2% masuk fund terpisah; aktif di mainnet; pencairan hanya via proposal Sputnik DAO (multi-sig) untuk insiden terverifikasi; kompensasi tidak dijamin otomatis. **Bisa di-override user kapan saja.**
- [ ] Skrip mass-revoke helper (G6 — `nft_revoke_all` per token + `remove_stale_listing`) — TASK implementasi

> SEC-IR-001 (drill pause & restore), SEC-IR-002 (kontak darurat + akses kedua), SEC-IR-003 (kanal komunikasi resmi) — terdefinisi di security-requirements.md.

## 5. Batas

- Ini desain prosedur, BUKAN bukti kemampuan respons. Kemampuan baru terbukti via drill (Phase D security roadmap).

## 6. On-call roster & peran (MVP = 1 orang + agent)

| Peran | Tanggung jawab saat insiden | MVP (sekarang) | Mainnet (PROPOSED) |
|---|---|---|---|
| Incident Commander (IC) | keputusan akhir, deklarasi sev, komunikasi | PLATFORM_OWNER | rotasi on-call |
| Security Lead | pause/break-glass, forensik | PLATFORM_OWNER | SECURITY (terpisah) |
| Infra/DevOps | VPS, deploy, rotasi secret, DNS | PLATFORM_OWNER | INFRA |
| Comms | status page + Telegram community | PLATFORM_OWNER | peran terpisah |
| Scribe | catat timeline + bukti | agent | peran terpisah |
| Finance (kompensasi) | pencairan insurance fund (G7) | PLATFORM_OWNER | FINANCE |

- MVP: satu orang memegang semua peran — karena itu **akses kedua wajib disiapkan** (SEC-IR-002) agar insiden tidak terkunci pada satu pihak.
- On-call mainnet: rotasi mingguan (PROPOSED); kontak darurat di [disaster-recovery.md](../deployment/disaster-recovery.md) §Kontak eskalasi.
- Eskalasi: S1 → IC segera; S2 → IC ≤ 1 jam; S3 → antrean normal.

## 7. Template notifikasi

**T1 — Status awal (S1/S2), publik:**

```text
[NearSea Status] <SEV> — <judul singkat>
Waktu (UTC): <YYYY-MM-DD HH:MM>
Dampak: <apa yang terdampak; apa yang AMAN>
Status: investigasi berlangsung / mitigasi berjalan
Tindakan user: <mis. "jangan bertransaksi dulu" / "tidak ada tindakan">
Update berikutnya: ≤ <30 menit / 2 jam>
Kebenaran aset: kepemilikan NFT selalu di wallet Anda; settlement on-chain tidak dapat diubah.
```

**T2 — Update berkala:**

```text
[NearSea Status] Update #<n> — <judul>
Sejak <waktu>: <perkembangan>
Saat ini: <status>
Ekspektasi pemulihan: <perkiraan / belum diketahui>
```

**T3 — Pemulihan:**

```text
[NearSea Status] RESOLVED — <judul>
Durasi: <mulai>–<selesai> UTC
Dampak terkonfirmasi: <ringkas; apakah ada dana/profil terdampak>
Tindakan yang diambil: <ringkas>
Langkah pencegahan: <ringkas>
Postmortem: <tanggal / kanal>
```

**T4 — Internal (Telegram admin) S1 trigger:**

```text
S1 ALERT — <skenario>
Deteksi: <alert/rule> pukul <UTC>
Aksi segera: <pause / revoke / take offline>
IC: <nama> · Scribe: <agent>
Bukti: <tx hash / receipt_id,event_index / log path>
```

- Aturan komunikasi: **jujur, tanpa jaminan palsu**; jangan menyebut angka korban sebelum terverifikasi; selalu tegaskan kepemilikan NFT ada di wallet (ADR-007).

## 8. Legal / regulasi (catatan timeline)

> Catatan kesadaran, bukan nasihat hukum. Data minim-PII (database-security §1) → kewajiban notifikasi rendah, tetapi tetap dievaluasi per insiden.

| Pemicu | Pertimbangan | Target waktu |
|---|---|---|
| Kebocoran data pribadi (profil/bio — minim) | kewajiban notifikasi bergantung yurisdiksi pengguna | evaluasi ≤ 72 jam sejak konfirmasi |
| Kebocoran dana user | komunikasi publik + kompensasi (G7) | pengumuman ≤ 24 jam |
| Permintaan penegak hukum | jalur resmi; hanya data yang kita miliki (minim) | respons sesuai surat resmi |
| Insiden pihak ketiga (provider RPC/IPFS) | bukan tanggung jawab notifikasi kita, tapi diumumkan | ≤ 24 jam |

- Karena platform non-custodial (ADR-007), tidak ada dana user yang disimpan; ini menurunkan profil regulasi (bukan custodial service).
- Konsultasi hukum = keputusan bisnis (⏳ open-by-design); dokumen ini hanya menetapkan timeline evaluasi internal.

## 9. Chain-of-custody bukti forensik

```text
1. IDENTIFIKASI : catat sumber bukti (log, tx hash, snapshot DB, capture memori).
2. AKUISISI     : salin ke direktori bukti TERPISAH, read-only; JANGAN bekerja di salinan asli.
3. HASH         : sha256 setiap file; catat di evidence-log.
4. LABEL        : evidence-log berisi: id, waktu (UTC), sumber, sha256, pengumpul, lokasi.
5. PENYIMPANAN  : object storage terpisah + akses terbatas (bukan VPS yang mungkin kompromi).
6. INTEGRITAS   : verifikasi sha256 sebelum/sesudah transfer; jangan modifikasi file.
7. RANTAI       : setiap perpindahan/perubahan akses dicatat (siapa, kapan, kenapa).
8. RETENSI      : bukti insiden disimpan ≥ 1 tahun (PROPOSED); tx chain permanen.
```

```bash
# Contoh: akuisisi + hash bukti (jalankan di VPS, simpan hasil ke storage terpisah).
mkdir -p /evidence/$(date -u +%Y%m%dT%H%M%SZ)
cp /var/log/auth.log* /evidence/<ts>/ 2>/dev/null
sha256sum /evidence/<ts>/* > /evidence/<ts>/SHA256SUMS
# tx hash / event: catat (receipt_id, event_index) di evidence-log (bukan disalin).
```

- Aturan: **jangan membersihkan log sebelum bukti diakuisisi** (prosedur umum §3 langkah FORENSICS).
- Bukti on-chain (tx) tidak dapat dimodifikasi → chain-of-custody cukup dengan mencatat `(receipt_id, event_index)`.

## 10. Runbook S1 menit-per-menit (owner key / exploit aktif)

```text
T+0   Alert `contract_owner_tx` / `contract_payout_fail` masuk Telegram.
T+0   IC (PLATFORM_OWNER) deklarasi S1; buka kanal insiden (Telegram khusus).
T+2   Verifikasi: tx benar dari owner/DAO? bandingkan hash implementasi + saldo escrow.
T+3   CONTAIN: guardian `pause_callers` panggil pause (single-key, cepat). Catat tx hash.
T+5   Konfirmasi: mutasi berhenti; cancel/withdraw user tetap terbuka (INV-022).
T+5   Scribe mulai timeline; akuisisi log (auth.log, app log) + hash (§9).
T+10  Tentukan vektor awal: key bocor? bug kontrak? tx upgrade?
T+15  Bila key bocor → anggap kompromi: siapkan akun/kunci FRESH (jangan seed lama).
T+20  Komunikasi T1 (status awal) ke status page + Telegram community.
T+30  Revoke akses: SSH/deploy key, Telegram token, DB creds, CI secrets (§8).
T+45  Forensik: identifikasi semua tx penyerang (dari alert + explorer) → daftar terdampak.
T+60  Rencana eradication: patch/deploy baru; jangan unpause sebelum verifikasi.
T+90  Bila perlu: jalankan playbook mass-revoke (nft_revoke_all + remove_stale_listing, G6).
H+2   Update T2; target RTO < 4 jam (disaster-recovery.md).
H+4   Verifikasi kontrak baru / patch; unpause via DAO (mainnet) atau owner (testnet).
H+24  Postmortem awal; putuskan kompensasi G7 (proposal DAO bila ada korban).
D+7   Postmortem final + action items + update SEC-* status.
```

- Setiap langkah dicatat dengan timestamp UTC; runbook ini **dilatih** via drill (SEC-IR-001, §12).
- Bila owner key tidak tersedia untuk pause → gunakan jalur guardian; bila keduanya tidak ada → eskalasi break-glass.

## 11. Template postmortem (blameless)

```md
# Postmortem — <judul> (<tanggal>)

- **Severity**: S1/S2/S3
- **Durasi**: <mulai>–<selesai> UTC (<total>)
- **Skenario**: <CS-n / playbook §2>
- **Deteksi**: <alert/rule/laporan> — waktu deteksi
- **Dampak**: <dana/profil/data; jumlah terdampak; apakah dapat dibalikkan>
- **Root cause**: <sebab teknis + sebab organisasi (5 why)>
- **Timeline**: <T+0 … resolusi, dengan timestamp>
- **Yang berjalan baik**: …
- **Yang gagal**: …
- **Action items**: [ ] <aksi> — owner — target
- **Update dokumen**: <SEC-* status, playbook, monitoring rule, test baru>
- **Kompensasi**: <G7 bila ada korban — status proposal DAO>
```

- Blameless: fokus sistem & proses, bukan menyalahkan individu.
- Wajib dipublikasikan internal ≤ 7 hari; publik bila insiden berdampak user.

## 12. Kalender drill

| Drill | Validasi | Frekuensi | Owner | Tanggal |
|---|---|---|---|---|
| Pause/unpause kontrak (testnet) | SEC-IR-001, CS-1/CS-9 | sebelum mainnet, lalu semesteran | SECURITY | ⏳ relatif: M4−2 minggu |
| Restore DB | SEC-DB-004 | per kuartal | PLATFORM_OWNER | ⏳ relatif: tiap awal kuartal |
| Re-provision VPS | disaster-recovery | per semester | PLATFORM_OWNER | ⏳ relatif: M4−1 bulan |
| Rotasi secret | secrets-and-gitignore §6 | tahunan / saat keluar | PLATFORM_OWNER | ⏳ relatif |
| Simulasi S1 tabletop | §10 runbook | sebelum mainnet, lalu tahunan | IC | ⏳ relatif: M4−1 bulan |
| Uji alert Telegram | monitoring.md | per kuartal | PLATFORM_OWNER | ⏳ relatif |
| Mass-revoke helper (G6) | CS-1 recovery | sebelum mainnet | SECURITY | ⏳ relatif: M4−2 minggu |

- Semua tanggal **relatif ke milestone** (M4) atau **⏳ open-by-design** — belum ada tanggal kalender tetap (tanpa tanggal target, lihat [milestones.md](../../tasks/milestones.md)).
- Hasil drill dicatat di sini + update status SEC-*; drill yang gagal = blocker mainnet.

## 13. Status

- Prosedur, template, runbook S1, chain-of-custody, kalender drill — **PROPOSED** (dibuktikan via drill, Phase D roadmap).
- SEC-IR-001 (drill pause & restore) — P0, mainnet gate.
- SEC-IR-002 (kontak darurat + akses kedua) dan SEC-IR-003 (kanal komunikasi) — P1, MVP.
