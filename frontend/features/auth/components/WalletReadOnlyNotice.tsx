import { t } from "@/i18n";

/** Dipakai halaman saat tidak ada wallet — mode baca, bukan halaman kosong. */
export function WalletReadOnlyNotice() {
  return <p className="text-sm text-zinc-500">{t("auth.readOnlyNotice")}</p>;
}
