# Fraud & Abuse

> Strategi deteksi penyalahgunaan. Prinsip mutlak: **fraud/risk score ≠ ownership state** — skor hanya memengaruhi tampilan (ranking, badge, discovery), TIDAK PERNAH memengaruhi settlement on-chain.

## 1. Matriks penyalahgunaan → deteksi → respons

| Abuse | Mekanisme deteksi (fase) | Respons |
|---|---|---|
| **Wash trading** (jual-beli ke diri via 2 akun) | On-chain: self-buy DITOLAK kontrak (INV-023, business rule ronde 1) — hanya menutup akun yang sama; wash via 2 akun tidak tersentuh cek itu. Off-chain heuristik (fase 2+): pola wallet-cluster A↔B bolak-balik, harga aneh, waktu dekat | Tidak menghentikan settlement (chain permissionless — FACT); flag volume "suspicious" di stats; biaya wash = fee 2% + gas (ekonomi deterrent) |
| **Fake volume** | Sama di atas; agregat volume tandai <threshold% wallet unik | Badge "volume tidak terverifikasi" pada stats |
| **Sybil / bot mint launchpad** | Allowlist phase whitelist (on-chain — FACT membatasi); max per wallet; pola multi-mint satu funder (fase 2) | Tidak ada blok on-chain tambahan di MVP; kurasi fase oleh creator/admin |
| **Fake collection / impersonation** | Report system (ronde 3) + similarity check nama (manual admin MVP) | Hide dari discovery (display-only); verified badge untuk asli |
| **Stolen NFT / scam listing** | Report + manual review | Same as above; komunikasi bahwa takedown tidak memengaruhi ownership |
| **Spam listing/report** | Min harga 0.01Ⓝ + storage pre-deposit (biaya spam, FACT); rate limit API | Blocklist display; rate limit |
| **Market manipulation (fake floor)** | Statistik dihitung dari sales riil; outlier filter | Tandai stats |
| **Impersonasi platform** (domain/mirror palsu) | TIDAK bisa dicegah teknis dari sisi kita — mitigasi: publish domain resmi di kanal resmi; onboarding page | Edukasi user |
| **Stolen wallet / drainer linkage** | UNKNOWN — RESEARCH REQUIRED (= **G11**, security-gap-analysis.md; fase 2+) | Warning banner pada listing dari wallet ter-flag |

## 2. Pemisahan domain (aturan)

```text
BLOCKCHAIN SETTLEMENT : permissionless — siapa pun boleh transaksi (FACT; kita tidak bisa & tidak boleh menahan)
FRAUD / RISK LAYER    : hanya memengaruhi discovery, ranking, badge, warning — di API/indexer
KEPUTUSAN             : admin via report queue (admin-security.md); SEMUA keputusan ber-log
```

## 3. Fase implementasi

- MVP: manual (report queue + admin). Hanya 2 otomatisasi: rate limit + unique constraint.
- Fase 2 (indexer): heuristik cluster + flag stats + wallet warnings.
- Fase 2+: skor risiko, integrasi intel pihak ketiga — **RESEARCH REQUIRED** (= G11; provider intel NEAR ecosystem).

## 4. Ambang deteksi konkret (semua **PROPOSED** — kalibrasi setelah baseline indexer fase 2)

> Angka di bawah **PROPOSED**, bukan business rule. Tidak memengaruhi settlement on-chain dalam kondisi apa pun. Ambang disimpan sebagai konfigurasi (bukan hardcode) agar bisa dikalibrasi tanpa deploy.

| ID | Heuristik | Ambang PROPOSED | Sinyal |
|---|---|---|---|
| H1 | `pair_loop` — pasangan wallet sama bolak-balik | ≥ 5 transfer NFT pasangan sama dalam ≤ 24 jam, selisih harga antar-transfer ≤ 10% | wash trading |
| H2 | `funder_sybil` — banyak wallet dari satu funder | ≥ 3 wallet penerima dana pertama dari funder sama dalam ≤ 1 jam, lalu mint/offer pada koleksi sama | sybil mint/offer |
| H3 | `fake_volume` — volume terkonsentrasi | koleksi 7 hari: wallet unik < 30% transaksi DAN top-2 wallet > 60% volume | fake volume |
| H4 | `floor_manipulation` — outlier harga | ≥ 3 sale pada rentang harga ≥ floor+50% atau ≤ floor−50% dalam ≤ 6 jam oleh cluster sama | manipulasi floor |
| H5 | `stolen_wallet` — kecocokan intel | alamat cocok daftar intel G11 (bila tersedia) | drainer/pencurian |
| H6 | `spam_offer` — offer massal | > 20 `make_offer` per wallet per jam ATAU > 200 per hari lintas koleksi | spam escrow |
| H7 | `report_brigade` — report terkoordinasi | ≥ 5 report ke target sama dari wallet berbeda dalam ≤ 10 menit, rasio akun baru (umur < 7 hari) > 50% | abuse report |

- Skor risiko gabungan (PROPOSED): `risk = Σ wᵢ·Hᵢ`, `wᵢ` default 1; `risk ≥ 3` → tandai untuk review manual; `risk ≥ 5` → badge "tidak terverifikasi" otomatis. H5 selalu → warning banner.
- Semua sinyal bersifat **display-layer** (badge/warning/ranking), tidak pernah memblokir transaksi.

## 5. Pseudocode & SQL deteksi

Pseudocode evaluator (fase 2, jalan di indexer/worker terjadwal, bukan di jalur API):

```text
FUNCTION evaluate_abuse(window_hours):
  FOR each heuristic H in [H1..H7]:
    config = load_threshold(H)          # dari tabel abuse_thresholds, bukan hardcode
    hits   = run_query(H, window_hours, config)
    FOR each hit in hits:
      upsert abuse_signal(
        subject_type = hit.type,        # wallet | collection | report_target
        subject_id   = hit.id,
        heuristic    = H,
        score        = hit.score,
        evidence     = hit.evidence,    # tx hash / receipt_id,event_index
        detected_at  = now()
      )
  recompute_risk_scores()               # gabungkan sinyal → risk
  apply_display_effects()               # badge/warning; TIDAK menyentuh settlement
```

Contoh SQL H1 (`pair_loop`) — hitungan transfer NFT pasangan wallet dalam jendela waktu:

```sql
-- events_raw: proyeksi event nft_transfer (fase 2). window = 24 jam (PROPOSED).
SELECT nft_contract_id,
       LEAST(from_account, to_account)    AS wallet_a,
       GREATEST(from_account, to_account) AS wallet_b,
       COUNT(*)                           AS transfers,
       COUNT(DISTINCT token_id)           AS tokens,
       MIN(block_timestamp)               AS first_seen,
       MAX(block_timestamp)               AS last_seen
FROM   events_raw
WHERE  event_type = 'nft_transfer'
  AND  block_timestamp >= now() - interval '24 hours'
GROUP  BY nft_contract_id, wallet_a, wallet_b
HAVING COUNT(*) >= 5
   AND COUNT(DISTINCT token_id) >= 2
ORDER  BY transfers DESC;
```

Contoh SQL H3 (`fake_volume`) — konsentrasi wallet unik per koleksi:

```sql
-- sales_raw: proyeksi event market_sale (fase 2). window = 7 hari (PROPOSED).
WITH s AS (
  SELECT collection_id, buyer, seller, price_yocto
  FROM   sales_raw
  WHERE  block_timestamp >= now() - interval '7 days'
)
SELECT collection_id,
       COUNT(DISTINCT buyer)::numeric / NULLIF(COUNT(*), 0) AS unique_ratio,
       SUM(price_yocto)                                     AS total_volume
FROM   s
GROUP  BY collection_id
HAVING COUNT(DISTINCT buyer)::numeric / NULLIF(COUNT(*), 0) < 0.30;
```

Contoh SQL H2 (`funder_sybil`) — wallet dengan funder awal sama (butuh data funding NEAR transfer):

```sql
-- transfers_raw: proyeksi transfer Ⓝ (fase 2). window = 1 jam (PROPOSED).
SELECT funder_account, COUNT(DISTINCT receiver_account) AS funded_wallets
FROM   transfers_raw
WHERE  amount_yocto >= 1000000000000000000000000   -- ≥ 1 Ⓝ (ambang PROPOSED)
  AND  block_timestamp >= now() - interval '1 hour'
GROUP  BY funder_account
HAVING COUNT(DISTINCT receiver_account) >= 3;
```

> Catatan: query di atas hanya contoh bentuk; skema final `events_raw`/`sales_raw` dimiliki [indexer-security.md](./indexer-security.md) + [database-schema.md](../database/database-schema.md). Dedup memakai identitas `(receipt_id, event_index)`.

## 6. Peran & tanggung jawab (per aktor)

| Aktor | Boleh apa | Tidak boleh | Jejak |
|---|---|---|---|
| **Reporter** (user) | kirim report (rate-limited 5/akun/hari), lampirkan bukti (URL/tx hash), banding hasil | menargetkan report massal; mengakses antrean | `reports` row (append-only), rate-limit log |
| **Admin** | putuskan report (hide/ignore), set verified, kelola blocklist; wajib `reason` | memutuskan tanpa step-up; mengubah `admin_audit`; akses dana | `admin_audit` (append-only, REVOKE UPDATE/DELETE) |
| **Creator** | laporkan impersonasi; kelola allowlist/phase koleksinya | menghapus/mengubah report; menandai verified koleksi sendiri | report + audit admin atas tindakannya |
| **Wallet (subject)** | banding atas flag/warning/badge yang dikenakan | menghapus sinyal; memengaruhi settlement | `abuse_signal` + jejak banding |
| **SUPPORT** (PROPOSED fase 2+) | baca report/kontak user (read-only) | aksi destruktif | query log |

## 7. False-positive & alur banding (appeal)

```text
1. TRIGGER     : flag/badge/warning tampil → subject melihat notifikasi "tandai tidak setuju".
2. SUBMIT      : subject kirim banding (scope `profile`/`report`) + alasan bebas-text; 1 banding per sinyal.
3. ANTREAN     : banding masuk antrean admin (sama seperti report), prioritas "appeal".
4. REVIEW      : admin memeriksa evidence (tx hash) — TIDAK melihat skor mentah sebagai kebenaran.
5. KEPUTUSAN   : admin pilih `upheld` (sinyal tetap) / `overturned` (sinyal dicabut) + wajib `reason`.
6. EFEK        : `overturned` → sinyal di-soft-delete dari display (evidence tetap tersimpan untuk audit).
7. SLA         : respons pertama ≤ 3 hari kerja; keputusan ≤ 7 hari kerja (PROPOSED).
8. BANDING-2   : maksimum satu tingkat banding; keputusan kedua final (dicatat).
```

- Semua keputusan banding = baris `admin_audit` dengan `action = 'appeal_decide'`.
- Banding TIDAK pernah menyentuh settlement on-chain — hanya lapisan display.

## 8. Retensi bukti (evidence retention)

| Jenis bukti | Retensi | Lokasi | Catatan |
|---|---|---|---|
| Tx hash / `(receipt_id, event_index)` | permanen (di chain) | chain + `events_raw` | sumber kebenaran, tidak bisa hilang |
| Baris `reports` + keputusan | permanen | PostgreSQL | riwayat moderasi |
| `admin_audit` | permanen | PostgreSQL (append-only) | REVOKE UPDATE/DELETE |
| `abuse_signal` + skor | 180 hari (PROPOSED) | PostgreSQL | cukup untuk audit & kalibrasi |
| Bukti lampiran eksternal (URL/screenshot) | 90 hari (PROPOSED) | object storage | hash dicatat; konten bisa kedaluwarsa |
| Log akses antrean | 14 hari | log VPS | rotasi logrotate |

- Prinsip: **keputusan & tx permanen; data turunan (skor/lampiran) boleh expire** — tapi tidak pernah dihapus sebelum jendela banding berakhir.
- Penghapusan evidence hanya via retensi otomatis terjadwal, bukan manual admin.

## 9. SLA respons (PROPOSED)

| Klasifikasi | Contoh | Respons pertama | Keputusan | Aksi display |
|---|---|---|---|---|
| Kritis (P0) | stolen NFT aktif, drainer ter-link, impersonasi platform | ≤ 4 jam | ≤ 24 jam | warning/hide segera |
| Tinggi (P1) | fake collection, fake volume besar | ≤ 24 jam | ≤ 3 hari | badge/hide |
| Sedang (P2) | wash trading kecil, spam listing | ≤ 3 hari | ≤ 7 hari | flag stats |
| Rendah (P3) | laporan tidak lengkap | ≤ 7 hari | ≤ 14 hari | tidak ada |

- SLA ini target operasional **display-layer**; kegagalan SLA tidak pernah berdampak pada dana user (settlement permissionless — FACT).

## 10. Metrik program fraud (PROPOSED)

| Metrik | Definisi | Target awal |
|---|---|---|
| Detection precision | % sinyal yang dikonfirmasi benar setelah review | ≥ 70% |
| Recall sampel | % kasus diketahui yang terdeteksi (audit sampel) | ≥ 60% |
| Appeal rate | % sinyal yang dibanding | ≤ 5% |
| Appeal overturn rate | % banding yang dikabulkan | ≤ 20% (di atas itu → kalibrasi ambang) |
| Waktu respons pertama | median per kelas | sesuai §9 |
| False-positive rate | % sinyal dicabut | ≤ 15% |
| Cakupan intel | % wallet ter-flag dengan sumber intel (H5) | diukur saat G11 aktif |

- Metrik ditinjau per kuartal; ambang §4 dikalibrasi dari metrik ini.

## 11. Status & gap

| Item | Status | Owner | Catatan |
|---|---|---|---|
| Rate limit + unique constraint (MVP) | DECIDED | backend | SEC-API-001 / SEC-AUTH-005 |
| Heuristik H1–H7 (fase 2) | PROPOSED | indexer | butuh indexer aktif |
| Skor risiko + badge otomatis | PROPOSED | indexer | kalibrasi dari metrik §10 |
| Integrasi wallet threat-intel (**G11**) | ⏳ **open-by-design** | security | riset fase 2+; belum ada katalog jelas di ekosistem NEAR; owner/date di [security-gap-analysis.md](./security-gap-analysis.md) §G11 |

- **G11**: tetap ⏳ open-by-design (bukan hilang) — sampai provider intel NEAR teridentifikasi, H5 tidak diaktifkan dan warning banner bergantung report manual.
