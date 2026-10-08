import { format, t } from "@/i18n";
import { formatNear, type YoctoNear } from "@/lib/format/money";

/**
 * Harga Ⓝ dari string yoctoNEAR. Selalu lewat `lib/format` — tidak ada aritmetika float
 * pada nilai yocto (AGENTS.md §Blockchain Rules).
 */
export function PriceChip({
  amountYocto,
  className,
}: {
  amountYocto: YoctoNear;
  className?: string;
}) {
  return (
    <span className={className}>
      {format(t("common.amountNear"), { amount: formatNear(amountYocto) })}
    </span>
  );
}
