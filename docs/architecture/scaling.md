# Scaling

> Rencana pertumbuhan NearSea: **horizontal & vertical**. Prinsipnya: lapisan aplikasi
> stateless, semua state di tempat yang bisa di-replikasi (DB/object storage) atau di chain.
> Terkait: [infrastructure.md](./infrastructure.md), [backend-architecture.md](./backend-architecture.md),
> [tech-stack.md](./tech-stack.md), [indexer-security.md](../security/indexer-security.md).

## 1. Prinsip

1. **Stateless di lapisan aplikasi** — FE/API tidak menyimpan state di memori proses; sesi di
   DB/redis, bukan di instance. Ini syarat utama horizontal scaling.
2. **Scale out dulu, up kemudian** — mulai 1 VPS (MVP), naik jumlah instance sebelum menaikkan
   spesifikasi mesin.
3. **Chain tetap otoritas** — tidak ada state uang yang di-cache; cache hanya proyeksi read.
4. **Ukur sebelum menambah** — scaling dipicu metrik (§8), bukan tebakan.

## 2. Vertical scaling (naikkan spesifikasi)

| Tier | Kebutuhan | Kira-kira untuk |
|---|---|---|
| T0 (MVP) | 2 vCPU / 2 GB / 40 GB SSD | demo M1, trafik rendah |
| T1 | 4 vCPU / 8 GB / 100 GB SSD | testnet publik, trafik sedang |
| T2 | 8 vCPU / 16 GB / 250 GB NVMe | pra-mainnet, trafik naik |

- Naikkan RAM/CPU sebelum menambah instance bila bottleneck **single-thread** (mis. build, DB tunggal).
- PostgreSQL paling sering jadi bottleneck pertama → lihat §4 sebelum menaikkan tier.

## 3. Horizontal scaling (tambah instance)

```text
            ┌────────────┐
  user ──►  │  LB / CDN  │  (TLS, rate limit, cache statis)
            └─────┬──────┘
        ┌─────────┼─────────┐
        ▼         ▼         ▼
   [ app#1 ]  [ app#2 ]  [ app#3 ]   ← stateless (Next.js FE + API routes)
        └─────────┼─────────┘
                  ▼
        ┌───────────────────┐
        │  PgBouncer (pool) │
        └────────┬──────────┘
          ┌──────┴──────┐
          ▼             ▼
   [ PG primary ]  [ PG replica(s) ]   ← replica untuk read
```

- **App layer**: beberapa container identik di belakang load balancer. Karena stateless,
  tidak butuh sticky session.
- **Sesi**: token (JWT) + denylist di DB/redis → konsisten lintas instance (SEC-AUTH-003/006).
- **Read/write split**: query read (discovery, profil, riwayat) → replica; write → primary.
- **Connection pooling**: PgBouncer di depan PostgreSQL (wajib begitu instance > 1).

## 4. Database

- **Read replica** untuk beban baca (listing discovery, halaman koleksi, leaderboard).
- **Index** yang tepat pada kolom pencarian (nama, harga, waktu) — hindari full scan.
- **Partisi** tabel besar (riwayat/activity, event mentah indexer) per waktu.
- **Retensi**: arsipkan data lama ke object storage; DB tetap ramping.
- **Backup** tetap harian (SEC-DB-001) dan **tidak** membebani primary (jalankan dari replica bila ada).

## 5. Cache

- **MVP**: tanpa cache aplikasi (sesuai tech-stack); read langsung RPC + NearBlocks.
- **Fase 2 (indexer)**: cache hasil agregasi (floor, volume, rarity) di redis; invalidasi saat
  event baru masuk.
- **Cache metadata/media**: hash-keyed dengan TTL (lihat [metadata-security.md](../security/metadata-security.md));
  kegagalan cache = placeholder, bukan error user.
- **Aturan**: cache hanya untuk read; transaksi uang **selalu** re-verify on-chain (SEC-ORDER-003).

## 6. CDN / WAF

- MVP: tanpa CDN (risiko diterima — origin IP terekspos, G12 terbuka).
- Saat trafik naik: **CDN/WAF di depan** (mis. Cloudflare) untuk cache statis, mitigasi DoS,
  dan menyembunyikan origin. Ini menutup G12.
- Rate limit app tetap ada sebagai lapis kedua (SEC-API-001).

## 7. RPC & chain (bagian yang tidak bisa "di-scale" sendiri)

- **RPC**: sebar beban ke beberapa provider + failover terkunci FASTNEAR → official → dRPC
  dengan health check bandingkan block hash ([infrastructure.md](./infrastructure.md)).
  Bila perlu throughput lebih → provider berkunci (QuickNode/Tatum/ZAN).
- **Read**: cache view call yang jarang berubah (metadata koleksi), fallback antar provider.
- **Batas chain (FACT)**: sharding NEAR berbasis account — semua listing ada di **satu akun
  kontrak market**, sehingga pemanggilan ke kontrak itu diproses **berurutan di satu shard**.
  Throughput market dibatasi kapasitas shard tersebut.
  - Bila kontensi menjadi bottleneck nyata → opsi fase lanjut: **sharded market contracts**
    (beberapa kontrak market, routing per koleksi) — ini perubahan arsitektur besar, wajib ADR baru.
  - Selama MVP: batas ini jauh di atas kebutuhan (gas murah, finality ~1.3 dtk).

## 8. Indexer (fase 2)

- Sumber stream **Neardata** (NEAR Lake DEPRECATED — dilarang).
- Ingest hanya blok **final** + dedup (`receipt_id`, `event_index`) → idempoten (SEC-INDEX-002).
- Scale dengan menambah worker consumer; **partisi** per rentang blok; checkpoint tersimpan.
- Indexer tidak pernah jadi otoritas ownership/settlement (SEC-INDEX-001).

## 9. Pemicu scaling (indikator)

| Indikator | Aksi |
|---|---|
| CPU/RAM app > 70% konsisten | tambah instance (horizontal) |
| Latensi DB naik / lock contention | read replica + pooling + index |
| Error rate RPC naik / rate limit | tambah provider, aktifkan cache read |
| Beban baca discovery tinggi | CDN + cache agregasi |
| Puncak event indexer tertinggal | tambah worker indexer |

## 10. Kapasitas per tier

> Angka **PROPOSED** (estimasi perencanaan, bukan janji). Dikalibrasi setelah load test (§12) & metrik nyata (§9). Tier selaras §2.

| Tier | Konfigurasi | Perkiraan kapasitas | Bottleneck tipikal |
|---|---|---|---|
| T0 (MVP) | 2 vCPU / 2 GB / 40 GB SSD, 1 app, 1 DB | ratusan–ribuan request/hari; puluhan tx/hari | DB + RPC rate limit |
| T1 | 4 vCPU / 8 GB / 100 GB SSD, 1–2 app, 1 DB | puluhan ribu request/hari; ratusan tx/hari | DB read, RPC |
| T2 | 8 vCPU / 16 GB / 250 GB NVMe, 2–3 app, primary+replica, PgBouncer | ratusan ribu request/hari; ribuan tx/hari | chain shard (semua listing satu akun market — §7) |

- **Catatan penting**: throughput **market** dibatasi kapasitas shard akun kontrak market (§7) — bukan kapasitas VPS. Tier T2 menaikkan lapisan aplikasi/DB, tetapi batas chain tetap.
- **Beban read discovery MVP** ada di RPC (ADR-004), bukan di VPS → kapasitas RPC sering jadi batas lebih dulu.

## 11. Biaya per tier

> Selaras [infrastructure.md](./infrastructure.md) §Model biaya; indikatif USD/bulan.

| Pos | T0 | T1 | T2 |
|---|---|---|---|
| VPS | $5–20 | $20–40 | $40–80 |
| DB tambahan / replica | — | — | $20–40 |
| Object storage backup | $1–5 | $1–5 | $5–10 |
| RPC berbayar (opsional) | $0 | $0–50 | $50–200 |
| CDN/WAF (opsional) | $0 | $0 | $0–20 |
| IPFS pinning | $0–20 | $0–20 | $20–50 |
| **Total indikatif** | **~$7–45** | **~$25–115** | **~$135–400** |

- **Driver biaya**: RPC throughput + storage (mainnet) + replica DB; compute VPS relatif murah.

## 12. Metodologi load test

> **PROPOSED** — dijalankan sebelum menaikkan tier / sebelum mainnet (bagian TASK-034).

```text
1. Baseline   → ukur T0: latensi p50/p95/p99, RPS, error rate, koneksi DB, hit RPC
2. Skenario   → (a) read discovery (view call), (b) API profil/report, (c) submit tx (write)
3. Beban naik → ramp RPS bertahap sampai SLO (§17) dilanggar → catat "knee"
4. Uji khusus → burst 20 pembeli 1 listing (race, bukan load — TC-016/017), double-submit
5. Analisis   → tentukan bottleneck (DB/RPC/app) → pilih aksi §9
6. Ulang      → setelah perubahan (index/replica/pooling) → bandingkan
```

| Aspek | Alat (kandidat) | Catatan |
|---|---|---|
| HTTP load | k6 / autocannon (PROPOSED) | Terhadap staging, bukan produksi |
| RPC load | script view-call | Pantau rate limit provider |
| DB | `pg_stat_statements`, `EXPLAIN ANALYZE` | Cari query lambat |
| Observabilitas | log + metrik (§6, monitoring.md) | Korelasi requestId |

- **Aturan**: load test **tidak** dijalankan terhadap mainnet/kontrak produksi dengan dana nyata; gunakan testnet/staging.

## 13. Konfigurasi LB & CDN

> MVP tanpa LB/CDN (risiko diterima, §6); diaktifkan saat trafik menuntut. Produk = **PROPOSED**.

| Komponen | Produk kandidat | Konfigurasi | Catatan |
|---|---|---|---|
| CDN/WAF | Cloudflare (free tier → pro) | cache statis, anti-DoS, sembunyikan origin | Menutup G12 |
| LB (app) | Caddy/nginx di depan app, atau LB cloud | round-robin, health check `/api/health`, TLS terminate | App stateless → tanpa sticky |
| Cache statis | CDN edge | `/_next/static/*`, gambar, font | Immutable hash → TTL panjang |
| Cache dinamis | **tidak** di-cache di edge | API per-akun `no-store` | Hindari data basi lintas user |

- **Aturan**: LB **tidak** boleh men-cache response API yang memuat data akun/session.
- **Header**: security headers (CSP/HSTS) dipasang konsisten di Caddy/CDN ([frontend-security.md](../security/frontend-security.md) §2).

## 14. Replikasi & penanganan replica-lag

> Read/write split (§3/§4). **PROPOSED** — aktif saat beban baca menuntut.

```text
write ─► primary ──(streaming replication)──► replica(s) ─► read query (discovery/profil/leaderboard)
              └─────────────────────────────────────────────► read "baca-sendiri" (read-your-writes)
```

| Aspek | Keputusan |
|---|---|
| Mode | Streaming replication (async) — PostgreSQL built-in |
| Routing | Write + read-after-write → primary; read berat (agregat, listing) → replica |
| Replica-lag | Pantau `pg_stat_replication` / LSN; bila lag > ambang → alihkan read ke primary sementara |
| Read-your-writes | Setelah user menulis (PATCH profil), baca berikutnya dari primary (hindari "data hilang") |
| Failover | Manual (MVP+); otomatis = PROPOSED (butuh tooling) |
| Backup | Jalankan dari replica agar tidak membebani primary (§4) |

- **Aturan**: data yang menentukan **transaksi uang** tidak pernah dibaca dari replica/DB — selalu view call on-chain (ADR-010/ADR-015).

## 15. Sizing PgBouncer

> Wajib saat instance app > 1 (§3). **PROPOSED**; angka dikalibrasi setelah load test.

| Parameter | Nilai awal (PROPOSED) | Alasan |
|---|---|---|
| Mode | `transaction` | Banyak koneksi pendek dari app |
| `default_pool_size` | 10–20 | ≤ `max_connections` Postgres |
| `max_client_conn` | 100–200 | App instance × pool |
| Total pool | ≤ Postgres `max_connections` (mis. 100) | Sisakan untuk admin/backup |
| `pool_mode` app | satu pool per service (web/indexer) | Isolasi beban |
| Timeout | `query_wait_timeout` pendek | Cegah antrean tak terbatas |

- **Pitfall**: `transaction` mode tidak mendukung `PREPARED STATEMENT` persisten — sesuaikan konfigurasi Prisma (`pgbouncer=true`).
- **Indexer fase 2**: role DB read-only terpisah, pool terpisah (database-security.md §2).

## 16. Redis — pilihan & HA

> **Fase 2 saja** (cache agregat/rate-limit lintas instance); MVP tanpa Redis (tech-stack.md).

| Aspek | Pilihan (PROPOSED) | Catatan |
|---|---|---|
| Peran | Cache agregat (floor/volume/rarity), rate-limit lintas instance, (opsional) session/denylist | §5 |
| Deployment | Container di VPS yang sama | Sederhana; HA bila perlu |
| Persistensi | RDB/AOF opsional | Cache boleh hilang (rebuild dari DB) |
| HA | Sentinel/Cluster = PROPOSED | Hanya bila cache jadi jalur kritis |
| Eviction | `allkeys-lru` + TTL | Cache tidak pernah jadi otoritas |
| Kegagalan | Fallback ke DB/RPC; **bukan** error user | Cache = optimisasi |

- **Aturan**: Redis **bukan** sumber kebenaran apa pun; kehilangannya tidak boleh menghentikan trading.

## 17. Target SLO/SLA

> **PROPOSED** (target internal). Angka final ditetapkan saat mainnet gate.

| Layanan | SLI | SLO target (PROPOSED) | Catatan |
|---|---|---|---|
| Frontend (read) | availability | 99.5%/bulan | Origin VPS tunggal (MVP) |
| API (auth/profil/report) | availability | 99.5%/bulan | Non-kritis untuk dana |
| API latency | p95 | ≤ 500 ms | Untuk endpoint non-chain |
| Settlement on-chain | keberhasilan tx sah | ≥ 99.9% (di luar gas/kondisi user) | Chain otoritatif |
| Data freshness notifikasi | jeda | ≤ 60 dtk | Polling MVP |
| Indexer (fase 2) | lag | ≤ N blok | Alert saat melewati |

- **Penting**: "downtime" FE/API **tidak** berarti dana tidak aman — settlement tetap on-chain (ADR-010). SLA di sini soal kenyamanan, bukan kustodi.

## 18. Autoscaling playbook & rollback

> Dipicu metrik §9. **PROPOSED**; manual dulu, otomatis nanti.

```text
[1] Deteksi   → alert metrik (CPU/RAM/latensi DB/error RPC) melewati ambang
[2] Triase    → tentukan bottleneck: app | DB | RPC | chain
[3] Aksi      → app  → tambah instance di belakang LB (§3)
                DB   → replica + pooling + index (§4/§14/§15)
                RPC  → tambah provider / cache read (§7)
                chain→ batas shard (§7) → eskalasi ADR sharded market (§19)
[4] Verifikasi→ cek SLO pulih (§17) + tidak ada regresi error
[5] Rollback  → bila aksi memperburuk: kembalikan konfigurasi sebelumnya
                (hapus instance / revert routing / turunkan pool)
[6] Catat     → postmortem singkat + update ambang bila perlu
```

| Aksi | Rollback |
|---|---|
| Tambah instance app | Hapus instance (stateless → aman) |
| Aktifkan replica | Alihkan read kembali ke primary |
| Naikkan pool PgBouncer | Turunkan; pantau `max_connections` |
| Aktifkan cache/Redis | Matikan; fallback DB/RPC |
| Naikkan tier VPS | Turunkan tier (maintenance window) |

- **Aturan**: perubahan scaling tidak mengubah kode kontrak; rollback kontrak terpisah (ci-cd.md §6).

## 19. Migrasi data untuk opsi sharded market

> Opsi fase lanjut **bila kontensi chain terbukti bottleneck** (§7). **OPEN QUESTION** — butuh ADR baru sebelum implementasi.

| Tahap | Aktivitas | Risiko | Mitigasi |
|---|---|---|---|
| 1. Evaluasi | Ukur kontensi shard akun market; buktikan bottleneck | Salah diagnosa | Data load test + metrik chain |
| 2. Desain | ADR baru: routing listing per koleksi ke beberapa kontrak market | Fragmentasi liquidity | Skema routing jelas |
| 3. Persiapan | Indexer menandai `market_contract` per listing/sale (kolom baru) | Data lama tanpa penanda | Backfill dari event (★ fase 2) |
| 4. Migrasi listing | Seller re-list ke market baru (non-custodial → listing bisa dicabut & dibuat ulang) | Listing lama tertinggal | Komunikasi + grace period |
| 5. Routing FE | FE memilih kontrak market berdasarkan koleksi | Bug routing | Test E2E + fallback |
| 6. Decommission | Kontrak lama hanya untuk cancel/withdraw | Dana tersangkut | Jalur withdraw permanen (escrow user-recoverable) |

- **Prinsip non-custodial memudahkan migrasi**: NFT tidak dipegang market; seller cukup revoke & list ulang ke market baru.
- **Escrow offer lama**: tetap user-recoverable via cancel/expire di kontrak lama (INV-005).
- **Wajib**: ADR baru + update `docs/` + tests sebelum implementasi (AGENTS.md).

## 20. Status

- Prinsip & rencana — **DECIDED (ronde 14)**.
- Implementasi horizontal (LB, replica, pooling) — **PROPOSED**, diaktifkan saat trafik menuntut.
- Sharded market contract — **OPEN QUESTION** (hanya bila kontensi chain terbukti jadi bottleneck; butuh ADR).
- Kapasitas/biaya per tier, metodologi load test, SLO/SLA, playbook autoscaling, migrasi sharded (§10–§19) — **PROPOSED** (angka indikatif; dikalibrasi saat load test & mainnet gate).
