# Issue tracker — di mana pekerjaan dilacak di repo ini

> Dibaca oleh `/to-tickets`, `/triage`, `/to-spec`, `/wayfinder`.
> Dibuat ronde 17c; diperbarui ronde 28 (remote GitHub + aturan anti-divergensi status).

Repo ini memakai **tracker markdown lokal** sebagai SSOT (`tasks/` + `.scratch/`). Repo **sudah**
punya remote GitHub ([`github.com/keydeveloping/nearsea`](https://github.com/keydeveloping/nearsea)),
jadi `gh` **dipakai** untuk PR & CI. Tapi **tiket tetap markdown di repo** — tidak ada GitHub Issues,
dan tidak ada link blocking native; `Blocked by:` tetap baris teks yang dibaca manusia/agent.

## Otoritas status: branch `dev`

> **Pelajaran ronde 28 — `.scratch/` di-track git, jadi status tiket bisa BERBEDA antar branch.**

`.scratch/m1-slice/issues/*.md` ikut ter-commit. Kalau tiket di-update `done` di branch fitur yang
**belum di-merge**, maka:

- orang yang membaca tiket dari `dev` melihat status **lama** (`ready-for-agent`), dan
- orang yang membaca dari branch fitur melihat status **baru** (`done`) —

dua jawaban berbeda untuk fakta yang sama. Ini persis pelanggaran SSOT yang dilarang
[AGENTS.md](../../AGENTS.md). Ronde 28 menemukan tiket 05–08 `done` di branch kontrak tapi
`ready-for-agent` di `dev` **dan** di branch FE.

**Aturan:**

1. **`dev` adalah otoritas status tiket.** Sebuah tiket dianggap `done` hanya kalau perubahan
   statusnya sudah ada di `dev` — bukan di branch fitur.
2. **Update status tiket = bagian dari PR yang mendaratkan kerjanya.** Jangan menandai `done`
   di branch lalu meninggalkannya belum di-merge berhari-hari; merge-kan atau jangan tandai.
3. **Jangan mengedit file tiket dari dua rantai branch sekaligus.** Kalau dua branch menyentuh
   file tiket yang sama, merge berikutnya akan menabrakkan status — rekonsiliasi **setelah**
   semua rantai mendarat, di satu PR khusus.
4. **AC "berjalan di CI dan hijau" wajib menyertakan URL run CI.** Branch yang belum pernah
   di-push belum pernah diuji CI; menandai AC itu `[x]` tanpa run URL = asersi, bukan bukti.

## Dua permukaan, satu SSOT

| Permukaan | Path | Untuk apa |
|---|---|---|
| **Backlog kanonik** | [`tasks/backlog.md`](../../tasks/backlog.md) | SSOT semua task produk. ID `TASK-NNN`, kolom Prioritas / Depends / Spec / Status / Estimate / Done-when / Milestone |
| **Milestone** | [`tasks/milestones.md`](../../tasks/milestones.md) | SSOT definisi & exit criteria milestone (`M0`, `M1`, `M1+`, `M2`, `M3`, `M4`) |
| **Tiket per-fitur** | `.scratch/<fitur>/issues/<NN>-<slug>.md` | Dihasilkan `/to-tickets`; satu file per tiket, tidak pernah satu file gabungan |

**Aturan:** kalau `/to-tickets` membuat tiket per-fitur di `.scratch/`, task yang **menutup** pekerjaan tetap harus terdaftar di `tasks/backlog.md` (kolom `Spec` menunjuk ke `.scratch/` atau ke `docs/features/*`). Jangan biarkan backlog dan `.scratch/` menyimpang — `tasks/backlog.md` yang menang.

## Konvensi `.scratch/`

- Satu fitur per direktori: `.scratch/<fitur-slug>/`
- Spec: `.scratch/<fitur-slug>/spec.md`
- Tiket: `.scratch/<fitur-slug>/issues/<NN>-<slug>.md`, bernomor dari `01`
- `Status:` di dekat atas file (lihat `triage-labels.md`)
- `Blocked by: NN, NN` di dekat atas file bila punya blocker; tiket bebas saat semua blocker `resolved`
- Riwayat percakapan ditambahkan di bawah heading `## Comments`

## Saat skill berkata "publish to the issue tracker"

- Kalau tiketnya **per-fitur hasil `/to-tickets`** → buat file di `.scratch/<fitur-slug>/issues/`.
- Kalau tiketnya **task produk tingkat milestone** → tambahkan baris di `tasks/backlog.md` dengan ID `TASK-NNN` berikutnya, dan lengkapi kolom `Milestone` + `Done-when`.

## Saat skill berkata "fetch the relevant ticket"

Baca file di path yang dirujuk. User biasanya memberikan path atau ID (`TASK-012`) langsung — untuk ID `TASK-*`, cari di `tasks/backlog.md`.

## Operasi wayfinding

Dipakai `/wayfinder`. **Map** = satu file, satu **child** file per tiket.

- **Map**: `.scratch/<effort>/map.md` (isi: Notes / Decisions-so-far / Fog).
- **Child ticket**: `.scratch/<effort>/issues/NN-<slug>.md`, bernomor dari `01`, pertanyaan di body. Baris `Type:` mencatat tipe (`research`/`prototype`/`grilling`/`task`); baris `Status:` mencatat `claimed`/`resolved`.
- **Blocking**: baris `Blocked by: NN, NN` di dekat atas.
- **Frontier**: file di `.scratch/<effort>/issues/` yang terbuka, tidak terblokir, belum diklaim; nomor terkecil menang.
- **Claim**: set `Status: claimed` dan simpan **sebelum** kerja apa pun.
- **Resolve**: tambahkan jawaban di bawah heading `## Answer`, set `Status: resolved`, lalu tambahkan pointer konteks (gist + tautan) ke Decisions-so-far di `map.md`.

## Setelah map selesai

`/wayfinder` **tidak membangun** — ia menyerahkan ke `/to-spec`, lalu `/to-tickets`, lalu `/implement`. Hasil keputusan yang material (arsitektur/produk) **wajib** dicatat sebagai ADR di `docs/decisions/` (aturan [AGENTS.md](../../AGENTS.md) "Setiap keputusan produk/arsitektur baru → catat di `docs/decisions/ADR-*.md`").
