# Disaster Recovery

> Diperluas dengan dimensi keamanan (pre-implementation research): pemulihan kunci/secrets, restore testing, dan ketergantungan chain.

## Skenario & rencana (draft)

| Skenario | Dampak | Rencana |
|---|---|---|
| Indexer/DB down/hilang | discovery & feed mati; transaksi chain TETAP AMAN | rebuild dari event chain (backfill dari block awal); frontend fallback ke RPC view langsung |
| RPC provider down | frontend gagal query | fallback chain terkunci: **FASTNEAR → official (rpc.near.org) → dRPC** + health check banding block hash |
| Bug kontrak kritis | dana/aset berisiko | **Pausable (ronde 13)**: owner pause → tx baru berhenti, penarikan escrow tetap terbuka; perbaikan via redeploy |
| IPFS pinning hilang | gambar 404 | re-pin dari sumber, multi-gateway |
| VPS mati / DB hilang | report & feed mati; **dana chain tetap aman** | restore backup PostgreSQL harian + re-provision via Docker/IaC; backup = cron `pg_dump` → object storage terpisah dari VPS |
| Key owner bocor | kontrol kontrak dicuri | MVP: satu owner key (simpan di VPS `~/.near-keys/` terenkripsi, seed offline 2 lokasi, tidak di git); **mainnet: Sputnik DAO V2 council 2-of-3 + timelock (DECIDED — ADR-013)** — recovery council member via proposal DAO |

## Prinsip

- Kebenaran dana/aset ada di chain — semua sistem off-chain harus bisa dibangun ulang dari chain.

## Security-critical recovery (pre-implementation research)

| Dimensi | Target | Status |
|---|---|---|
| RPO | ≤ 24 jam (backup harian) untuk data app; chain data = tidak relevan (selalu ada) | DECIDED |
| RTO | **< 4 jam** (MVP manual re-provision; mainnet otomatis — selaras database-security §11 "RTO < 4 jam") | DECIDED |
| Blockchain dependency | Chain tidak bisa "dipulihkan" oleh kita — recovery selalu di sisi off-chain/kontrak baru | FACT |
| Database recovery | restore drill wajib terdokumentasi (SEC-DB-004) — backup dianggap invalid sampai restore diuji | PROPOSED |
| Index rebuild | rebuild penuh dari events_raw (fase 2) — prosedur di indexer-security.md | PROPOSED |
| RPC failover | fallback chain terkunci: **FASTNEAR → official (rpc.near.org) → dRPC** + health check | DECIDED |
| Secrets recovery | seed offline 2 lokasi (key-management.md); rotasi semua secret dalam playbook IR | DECIDED interim |
| Signer/owner recovery | mainnet: council Sputnik DAO 2-of-3 recovery via proposal; **catatan**: redeploy ke akun BARU tidak memindahkan state/NFT referensi lama — opsi testnet saja | PROPOSED |
| Infrastructure recreation | Docker Compose + IaC script — VPS baru jalan < 4 jam | PROPOSED |

- **Aturan**: backup tidak dianggap valid sampai restore didesain & diuji.

## Jadwal & retensi backup

| Backup | Frekuensi | Retensi | Tempat | Tool |
|---|---|---|---|---|
| `pg_dump` penuh | harian (mis. 02:00 UTC) | 7 harian | object storage (S3-compatible) | `pg_dump` + container `backup` |
| `pg_dump` mingguan | mingguan | 4 mingguan | object storage | rotasi |
| `pg_dump` bulanan | bulanan | 3 bulanan | object storage | rotasi |
| WAL / point-in-time (opsional) | kontinu | 7 hari | object storage | pgBackRest/WAL-G (fase lanjut) |
| Snapshot `events_raw` (fase 2) | sebelum rebuild | 1 snapshot | object storage | `pg_dump` tabel |
| Konfigurasi (Caddyfile, compose, `.env` non-secret) | per perubahan | via git | repo | git |

- Object storage **terpisah dari VPS** (SEC-DB-001); backup tidak pernah disimpan hanya di VPS yang sama.
- Backup terenkripsi saat transit & saat diam (SSE provider); kredensial di env (SECRET).
- **Backup dianggap tidak valid sampai restore diuji** (SEC-DB-004).

```bash
# Cron harian di VPS (container `backup`).
docker compose exec -T db pg_dump -U "$POSTGRES_USER" -Fc -d "$POSTGRES_DB" \
  > /backup/nearsea_$(date -u +%Y%m%dT%H%M%SZ).dump
sha256sum /backup/nearsea_*.dump > /backup/nearsea_*.dump.sha256

# Upload ke object storage (contoh aws-cli; rclone juga bisa).
aws s3 cp /backup/ s3://"$BACKUP_BUCKET"/ --recursive \
  --endpoint-url "$BACKUP_S3_ENDPOINT"
```

## Runbook restore (database)

> **Prasyarat**: VPS/DB baru sudah ter-provision (TASK-028) + akses object storage + `DATABASE_URL` tujuan.

```bash
# 1. Ambil backup terbaru yang terverifikasi.
aws s3 ls s3://"$BACKUP_BUCKET"/ --endpoint-url "$BACKUP_S3_ENDPOINT" | sort | tail
aws s3 cp s3://"$BACKUP_BUCKET"/nearsea_<timestamp>.dump /restore/ \
  --endpoint-url "$BACKUP_S3_ENDPOINT"

# 2. Hentikan app agar tidak ada tulis selama restore.
cd /opt/nearsea && docker compose stop web

# 3. Restore ke DB (buat DB bila perlu).
docker compose exec -T db createdb -U "$POSTGRES_USER" "$POSTGRES_DB" || true
docker compose exec -T db pg_restore -U "$POSTGRES_USER" -d "$POSTGRES_DB" \
  --clean --if-exists /restore/nearsea_<timestamp>.dump

# 4. Verifikasi schema + migration state.
docker compose run --rm web npx prisma migrate status

# 5. Nyalakan app + smoke test.
docker compose up -d web
curl -fsS "https://<domain>/api/health"
```

- Restore **hanya** memulihkan data off-chain (`profiles`, `reports`, `blocklist`, `admin_audit`, `auth_nonce`, `sessions`).
  Data chain tidak dipulihkan (selalu ada di chain).
- Jika `events_raw` hilang (fase 2) → rebuild dari chain (backfill), **bukan** dari backup.
- Restore ke env mainnet = aksi berdampak → butuh approval user (AGENTS.md) + backup pra-restore.

## Verifikasi integritas backup

| Cek | Metode | Frekuensi |
|---|---|---|
| File ada & ukuran wajar | `aws s3 ls` + banding ukuran dump sebelumnya | harian (otomatis) |
| Checksum | `sha256sum` saat buat; verifikasi saat unduh | harian |
| Dump bisa dibaca | `pg_restore --list <dump>` sukses | harian |
| Restore penuh ke DB uji | restore ke DB sementara + banding jumlah baris tabel kunci | per kuartal (drill) |
| Retensi berjalan | file melewati retensi terhapus | mingguan |

```bash
# Verifikasi dump tanpa restore penuh.
pg_restore --list /restore/nearsea_<timestamp>.dump > /dev/null && echo "OK: dump valid"
sha256sum -c /restore/nearsea_<timestamp>.dump.sha256
```

- Backup gagal/tidak lolos verifikasi → alert Telegram (`backup_failed`, monitoring.md) + jalankan manual.

## Drill

| Drill | Isi | Kadensi | Owner | Bukti |
|---|---|---|---|---|
| Restore DB | restore backup ke DB uji, verifikasi schema + data | per kuartal (dan sebelum mainnet) | PLATFORM_OWNER (MVP) | catatan drill + `prisma migrate status` |
| Pause/unpause kontrak | guardian pause → tx mutasi ditolak → unpause | sebelum mainnet (SEC-IR-001) | SECURITY | test-cases |
| Re-provision VPS | bangun ulang dari Docker Compose + restore | per semester | PLATFORM_OWNER | log runbook |
| Rotasi secret | rotasi JWT/DB/SSH/Telegram | tahunan / saat anggota keluar | PLATFORM_OWNER | log rotasi |

- Drill restore = syarat menutup **SEC-DB-004** (status BLOCKED sampai infra TASK-030 ada).
- Hasil drill dicatat di [incident-response.md](../security/incident-response.md) + update status SEC-*.

## RPO/RTO per kelas data

| Kelas data | Contoh | RPO | RTO | Sumber pemulihan |
|---|---|---|---|---|
| Chain (ownership/dana) | NFT, listing, escrow, fee | 0 (final on-chain) | tidak relevan | chain (tidak dipulihkan oleh kita) |
| App-mutable | `profiles`, `reports`, `blocklist`, `admin_audit` | ≤ 24 jam (backup harian) | < 4 jam (restore) | backup `pg_dump` |
| Auth/session | `auth_nonce`, `sessions` | ≤ 24 jam | < 4 jam | backup `pg_dump` (session boleh hilang → login ulang) |
| Chain-derived (fase 2) | proyeksi listing/floor/volume, `events_raw` | 0 (dari chain) | < 1 hari (rebuild) | rebuild dari chain / `events_raw` |
| Secrets | JWT, DB, bot, SSH, backup key | 0 (rotasi) | < 1 jam | secret store + rotasi |
| Owner key | owner / DAO council | 0 | jam (DAO recovery) | seed offline 2 lokasi / proposal DAO |

- Angka MVP **DECIDED**: RPO ≤ 24 jam / RTO < 4 jam (lihat tabel security-critical recovery di atas).
- Kehilangan `sessions` hanya memaksa login ulang — bukan kehilangan data.

## Skenario: semua provider RPC gagal

```text
TRIGGER : FASTNEAR + official + dRPC sama-sama gagal/timeout (rate limit global, outage).
DAMPAK  : FE tidak bisa query/mengirim tx; chain tetap aman (settlement tidak terpengaruh).
LANGKAH :
1. Konfirmasi kegagalan dari health check (banding block hash) — bukan bug FE.
2. Tampilkan banner "jaringan sedang sibuk" + nonaktifkan aksi tulis (read fallback sudah dicoba).
3. Coba provider cadangan berbayar bila tersedia (QuickNode/Tatum/ZAN — bila kunci dikonfigurasi).
4. Bila semua gagal: mode read-only terbatas; tx user akan gagal terkirim (bukan salah settle).
5. Pantau status provider resmi; pulihkan otomatis saat provider sehat.
6. Catat insiden (S3) + evaluasi menambah provider/kunci berbayar.
```

- Ini insiden **ketersediaan**, bukan kehilangan dana: tidak ada rollback settlement.
- Pencegahan: urutan fallback terkunci + health check block hash (indexer-security §2); opsi provider berbayar (infrastructure.md).

## Playbook: kompromi secret (detail)

> Melengkapi [secrets-and-gitignore.md §6](../development/secrets-and-gitignore.md) + [incident-response.md](../security/incident-response.md).

```text
1. IDENTIFIKASI
   - Secret apa, di mana terpapar (repo/CI/log/chat/screenshot), sejak kapan.
   - Kelas: JWT / DB / Telegram / SSH deploy / backup key / owner key.

2. CONTAIN (dulu, cepat)
   - Cabut kredensial: revoke SSH/deploy key, revoke token bot, ganti password DB.
   - Owner key (bila bocor) → pause via guardian (mainnet) / pause owner (testnet).

3. ROTASI (anggap bocor, bukan sekadar hapus commit)
   - JWT_SECRET      : generate baru → invalidate semua sesi (user login ulang).
   - DATABASE_URL    : buat role/password baru → update .env.server → restart web.
   - TELEGRAM_BOT_TOKEN : /revoke di BotFather → token baru.
   - DEPLOY_SSH_KEY  : buat keypair baru → pasang di server → hapus GitHub Secret lama.
   - BACKUP_S3_*     : rotasi access key di provider object storage.
   - Owner key       : prosedur key-management §3a (proposal DAO remove_member → add fresh key).

4. VERIFIKASI
   - Secret lama tidak lagi valid (uji akses harus gagal).
   - Aplikasi tetap sehat (/api/health 200) setelah rotasi.
   - Git history dibersihkan (git filter-repo) SETELAH rotasi (bukan pengganti rotasi).

5. FORENSIK & POSTMORTEM
   - Simpan log/tx hash/bukti SEBELUM dibersihkan.
   - Catat di incident-response.md; perbarui status SEC-*; perbaiki akar penyebab.
```

- Aturan: **rotasi dulu, hapus history belakangan**; menghapus commit tidak membatalkan kebocoran.

## Kontak eskalasi

| Peran | Tanggung jawab | Kontak |
|---|---|---|
| PLATFORM_OWNER | keputusan S1, akses VPS/DB, rotasi | ⏳ open-by-design (diisi saat SEC-IR-002) |
| SECURITY | pause kontrak, investigasi insiden | ⏳ open-by-design |
| INFRA | re-provision VPS, DNS/SSL | ⏳ open-by-design |
| Provider RPC | laporan outage | kanal support provider |
| Registrar domain | hijack/DNS (CS-8) | panel registrar + kontak darurat |
| Auditor (TASK-027) | verifikasi pasca-insiden kontrak | kontak vendor audit |

- Kontak darurat + akses kedua wajib ada sebelum mainnet (**SEC-IR-002**); dokumen "siapa boleh apa saat S1" = access-control-matrix.
- MVP = 1 orang: siapkan akses kedua (VPS/DM) agar insiden tidak terkunci pada satu pihak.

## Status

- Backup harian → object storage — **PROPOSED** (TASK-030, SEC-DB-001).
- Restore drill — **BLOCKED** (SEC-DB-004) sampai infra VPS/DB ada (TASK-028).
- RPO ≤ 24 jam / RTO < 4 jam (MVP) — **DECIDED**.
- Playbook secret & RPC — **DECIDED interim** (dibuktikan saat drill).
