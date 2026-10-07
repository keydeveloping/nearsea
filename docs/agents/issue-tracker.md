# Issue tracker — di mana pekerjaan dilacak di repo ini

> Dibaca oleh `/to-tickets`, `/triage`, `/to-spec`, `/wayfinder`.
> Dibuat ronde 17c.

Repo ini memakai **tracker markdown lokal**. Tidak ada GitHub/GitLab remote saat ini
(repo belum di-`git init`), jadi `gh`/`glab` **tidak dipakai**.

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
