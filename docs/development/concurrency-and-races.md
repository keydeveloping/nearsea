# Concurrency & Race Conditions

> Menjawab: **"kalau 1 NFT dibeli 20+ orang bersamaan, bagaimana?"** dan pola race lain.
> Terkait: [order-protocol-security.md](../security/order-protocol-security.md),
> [smart-contract-invariants.md](../security/smart-contract-invariants.md),
> [threat-model.md](../security/threat-model.md), [features/marketplace.md](../features/marketplace.md).

## 1. Ringkasan jawaban

**Bukan "kuat-kuatan RPC".** Penentu pemenang adalah **urutan eksekusi on-chain**, bukan
endpoint RPC mana yang dipakai. Skenario 20 pembeli untuk NFT A#1:

- **1 pembeli menang**, **19 kalah otomatis dan dananya kembali** — dijamin oleh state kontrak.
- RPC hanya pintu masuk; kegagalannya (rate limit) memengaruhi *pengiriman* tx, bukan kebenarannya.

## 2. Mengapa race aman: tiga lapis jaminan

### Lapis 1 — Serialisasi oleh chain (FACT)

- NEAR **tidak punya priority fee / gas auction** (berbeda dari Ethereum). Pembeli tidak bisa
  "menang dengan bayar gas lebih tinggi".
- Sharding NEAR berbasis **account**: satu akun berada di satu shard. Karena **semua listing
  berada di satu akun kontrak market**, semua pemanggilan `buy` ke kontrak itu diproses
  **berurutan** oleh shard yang sama — bukan paralel. Urutan ditentukan block producer dan
  deterministik.
- Di dalam satu blok, receipt dieksekusi berurutan; tidak ada eksekusi bersamaan pada state yang sama.

### Lapis 2 — Optimistic removal (DESIGN, pola tutorial RESEARCH.md §10)

- `buy` **menghapus entry sale lebih dulu**, lalu memicu promise cross-contract
  (`nft_transfer_payout`).
- Tx pembeli ke-2..20 mengeksekusi terhadap state yang sale-nya **sudah tidak ada** →
  assert gagal → **tx revert**.
- Revert = state kembali seperti semula dan **deposit Ⓝ tidak berpindah** (kembali ke pengirim).
  Yang hangus hanya gas (dibakar oleh jaringan) — bukan harga NFT.

### Lapis 3 — Unique key & atomic resolve

- **INV-007 / SEC-ORDER-005**: key listing unik (`contract_id.token_id`) → tidak mungkin ada
  dua listing aktif untuk token yang sama, tidak ada overfill.
- **INV-009**: offer yang sudah di-accept tidak bisa di-accept lagi (entry dihapus atomik).
- Jika promise **pemenang sendiri** gagal (mis. payout invalid), `resolve_purchase`
  mengembalikan state sale + refund buyer atomik → listing **aktif lagi**, pembeli lain bisa coba.

## 3. Hasil untuk 20 pembeli

| Pembeli | Yang terjadi |
|---|---|
| Ke-1 (pemenang) | Sale terhapus → NFT pindah → seller + royalti dibayar − fee 2% |
| Ke-2..20 | Sale sudah tidak ada → tx revert → deposit Ⓝ kembali (hanya gas hangus) |
| Pemenang gagal promise | Sale dipulihkan + refund → NFT tetap dijual, bisa dicoba lagi |

**Tidak ada** kondisi di mana dua orang membayar untuk NFT yang sama, atau NFT "hilang".

## 4. Peran & batas RPC

- RPC adalah **pintu masuk tx & sumber read** — bukan penentu pemenang.
- **Rate limit**: bila 20 orang memakai satu endpoint yang sama, yang bisa gagal adalah
  **pengiriman** tx (`429`/timeout), bukan kebenaran settlement. Mitigasi:
  - failover terkunci **FASTNEAR → official → dRPC** + health check bandingkan block hash
    (lihat [infrastructure.md](../architecture/infrastructure.md));
  - read call boleh di-cache/di-fallback; write call di-retry dengan backoff.
- **Degraded mode**: saat RPC bermasalah → FE menonaktifkan aksi tulis sementara, menampilkan
  status jelas ("jaringan sibuk, coba lagi"), dan tetap mengizinkan cancel/withdraw yang aman.
- **FE wajib re-verify** harga/ownership via view sebelum sign (SEC-ORDER-003) — ini penjaga
  cache basi, bukan penjaga race.

## 5. Double-submit & aksi berulang

| Kasus | Penanganan |
|---|---|
| Pembeli sama menekan Buy 2× | Tombol `pending-tx` disabled sejak submit; nonce per access key menolak duplikat; sale sudah terhapus pada tx pertama → tx kedua revert |
| Seller menekan Cancel 2× | Entry sale sudah terhapus → tx kedua revert (tidak ada efek ganda) |
| Dua orang `accept_offer` pada offer sama | Entry offer dihapus atomik sebelum settle (INV-009) → yang kedua revert + escrow tetap aman |
| Retry setelah timeout (tx sebenarnya masuk) | Cek status via tx hash dulu sebelum kirim ulang (hindari duplikat) |

## 6. Race pada bundle (kasus khusus)

- Bundle tidak punya rollback atomik lintas-receipt (FACT NEAR). Karena itu keandalan dicapai
  lewat **pre-validasi sebelum transfer pertama** (ownership + approval + simulasi payout semua
  item; INV-025), lalu agregasi payout (merge per receiver; > 10 receiver unik → ditolak saat
  `create_bundle`).
- Kegagalan residual mid-loop (mis. concurrent transfer) → status **PARTIAL** tercatat on-chain
  + event `market_bundle_partial` + refund dana belum terpakai + kompensasi via G7.
- Token dalam bundle aktif **tidak boleh** di-list/di-offer terpisah (INV-028) — mencegah race
  silang antara bundle dan listing tunggal.

## 7. UX pembeli yang kalah

- Sebelum sign: harga/ownership/status di-re-verify (SEC-ORDER-003).
- Setelah tx revert: tampilkan **satu** pesan `CONFLICT_SOLD` ("Sudah terjual — dana kamu
  kembali") via [error-handling.md](./error-handling.md) — bukan teks panic mentah.
- Listing yang sudah terjual hilang dari discovery (invalidate query saat final).

## 8. Yang TIDAK dipakai (dan alasannya)

- **Off-chain lock / antrean / mutex** — tidak perlu; chain sudah menjadi titik serialisasi.
- **Priority fee** — NEAR tidak mendukung; tidak ada jalur "bayar lebih untuk menang".
- **Off-chain order signature** — order = state on-chain (ADR-012); tidak ada risiko replay signature.

## 9. Cakupan test

- TC concurrent/double-buy ada di [testing/test-cases.md](../testing/test-cases.md)
  (race 20 pembeli, double-buy, double-accept).
- Invariant terkait: INV-007, INV-008, INV-009, INV-016, INV-025, INV-028.

## 10. Race di API off-chain (level DB)

> Seluruh §1–§9 di atas tentang **chain** (satu penulis logis: shard akun kontrak). API off-chain punya penulis **paralel** (banyak request) → butuh kontrol eksplisit di DB. Prinsip: API tidak menyentuh dana, jadi race di sini berakibat data (profil/report/audit) — bukan kerugian finansial.

### 10a. PATCH profil bersamaan (same-account)

| Skenario | Perilaku | Keputusan |
|---|---|---|
| Dua tab mengirim `PATCH /accounts/me/profile` hampir bersamaan | **Last-write-wins** — request yang commit terakhir menang; tidak ada merge field | **DECIDED** (selaras `endpoints.md`: idempotency last-write) |
| Client retry karena timeout | Header `Idempotency-Key` (§12) mencegah penulisan ganda tak sengaja | DECIDED |
| Ingin mencegah timpa-tulisan (optimistic) | **Opsional**: klien kirim `If-Match: <ETag>`; server tolak `412` bila `updated_at`/hash berbeda | **PROPOSED** (belum di MVP) |

- Implementasi last-write: satu `UPDATE ... SET ... , updated_at = now() WHERE account_id = $1` — **tanpa** read-modify-write terpisah (hindari lost update di aplikasi).
- Field yang dikirim hanya yang diubah (`PATCH` semantik); dua request yang mengubah **field berbeda** tetap bisa saling menimpa bila berurutan — diterima (satu user, satu akun).
- `profiles.UNIQUE(account_id)` + `INSERT ... ON CONFLICT (account_id) DO UPDATE` membuat create-vs-update race aman (tidak ada dua baris).

### 10b. POST report bersamaan (anti-spam)

- **Idempotensi primer = constraint DB**: `UNIQUE(reporter, target, hari_kalender)` ([database-security.md](../security/database-security.md) §3).
- Dua request identik bersamaan → satu `INSERT` sukses, yang lain melanggar unique → server mengembalikan **409 `CONFLICT_REPORT_EXISTS`** (bukan error 500).
- Tidak ada read-then-insert di aplikasi (itu race) — **selalu** andalkan constraint, tangkap pelanggaran unique → 409.
- Rate limit (5/akun/hari) ditegakkan token bucket di layer API ([api-security-architecture.md](../security/api-security-architecture.md) §9); bucket sendiri dijaga atomik (Redis INCR/Lua atau lock), bukan baca-lalu-tulis.

### 10c. Kapan pakai optimistic locking

| Aksi | Strategi | Alasan |
|---|---|---|
| PATCH profil (self) | last-write-wins | satu pemilik data; kehilangan edit minor dapat diterima |
| Keputusan report admin | **optimistic (conditional update)** | dua admin bisa memutus bersamaan; keputusan ganda = masalah audit |
| Set verified / blocklist | **optimistic + unique constraint** | state boolean/keberadaan tidak boleh ganda |
| Idempotency store | unique constraint + state machine | lihat §12 |

Pola conditional update (anti double-decide):

```sql
-- Hanya satu admin yang "menang"; yang kalah dapat 0 baris → 409.
UPDATE reports
   SET status = 'resolved', action = $2, decided_by = $3, decided_at = now()
 WHERE id = $1
   AND status = 'open'
RETURNING id;
-- rows affected = 0  → 409 CONFLICT_ALREADY_DECIDED
```

## 11. Locking di indexer

> Indexer = **penulis tunggal** proyeksi chain (SEC-INDEX-001/002). Race utama: dua instance menulis proyeksi sama (split-brain), atau menulis blok non-final.

| Kontrol | Mekanisme | Tujuan |
|---|---|---|
| **Single writer (leader)** | `pg_advisory_lock(<key>)` pada koneksi khusus selama proses ingest hidup | Hanya satu instance menulis; instance kedua menunggu/menolak start |
| **Batch per rentang blok** | proses blok `[n, n+k)` dalam satu transaksi DB (proyeksi + cursor) | Proyeksi & cursor naik atomik — tidak ada "cursor maju tapi data belum" |
| **Dedup event** | `UNIQUE(block_height, receipt_id, event_index)` di `events_raw` + `ON CONFLICT DO NOTHING` | Replay/at-least-once aman |
| **Final-only ingest** | Proses hanya blok **final**; blok non-final di-buffer | Mencegah proyeksi dari fork yang dibuang |
| **Rebuild eksklusif** | Ingest **dihentikan** (lock dipegang job rebuild) sebelum TRUNCATE proyeksi | Tidak ada race tulis saat rebuild ([migrations.md](../database/migrations.md) §5) |
| **Cursor persist** | Cursor disimpan di tabel job, di-update dalam transaksi batch yang sama | Resume deterministik setelah crash |

- Urutan eksekusi internal: `block_height` → urutan receipt dalam blok → `event_index` — sama seperti ordering notifikasi ([features/notifications.md](../features/notifications.md) § Ordering).
- **Failover**: lock dilepas saat proses mati (advisory lock putus dengan koneksi) → instance pengganti mengambil alih; replay dari cursor terakhir (idempoten).
- Indexer **tidak** pernah menulis tabel app-mutable/audit — dijaga GRANT/REVOKE ([database-security.md](../security/database-security.md) §8), bukan hanya konvensi.
- Dua job yang menyentuh tabel proyeksi sama (mis. rebuild + backfill) dilarang berjalan bersamaan; jalankan berurutan.

## 12. Desain idempotency-key untuk write API

Header `Idempotency-Key: <UUID v4>` untuk `POST`/`PATCH` (detail klien: [api-overview.md](../api/api-overview.md) § Idempotency-Key).

**Kunci penyimpanan**: `(account_id, method, path, key)` — bukan hanya `key`, agar dua akun tidak saling menimpa.

**State machine** (satu baris per kunci):

```text
ABSENT ──request masuk──► IN_PROGRESS ──selesai──► COMPLETED (simpan status+body)
                              │                        │
                              │ (request lain,          │ replay dengan key sama
                              │  key sama, bersamaan)   ▼
                              └──► 409 CONFLICT_*    kembalikan body tersimpan
                                   (request sedang     + header Idempotency-Replayed: true
                                    diproses)
```

| Aturan | Nilai |
|---|---|
| Insert kunci | `INSERT ... ON CONFLICT DO NOTHING` atomik; menang = pemroses, kalah = lihat state |
| State `IN_PROGRESS` + request lain | **409** `CONFLICT_*` (jangan blokir-menunggu di MVP) — klien retry setelah jeda |
| Body berbeda dengan key sama | 409 `CONFLICT_*` — bandingkan **hash body** (sha256) yang disimpan |
| Selesai | simpan `status_code`, `response_body` (≤ batas), `body_hash`, `completed_at` |
| Retensi | **24 jam** (⏳ final saat implementasi, selaras api-overview) + job pembersih TTL |
| Kegagalan di tengah | baris `IN_PROGRESS` dihapus (atau di-TTL) agar klien bisa retry |
| Cakupan | hanya endpoint non-idempoten (`POST/PATCH`); `GET` tidak perlu |
| Bukan pengganti constraint | `POST /reports` tetap unik `(account, target, hari)` di DB |

- Idempotency store = tabel DB (`idempotency_keys`) dengan unique index pada `(account_id, method, path, key)` — aman lintas-instance (bukan in-memory).
- Respons tersimpan **tidak** memuat data sensitif yang boleh basi (mis. token sesi) — endpoint auth tidak memakai store ini.

## 13. Parameter retry RPC (konkret)

> Selaras [error-handling.md](./error-handling.md) §13 dan [infrastructure.md](../architecture/infrastructure.md) (failover FASTNEAR → official → dRPC). RPC memengaruhi **pengiriman**, bukan kebenaran settlement (§1).

| Operasi | Maks percobaan | Backoff | Sebelum retry |
|---|---|---|---|
| Read (view call) | 3 | 200 ms → 600 ms → 1800 ms, jitter ±20% | failover ke provider berikutnya setelah percobaan ke-3 |
| Write (kirim tx) | 2 | 500 ms → 1500 ms | **cek status tx via hash** — jangan kirim ulang bila sudah masuk |
| Health check provider | 3 | 1 s → 2 s → 4 s | bandingkan block hash antar provider (deteksi node tertinggal) |

- Batas total waktu satu aksi user ≤ **15 detik**; setelah itu tampilkan status dan hentikan retry otomatis.
- `429` → hormati `Retry-After`; jangan retry sebelum jendela itu.
- Retry write **tidak** mengubah nonce/access key; nonce per access key menolak duplikat ([smart-contract-security-architecture.md](../security/smart-contract-security-architecture.md)).
- Degraded mode: bila semua provider gagal → nonaktifkan aksi tulis sementara (UI jelas), cancel/withdraw tetap tersedia (§4).
- Jitter wajib (mencegah semua klien retry serempak saat provider pulih).

## 14. Race aksi admin konkuren

| Skenario | Penanganan |
|---|---|
| Dua admin memutus report sama | Conditional `UPDATE ... WHERE status='open'` (§10c) → satu sukses, yang lain **409 `CONFLICT_ALREADY_DECIDED`** |
| Dua admin mem-blocklist target sama | `UNIQUE(target_type, target_id)` → satu sukses, yang lain **409 `CONFLICT_ALREADY_BLOCKED`** |
| Hide target lalu ada report baru untuk target sama | Report baru tetap `open`; hide bersifat pada target (bukan report) — keputusan lama tidak dibatalkan |
| Set `verified=true` bersamaan dengan `verified=false` | Last-write-wins pada kolom boolean; audit mencatat **keduanya** (`admin_audit` append-only) |
| Step-up signature dipakai dua kali | Nonce step-up sekali pakai (konsumsi atomik) → percobaan kedua **401 `ADMIN_STEPUP_INVALID`** |
| Admin A menonaktifkan admin B saat B beraksi | Cek allowlist **di setiap request** (bukan cache sesi panjang) → aksi B setelah pencabutan **403 `FORBIDDEN_ADMIN`** |

- **Semua** aksi destruktif admin = conditional update + audit append-only + step-up nonce sekali pakai; tidak ada jalur "read lalu write" tanpa kondisi.
- Audit ditulis dalam **transaksi yang sama** dengan perubahan state → tidak ada keputusan tanpa jejak, tidak ada jejak tanpa keputusan.
- Rate limit admin (30/menit/akun) mencegah dua admin saling menimpa dalam loop cepat, tetapi **bukan** pengganti conditional update.

## 15. Race silang offer ↔ bundle (di luar INV-028)

> INV-028 melarang token dalam bundle **aktif** di-list/di-offer terpisah. Di luar itu masih ada interleaving yang harus dijawab eksplisit. Prinsip tetap sama: **urutan eksekusi on-chain menentukan pemenang**; yang kalah revert tanpa kehilangan dana.

| Interleaving | Yang terjadi | Dijamin oleh |
|---|---|---|
| `create_bundle` (menyerap token A) ∥ `make_offer` (token A) | Keduanya receipt terpisah pada kontrak market yang sama → **diserialisasi**; yang jalan kedua melihat state terbaru dan **revert** (A sudah di bundle aktif) | INV-028, INV-007 (serialisasi shard) |
| `make_offer` (A) lalu `create_bundle` (A) | Offer aktif membuat A tidak boleh masuk bundle aktif → `create_bundle` **revert** (atau offer harus di-cancel dulu) | INV-028 + pre-validasi INV-025 |
| `accept_offer` (A) ∥ `create_bundle` (A) | Yang kedua revert: bila bundle menang, accept gagal (A bukan milik seller bebas); bila accept menang, bundle gagal pre-validasi ownership | INV-025, INV-016 |
| `buy_bundle` ∥ `cancel_offer` (token di bundle) | Tidak bertabrakan — cancel hanya menyentuh escrow offer; `buy_bundle` memvalidasi ownership/approval saat itu | INV-025, INV-016 |
| `buy_bundle` ∥ `buy` (token di bundle) | Token bundle tidak boleh punya listing terpisah (INV-028) → salah satu tidak mungkin ada; bila state basi → pre-validasi abort | INV-028, INV-025 |
| `buy_bundle` ∥ `buy_bundle` (bundle sama) | Yang pertama settle; yang kedua melihat bundle sudah tidak aktif → revert | INV-025 (status bundle) |
| Offer auto-superseded ∥ offer baru dibuat | Entry offer dihapus/`superseded` atomik; offer baru punya key sendiri | INV-009, INV-024 |

- **Pre-validasi INV-025 adalah penjaga utama**: semua item bundle divalidasi (ownership + approval + simulasi payout) **sebelum transfer pertama**; kegagalan → abort bersih + refund penuh, nol transfer.
- **Kegagalan residual** (mis. ownership berubah tepat di tengah loop — sangat jarang karena satu shard) → status `PARTIAL` + event `market_bundle_partial` + refund dana belum terpakai + kompensasi G7 (INV-025). Ini satu-satunya jalur yang tidak atomik, dan **dicatat on-chain**.
- Bila `create_bundle` menyerap token ber-offer, perilaku final (tolak vs auto-cancel offer) — ⏳ **open-by-design**; yang mengikat sekarang: tidak boleh ada state di mana token punya offer aktif **dan** bundle aktif (INV-028) → salah satu operasi revert.
- **Tidak ada** lock off-chain untuk kasus ini — semuanya diselesaikan chain (§8).

## 16. Status

- Analisis & jaminan chain (§1–§9) — **DECIDED (ronde 14)**, berbasis fakta NEAR + pola optimistic removal.
- Race API/DB, locking indexer, idempotency-key, retry RPC, race admin, race offer↔bundle (§10–§15) — **DECIDED (ronde 15)**.
- Optimistic locking profil (`If-Match`/412) & perilaku `create_bundle` vs offer aktif — **PROPOSED / ⏳ open-by-design** (dikunci saat implementasi; INV-028 tetap mengikat).
- Test concurrency — **PROPOSED** (dibuat bersama test sandbox, Fase 1); test race API/DB menyertai endpoint (Fase 2).
