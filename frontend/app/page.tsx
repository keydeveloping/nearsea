import { t } from "@/i18n";

export default function Home() {
  return (
    <main className="flex flex-1 flex-col items-center justify-center gap-4 px-6 py-24 text-center">
      <h1 className="text-4xl font-semibold tracking-tight">{t("common.appName")}</h1>
      <p className="max-w-xl text-lg text-zinc-600 dark:text-zinc-400">{t("common.tagline")}</p>
      <p className="text-sm text-zinc-500">{t("common.scaffoldNotice")}</p>
    </main>
  );
}
