# Domain Docs — cara skill membaca dokumentasi domain repo ini

> Dibaca oleh skill engineering (Matt Pocock) sebelum mengeksplorasi kode.
> Dibuat ronde 17c. **Aturan perilaku tetap milik [AGENTS.md](../../AGENTS.md); file ini hanya memetakan lokasi.**

## Sebelum eksplorasi, baca ini

- **[CONTEXT.md](../../CONTEXT.md)** di root — glosarium istilah domain (istilah kanonik + istilah terlarang).
- **`docs/decisions/`** — baca ADR yang menyentuh area kerja. **Catatan: di repo ini rumah ADR adalah `docs/decisions/ADR-NNN-slug.md`, BUKAN `docs/adr/NNNN-slug.md`.** Jangan buat folder `docs/adr/` — rumah ADR kedua = kekacauan, dan akan memutus 361 rujukan `ADR-0…` + 65 rujukan `decisions/` yang sudah ada di 97 file.
- **[docs/DOCUMENTATION-MAP.md](../DOCUMENTATION-MAP.md)** — SSOT: peta "perubahan apa → dokumen mana yang wajib dicek" + indeks lengkap semua file.
- **[docs/security/security-requirements.md](../security/security-requirements.md)** — requirement `SEC-*` berstatus P0. **Bagian dari definisi "selesai"**, bukan dokumen opsional.

Repo ini **single-context** (tidak ada `CONTEXT-MAP.md`; tidak ada monorepo signal). Satu `CONTEXT.md` di root melayani seluruh repo.

## Tata letak aktual

```text
/
├── CONTEXT.md                 # glosarium domain (kanonik)
├── AGENTS.md                  # aturan kerja agent (mengikat)
├── docs/
│   ├── DOCUMENTATION-MAP.md   # peta sinkronisasi + indeks semua dokumen
│   ├── decisions/             # ADR-NNN-slug.md   ← rumah ADR repo ini
│   ├── features/<fitur>.md    # spesifikasi perilaku (pemilik perilaku)
│   ├── contracts/<kontrak>.md # reference implementasi (signature/layout/konstanta)
│   ├── security/              # SEC-*, threat model, invariant
│   └── ...                    # api/ architecture/ database/ testing/ deployment/ development/ research/
└── tasks/
    ├── backlog.md             # SSOT task & prioritas
    └── milestones.md          # SSOT milestone
```

### Pemisahan yang harus dihormati

| Pertanyaan | Dokumen pemilik |
|---|---|
| "Apa perilaku yang benar?" | `docs/features/<fitur>.md` |
| "Bagaimana signature/layout state-nya?" | `docs/contracts/<kontrak>.md` |
| "Kenapa keputusan ini diambil?" | `docs/decisions/ADR-NNN-*.md` |
| "Apa arti istilah ini?" | `CONTEXT.md` |
| "Apa yang harus di-update saat X berubah?" | `docs/DOCUMENTATION-MAP.md` |

`docs/contracts/*` adalah **reference implementasi**, bukan pemilik perilaku. Kalau keduanya bentrok, `features/*` menang dan `contracts/*` yang dikoreksi.

## Pakai kosakata glosarium

Saat output menamai konsep domain (judul issue, nama test, usulan refactor), pakai istilah **sebagaimana didefinisikan di `CONTEXT.md`**. Jangan melenceng ke sinonim yang glosarium eksplisit hindari (mis. `order` → pakai `Sale`/`Offer`/`Bundle` sesuai konteks; `MVP` → pakai `M1`/`M1+`).

Kalau konsep yang dibutuhkan belum ada di glosarium, itu sinyal: entah kamu sedang menciptakan bahasa yang tidak dipakai proyek ini (pertimbangkan ulang), atau ada gap nyata (catat untuk `/domain-modeling`).

## Tandai konflik ADR

Kalau output bertentangan dengan ADR yang ada, **permukaan secara eksplisit** — jangan diam-diam menimpanya:

> _Bertentangan dengan ADR-007 (approval listing model), tapi layak dibuka ulang karena…_

## Kalau file-file ini tidak ada

Prosedur normal: **lanjut diam-diam**, jangan menghentikan pekerjaan untuk menawarkan pembuatannya. Tapi di repo ini semuanya sudah ada — jangan membuat ulang dengan nama berbeda.
