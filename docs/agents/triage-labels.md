# Triage labels — pemetaan peran triage ke kosakata repo ini

> Dibaca oleh `/triage`. Dibuat ronde 17c.

Skill berbicara dalam lima peran triage kanonik. Tabel ini memetakan peran itu ke label yang dipakai tracker repo ini.

| Peran di skill | Label di tracker kita | Arti |
|---|---|---|
| `needs-triage` | `needs-triage` | Perlu dievaluasi pemilik proyek |
| `needs-info` | `needs-info` | Menunggu pelapor memberi informasi tambahan |
| `ready-for-agent` | `ready-for-agent` | Sudah terspesifikasi penuh, siap dikerjakan agent |
| `ready-for-human` | `ready-for-human` | Butuh implementasi manusia |
| `wontfix` | `wontfix` | Tidak akan dikerjakan |

## Cara memakai di tracker markdown lokal

Karena tidak ada remote GitHub/GitLab, label ditulis sebagai baris `Status:` di dekat atas file tiket `.scratch/<fitur>/issues/*.md`, atau sebagai nilai kolom `Status` di [`tasks/backlog.md`](../../tasks/backlog.md).

**Catatan penting untuk repo ini:** kolom `Status` di `tasks/backlog.md` sudah punya kosakata sendiri (`todo`, `in-progress`, `done`) yang berarti **progres**, bukan **triage**. Jangan campur keduanya dalam satu kolom:

- **Progres** → kolom `Status` di `tasks/backlog.md` (`todo`/`in-progress`/`done`).
- **Triage** → hanya untuk issue yang **datang dari luar** (bug report, permintaan fitur). Ditulis sebagai baris `Triage:` di tiket `.scratch/`.

Tiket yang dibuat `/to-tickets` **sudah agent-ready** — jangan di-triage ulang (aturan `/triage`).
