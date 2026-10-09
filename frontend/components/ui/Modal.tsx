"use client";

import { useEffect, useId, useRef, type ReactNode } from "react";

/**
 * Dialog modal: Esc & klik latar menutup, fokus terkurung di dalam dialog, dan fokus kembali
 * ke pemicu saat tutup (docs/04-ux-ui-spec.md §Accessibility).
 *
 * `dismissible` mematikan ketiga jalur tutup itu — dipakai saat transaksi sedang berjalan,
 * di mana menutup dialog akan menyembunyikan progres dan hasilnya dari user.
 */
export function Modal({
  open,
  title,
  onClose,
  dismissible = true,
  children,
}: {
  open: boolean;
  title: string;
  onClose: () => void;
  dismissible?: boolean;
  children: ReactNode;
}) {
  const dialogRef = useRef<HTMLDivElement>(null);
  const titleId = useId();

  // Fokus dikembalikan ke elemen yang membuka dialog; kalau tidak, fokus jatuh ke <body>
  // dan pengguna keyboard kehilangan posisinya di halaman.
  useEffect(() => {
    if (!open) return;

    const trigger = document.activeElement;
    dialogRef.current?.focus();

    return () => {
      if (trigger instanceof HTMLElement) trigger.focus();
    };
  }, [open]);

  useEffect(() => {
    if (!open) return;

    const onKeyDown = (event: KeyboardEvent) => {
      const dialog = dialogRef.current;
      if (dialog === null) return;

      if (event.key === "Escape") {
        if (dismissible) onClose();
        return;
      }
      if (event.key !== "Tab") return;

      const focusable = dialog.querySelectorAll<HTMLElement>(
        'button:not([disabled]), [href], input:not([disabled]), select, textarea, [tabindex]:not([tabindex="-1"])',
      );
      if (focusable.length === 0) return;

      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      const active = document.activeElement;

      if (event.shiftKey && active === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && active === last) {
        event.preventDefault();
        first.focus();
      }
    };

    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [open, onClose, dismissible]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      onClick={() => {
        if (dismissible) onClose();
      }}
    >
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
        onClick={(event) => event.stopPropagation()}
        className="w-full max-w-md rounded-lg border border-zinc-200 bg-white p-6 shadow-xl outline-none dark:border-zinc-800 dark:bg-zinc-950"
      >
        <h2 id={titleId} className="mb-4 text-lg font-semibold tracking-tight">
          {title}
        </h2>
        {children}
      </div>
    </div>
  );
}
