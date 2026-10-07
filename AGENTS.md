# AGENTS.md

> Aturan kerja untuk AI agent yang mengerjakan proyek ini.
> Source of truth: `docs/`. Riset teknis NEAR: `RESEARCH.md`.

## Project Rules

- Bahasa kontrak pintar: **Rust** (`near-sdk`) — standar NEP-171/177/178/181/199/297.
- Setiap keputusan produk/arsitektur baru → catat di `docs/decisions/ADR-*.md`.
- Setiap fitur baru wajib punya spesifikasi di `docs/features/<fitur>.md` SEBELUM dikode.
- **Wajib baca & penuhi `docs/security/security-requirements.md`**: setiap task wajib memenuhi SEMUA requirement berstatus P0 yang READY-FOR-IMPLEMENTATION atau DECIDED yang relevan dengannya — requirement itu bagian dari definisi "selesai" task (kontrak/API/FE).
- Perubahan schema database wajib lewat migration, tidak pernah edit table langsung.
- Kontrak NFT dan kontrak market adalah dua kontrak terpisah (pola NEAR — lihat RESEARCH.md bagian 1).
- Setiap perubahan pada argumen method kontrak = perubahan API → update `docs/` + tests bersamaan.

## Konvensi Repo (baca dulu sebelum membuat file baru)

> Repo ini punya struktur **sengaja** yang lebih lengkap dari konvensi default tooling. Kalau sebuah
> skill/panduan menyebut path default yang berbeda, **ikuti konvensi repo ini** — jangan membuat
> struktur paralel.

| Kalau panduan/skill menyebut… | Di repo ini, gunakan… |
|---|---|
| `CONTEXT.md` (glosarium domain) | **[CONTEXT.md](./CONTEXT.md)** — sudah ada di root; **baca sebelum menamai konsep baru**, dan update saat istilah ambigu diselesaikan |
| `docs/adr/NNNN-slug.md` | **`docs/decisions/ADR-NNN-slug.md`** — **jangan** buat folder `docs/adr/` (rumah ADR kedua = kekacauan) |
| indeks/peta dokumentasi | **[docs/DOCUMENTATION-MAP.md](./docs/DOCUMENTATION-MAP.md)** — SSOT sinkronisasi |
| task/tiket | **`tasks/backlog.md`** (SSOT prioritas) + `tasks/milestones.md` (SSOT milestone) |
| spesifikasi fitur | `docs/features/<fitur>.md` |
| reference implementasi kontrak | `docs/contracts/<kontrak>.md` |
| konfigurasi skill engineering | **`docs/agents/`** (`domain.md`, `issue-tracker.md`, `triage-labels.md`) — diisi sekali, bukan tempat spesifikasi |

- **Jangan menambah struktur baru** (folder/indeks/register) tanpa alasan yang tidak bisa diselesaikan
  struktur yang ada. Proyek ini sudah ratusan file dokumen; menambah lapisan baru punya biaya nyata.
- Saat membuat file baru, ikuti aturan **File & Secret Hygiene** di bawah + daftarkan di
  DOCUMENTATION-MAP §4 agar tidak jadi file yatim.

## Agent skills

> Blok ini dibaca skill engineering (Matt Pocock: `/grill-with-docs`, `/to-spec`, `/to-tickets`,
> `/triage`, `/wayfinder`, `/domain-modeling`). Fungsinya: memberi tahu skill **di mana** hal-hal
> berada, supaya mereka tidak perlu restrukturisasi repo. Diperbarui ronde 17c.

### Issue tracker

Tracker markdown lokal — SSOT `tasks/backlog.md` (task produk, `TASK-NNN`) + `tasks/milestones.md`;
tiket per-fitur hasil `/to-tickets` di `.scratch/<fitur>/issues/`. Tidak ada remote GitHub/GitLab,
jadi `gh`/`glab` tidak dipakai. Lihat [docs/agents/issue-tracker.md](./docs/agents/issue-tracker.md).

### Triage labels

Lima peran kanonik, nama label = nama peran. Kolom `Status` di `tasks/backlog.md` berarti **progres**
(`todo`/`in-progress`/`done`), bukan triage — triage hanya untuk issue dari luar. Lihat
[docs/agents/triage-labels.md](./docs/agents/triage-labels.md).

### Domain docs

**single-context** — satu [CONTEXT.md](./CONTEXT.md) di root. Rumah ADR repo ini adalah
**`docs/decisions/ADR-NNN-slug.md`** (bukan `docs/adr/`). Lihat
[docs/agents/domain.md](./docs/agents/domain.md) dan [docs/DOCUMENTATION-MAP.md](./docs/DOCUMENTATION-MAP.md).

## Documentation Sync Rules (WAJIB — berlaku untuk SETIAP perubahan)

Peta lengkap "perubahan apa → dokumen mana yang wajib dicek" ada di:
**[docs/DOCUMENTATION-MAP.md](./docs/DOCUMENTATION-MAP.md)**

Aturan inti:

1. Sebelum menyatakan selesai, jalankan **checklist crosscheck** di DOCUMENTATION-MAP.md.
2. Perubahan keputusan (produk/arsitektur) → update ADR terkait + SEMUA dokumen yang mengutip keputusan itu.
3. Jika ada istilah/angka yang berubah (nama produk, fee, batas royalti, dst.) → **grep istilah lama di seluruh repo**; dilarang menyisakan nilai lama di dokumen mana pun.
4. Update bagian **Status** di `docs/00-project-overview.md`.
5. **Single source of truth**: setiap fakta hanya "dimiliki" satu dokumen (lihat tabel di DOCUMENTATION-MAP.md); dokumen lain hanya boleh MERUJUK, bukan menyatakan ulang nilai berbeda.
6. Jika perubahan menghasilkan pekerjaan baru → tambah task di `tasks/backlog.md`; jika menutup pekerjaan → update status task/milestone.

## Behavioral Guidelines (cara kerja agent)

> Panduan perilaku untuk mengurangi kesalahan umum LLM saat menulis kode. Jika berbenturan dengan aturan spesifik proyek di atas, **aturan proyek yang menang**.

**Tradeoff:** panduan ini condong ke hati-hati daripada cepat. Untuk tugas sepele, gunakan pertimbangan.

### 1. Think Before Coding

**Jangan berasumsi. Jangan sembunyikan kebingungan. Tampilkan tradeoff-nya.**

Sebelum implementasi:
- Nyatakan asumsimu secara eksplisit. Jika tidak yakin, tanya.
- Jika ada beberapa interpretasi, ajukan semuanya — jangan pilih diam-diam.
- Jika ada pendekatan yang lebih sederhana, katakan. Bantah bila memang perlu.
- Jika ada yang tidak jelas, berhenti. Sebutkan apa yang membingungkan. Tanya.

### 2. Simplicity First

**Kode minimum yang menyelesaikan masalah. Tidak ada yang spekulatif.**

- Tidak ada fitur di luar yang diminta.
- Tidak ada abstraksi untuk kode sekali pakai.
- Tidak ada "fleksibilitas"/"konfigurabilitas" yang tidak diminta.
- Tidak ada penanganan error untuk skenario yang mustahil.
- Jika kamu menulis 200 baris padahal bisa 50, tulis ulang.

Tanya dirimu: "Apakah senior engineer akan bilang ini overcomplicated?" Jika ya, sederhanakan.

### 3. Surgical Changes

**Sentuh hanya yang wajib. Bersihkan hanya kekacauanmu sendiri.**

Saat mengedit kode yang ada:
- Jangan "memperbaiki" kode, komentar, atau format di sekitarnya.
- Jangan refactor hal yang tidak rusak.
- Ikuti gaya yang ada, meski kamu akan melakukannya berbeda.
- Jika menemukan dead code yang tidak terkait, sebutkan — jangan hapus.

Saat perubahanmu menciptakan orphan:
- Hapus import/variabel/fungsi yang menjadi tidak terpakai **akibat perubahanmu**.
- Jangan hapus dead code yang sudah ada sebelumnya kecuali diminta.

Ujiannya: setiap baris yang berubah harus bisa dilacak langsung ke permintaan user.

### 4. Goal-Driven Execution

**Definisikan kriteria sukses. Ulangi sampai terverifikasi.**

Ubah tugas menjadi tujuan yang bisa diverifikasi:
- "Tambahkan validasi" → "Tulis test untuk input tidak valid, lalu buat lolos"
- "Perbaiki bug" → "Tulis test yang mereproduksinya, lalu buat lolos"
- "Refactor X" → "Pastikan test lolos sebelum dan sesudah"

Untuk tugas multi-langkah, sebutkan rencana singkat:
```
1. [Langkah] → verify: [cek]
2. [Langkah] → verify: [cek]
3. [Langkah] → verify: [cek]
```

Kriteria sukses yang kuat membuatmu bisa beriterasi mandiri. Kriteria lemah ("buat agar jalan") butuh klarifikasi terus-menerus.

> Selaras dengan Definition of Done proyek: [docs/testing/testing-strategy.md](./docs/testing/testing-strategy.md).

---

**Panduan ini berhasil jika:** perubahan yang tidak perlu di diff berkurang, rewrite akibat overcomplication berkurang, dan pertanyaan klarifikasi muncul **sebelum** implementasi, bukan setelah kesalahan.

## Coding Style (ronde 12 + 14)

Detail lengkap: **[docs/development/code-standards.md](./docs/development/code-standards.md)**.

- Rust: `cargo fmt` + `cargo clippy` tanpa warning; konvensi penamaan standar Rust.
- TypeScript: strict mode, **tanpa `any`** kecuali diberi alasan eksplisit di komentar; `eslint` + `prettier`.
- Copy UI bahasa Inggris, semua string lewat `i18n/` (tidak ada hardcode di komponen).
- **Komentar: hanya di baris yang benar-benar penting.** Komentari *mengapa* (keputusan non-obvious, batasan keamanan, konstanta sihir), bukan *apa* (kode yang sudah jelas). Dilarang menuliskan riwayat perubahan di komentar. `TODO` hanya boleh ada bila menunjuk task di `tasks/backlog.md`.
- Error & notifikasi mengikuti katalog terpusat: [docs/development/error-handling.md](./docs/development/error-handling.md) (satu error = satu kode + satu pesan; tanpa stack trace ke client).

## Blockchain Rules (NEAR)

- Transfer NFT selalu `assert_one_yocto()` + cek ownership.
- Saldo/nilai selalu **string yoctoNEAR (u128)** di JSON — dilarang pakai `number`.
- Callback cross-contract selalu `#[private]`; otorisasi pakai `predecessor_account_id`.
- Royalti payout maks 10 penerima; validasi total payout ≤ harga (sisa ≤ 1 yocto).
- User membayar storage sendiri (pola NEP-145) — kontrak tidak boleh menanggung storage user.
- Media aset di IPFS; on-chain hanya URL + hash. Dilarang menyimpan gambar on-chain (batas 4 MB/call, 1 Ⓝ/100 KB).

## Server-Side Fetch & SSRF (WAJIB)

> Berlaku setiap kali **server/backend/worker** (bukan browser) mem-fetch URL — mis. metadata fetcher fase 2, indexer, proxy media.
> Spesifikasi lengkap (limit redirect/timeout/size/MIME, isolasi proses, cache): **[docs/security/metadata-security.md](./docs/security/metadata-security.md) §3**.

- **Protokol hanya http/https.** Proyek lebih ketat: **https saja** untuk fetch eksternal; `http://` (non-TLS) ditolak. `ipfs://` ditulis ulang ke gateway https allowlist.
- **Validasi host SEBELUM mengirim request.** Tolak `localhost`, loopback (127.0.0.0/8, `::1`), privat (10/8, 172.16/12, 192.168/16, fc00::/7), link-local (169.254/16, fe80::/10), dan reserved/CGNAT (100.64/10, 0.0.0.0). Praktiknya: **tolak semua yang bukan global-unicast**.
- **Anti DNS-rebinding:** resolve → validasi IP → koneksi ke IP yang sama (pin), bukan resolve ulang.
- Redirect ≤3 (re-validasi tiap hop); timeout 5 dtk; response ≤5 MB; dekompresi ≤25 MB; Content-Type allowlist.
- Fetcher berjalan **terisolasi** (proses/container terpisah, egress terbatas, tanpa kredensial DB utama).

## Git Rules (ronde 12 + 14)

Detail lengkap: **[docs/development/git-workflow.md](./docs/development/git-workflow.md)**.

- **Tiga branch permanen**: `mainnet` (produksi), `testnet` (rilis testnet), `dev` (integrasi developer). Default branch = `mainnet`.
- Alur promosi **satu arah**: `feat/<nama>` → PR → `dev` → PR → `testnet` → PR → `mainnet`. Tidak ada jalur pintas.
- **WAJIB tanya user dulu sebelum merge ke `testnet` atau `mainnet`** (dan sebelum deploy ke sana). Merge ke `dev` boleh tanpa tanya selama CI hijau. Tanpa jawaban user = jangan merge.
- Branch per fitur/task (`feat/offer-escrow`, `fix/refund-edge`); hapus setelah merge.
- **Conventional commits**: `feat:`, `fix:`, `docs:`, `test:`, `chore:`, `security:` …
- Semua branch target harus selalu buildable; rollback = revert merge commit (atau deploy ulang tag sebelumnya).
- Versi & rilis: [docs/development/versioning-and-release.md](./docs/development/versioning-and-release.md) (SemVer per-artefak; tag `contract-vX.Y.Z` / `web-vX.Y.Z`).
- Pipeline & gate CI: [docs/development/ci-cd.md](./docs/development/ci-cd.md).

## File & Secret Hygiene (ronde 14 — WAJIB)

Detail lengkap: **[docs/development/secrets-and-gitignore.md](./docs/development/secrets-and-gitignore.md)**.

- **Sebelum membuat/mengisi file baru**: tentukan dulu apakah isinya bisa sensitif. Jika ya → **tambahkan polanya ke `.gitignore` LEBIH DULU**, baru isi filenya.
- Kredensial **hanya** dibaca dari environment variable atau secret store. **Dilarang** menulis kredensial yang bisa dipakai (usable literal) di source, contoh (`.env.example`), maupun test — termasuk placeholder yang menyerupai kredensial asli.
- Tidak ada secret/private key/seed di kode, frontend, git, log, atau chat.
- `.env` asli tidak pernah masuk repo; `.env.example` hanya berisi placeholder.
- Jalankan secret scan (gitleaks) sebelum commit; CI menjalankannya otomatis di setiap PR.

## Testing Rules

- Setiap method mutasi kontrak wajib ada test sandbox (unit + cross-contract).
- **Semua endpoint API wajib punya test** — minimal happy path + jalur error (kode error sesuai katalog [docs/development/error-handling.md](./docs/development/error-handling.md)). Endpoint baru tanpa test = belum selesai.
- Fitur = selesai hanya jika AC-nya lolos (lihat docs/testing — definition of done).
- PR tanpa hasil test yang jelas (lolos/gagal) tidak boleh merge.

## Build & Test Commands (jalankan sebelum menyatakan selesai)

> Perintah ini = padanan gate CI lokal. Jalankan yang relevan dengan perubahanmu; jangan mengandalkan CI untuk menemukan masalah yang bisa dilihat lokal.

```bash
# --- Kontrak (Rust) ---
cargo fmt --all -- --check                 # format
cargo clippy --all-targets -- -D warnings  # lint (wajib 0 warning)
cargo test --workspace                     # unit + sandbox (near-workspaces)
cargo near build                           # build wasm (mode reproducible saat rilis)
cargo audit                                # CVE dependency

# --- Frontend / API (pnpm, TypeScript strict) ---
pnpm --dir frontend install --frozen-lockfile
pnpm --dir frontend lint
pnpm --dir frontend typecheck
pnpm --dir frontend test -- --run          # Vitest
pnpm --dir frontend build
pnpm audit --audit-level=high

# --- Database (jika menyentuh schema) ---
pnpm prisma migrate deploy                 # apply ke DB test
pnpm prisma migrate status                 # cek drift
#   up → down → up harus menghasilkan schema identik (migrations.md §6)

# --- Secret & E2E ---
gitleaks detect --config .gitleaks.toml --redact
pnpm --dir frontend exec playwright test   # E2E jalur emas (testnet; pra-rilis/nightly)
```

- Perubahan kontrak: **wajib** `fmt` + `clippy` + `test` hijau sebelum PR.
- Perubahan FE/API: **wajib** `lint` + `typecheck` + `test` hijau; endpoint baru wajib punya test happy + error.
- Perubahan schema: **wajib** migration + `down.sql` + gate DB hijau ([migrations.md](./docs/database/migrations.md) §6).
- Perintah lengkap & anggaran waktu: [docs/development/ci-cd.md](./docs/development/ci-cd.md) dan [docs/testing/testing-strategy.md](./docs/testing/testing-strategy.md).

## Code Review Checklist (untuk reviewer & self-review)

> Jalankan sebelum meminta review; PR yang gagal poin wajib akan dikembalikan.

**Wajib (blok merge bila gagal):**

- [ ] CI + Security hijau; tidak ada check yang di-skip tanpa alasan.
- [ ] Tidak ada kredensial usable di source/contoh/test; `.env` tidak ter-commit; pola file sensitif ada di `.gitignore`.
- [ ] Tidak ada `any` di TypeScript tanpa alasan eksplisit; tidak ada `number` untuk nilai yoctoNEAR.
- [ ] Semua method mutasi kontrak & endpoint API punya test (sukses + jalur gagal); ID TC/INV dirujuk.
- [ ] Panic kontrak dipetakan ke kode error terpusat — tidak ada pesan error ad-hoc ([error-handling.md](./docs/development/error-handling.md)).
- [ ] Requirement SEC-* P0 relevan terpenuhi; aksi destruktif punya step-up/otorisasi sesuai katalog.
- [ ] Perubahan API/kontrak/DB → dokumen di [DOCUMENTATION-MAP.md](./docs/DOCUMENTATION-MAP.md) sudah disinkronkan (termasuk grep istilah lama).
- [ ] Tidak ada TODO tanpa ID task di `tasks/backlog.md`.
- [ ] Perubahan `.github/workflows/**` → review khusus (jalur injeksi secret).

**Kualitas:**

- [ ] Komentar hanya di baris penting (why, bukan what); tidak ada riwayat perubahan di komentar.
- [ ] Kode mengikuti pola/modul yang sudah ada; tidak menciptakan pola paralel.
- [ ] Fungsi/file dalam batas ukuran wajar ([code-standards.md](./docs/development/code-standards.md) §12).
- [ ] Tidak ada refactor/perbaikan di luar scope permintaan (Surgical Changes).
- [ ] Rollback terpikirkan (kontrak: storage layout; web: tag sebelumnya).

## Research Tasks (cara menangani tugas riset)

> Riset dipakai saat ada **OPEN QUESTION / RESEARCH REQUIRED** di dokumen (mis. provider IPFS, integrasi DAO, endpoint NearBlocks). Tujuannya keputusan yang bisa dicatat, bukan eksplorasi tanpa akhir.

```text
1. TENTUKAN pertanyaan tunggal yang harus dijawab + kriteria keputusan (mis. "provider mana, dengan syarat X").
2. CARI sumber primer: dokumentasi resmi NEAR (docs.near.org, nomicon.io), repo resmi, standar NEP.
   Catat URL + tanggal akses. Prioritaskan fakta terverifikasi, bukan opini blog.
3. UJI bila memungkinkan (spike kecil di testnet/sandbox) — bukan sekadar membaca.
4. TULIS hasil di dokumen yang tepat:
   - Fakta teknis NEAR → RESEARCH.md (bagian terkait) + tanggal.
   - Keputusan → ADR baru (format rumah: Status/Problem/Context/Options/Trade-offs/Security/
     Scalability/Decision/Rejected/Consequences) atau update ADR terkait.
   - Item belum diputuskan → tandai ⏳ open-by-design, jangan mengarang keputusan.
5. PROPAGASI: update dokumen yang mengutip keputusan (DOCUMENTATION-MAP sync trigger) + backlog bila muncul task.
```

- Larangan: menulis fakta riset sebagai keputusan produk; mengubah business rules (fee/royalti) tanpa persetujuan user.
- Riset yang menyentuh keamanan → naikkan status SEC-* terkait (mis. PROPOSED → DECIDED/READY-FOR-IMPLEMENTATION).
- Selalu bedakan **FACT** (terverifikasi) vs **PROPOSED** (usulan) vs **⏳ open-by-design** (belum diputuskan).

## Escalation Path (kapan berhenti & bertanya)

> Agent wajib **berhenti dan bertanya** — bukan menebak — pada situasi berikut.

| Situasi | Ke mana | Aturan |
|---|---|---|
| Merge/deploy ke `testnet`/`mainnet`, atau transfer ownership kontrak | **User** (wajib, tunggu jawaban) | [git-workflow.md](./docs/development/git-workflow.md) §3 |
| Perubahan business rules (fee, royalti, batas) | **User** | Dilarang mengubah tanpa update docs + persetujuan |
| Ambiguitas produk/UX yang mengubah scope | **User** | Tanyakan sebelum koding (Think Before Coding) |
| Kontradiksi antar dokumen (SSOT bentrok) | **User** (jika menyangkut keputusan) | Perbaiki dokumen pemilik dulu; jangan pilih diam-diam |
| Keputusan arsitektur baru | **User** + catat ADR | Jangan implementasi sebelum ADR dicatat |
| Risiko keamanan P0 baru ditemukan | **User** + dokumen security | Naikkan status SEC-*, jangan tunda |
| Bug produksi / insiden | Ikuti [incident-response.md](./docs/security/incident-response.md); eskalasi ke **User** | Pause bila perlu (guardian) |
| Blokir teknis yang butuh akses/secret/akun | **User** (operator) | Jangan menebak kredensial |
| Keputusan sepele yang punya default jelas | **Tidak perlu bertanya** | Pilih default wajar, sebutkan di laporan |

- Format eskalasi: sebutkan **konteks**, **opsi**, **rekomendasi**, dan **dampak** bila salah — ringkas.
- Saat menunggu jawaban: hentikan pekerjaan yang bergantung pada keputusan itu; jangan lanjut dengan asumsi.
- Semua eskalasi yang menghasilkan keputusan → catat di ADR/dokumen terkait agar tidak terulang.

## Important Architecture Decisions

Lihat [docs/decisions/ADR-001.md](./docs/decisions/ADR-001.md) — log lengkap ADR-001..016
(termasuk: approval model ala OpenSea, launchpad berphase, fee 2%, infra VPS, data RPC+NearBlocks,
security: chain-centric boundary, NEP-413 auth, on-chain orderbook, key management Sputnik DAO, metadata isolation, indexer Neardata, intent proyek + scope cut M1).

> Catatan: "ronde N" di seluruh dokumen = nomor sesi tanya-jawab keputusan; log per ronde ada di ADR-001.md dan riwayat sinkronisasi di DOCUMENTATION-MAP.md.

## Things Agents Must NOT Do

- TIDAK boleh deploy ke mainnet tanpa persetujuan eksplisit user.
- TIDAK boleh merge PR ke `testnet`/`mainnet` (atau deploy ke sana) tanpa bertanya dulu ke user.
- TIDAK boleh menyimpan secret/private key di kode, frontend, atau git.
- TIDAK boleh menulis kredensial yang bisa dipakai di source/contoh/test.
- TIDAK boleh mengisi file sensitif sebelum polanya masuk `.gitignore`.
- TIDAK boleh mengubah business rules (fee, royalti) tanpa update `docs/` dulu.
- TIDAK boleh melewati checklist keamanan NEAR (RESEARCH.md bagian 3) pada method kontrak publik.
