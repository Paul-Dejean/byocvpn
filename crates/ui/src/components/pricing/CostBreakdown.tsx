import { LedgerEntryWithCost } from "../../types/ledger";
import { formatBytes } from "../../lib/bytes";

const HOURS_PER_MONTH = 730;

interface CostBreakdownProps {
  entry: LedgerEntryWithCost;
}

export function CostBreakdown({ entry }: CostBreakdownProps) {
  if (entry.isPricingUnknown) {
    return (
      <p className="text-caption text-fg-medium">
        Pricing is unavailable for this instance type, so no cost
        estimate can be shown.
      </p>
    );
  }

  const bytesSentGb = entry.bytesSent / 1024 ** 3;
  const hourlyComputeRate =
    entry.uptimeHours > 0 ? entry.computeCost / entry.uptimeHours : 0;
  const hourlyIpRate =
    entry.uptimeHours > 0 ? entry.ipCost / entry.uptimeHours : 0;
  const egressRatePerGb =
    bytesSentGb > 0 ? entry.egressCost / bytesSentGb : 0;
  const storageHourlyRate = entry.storageRatePerGbMonth / HOURS_PER_MONTH;

  return (
    <div className="flex flex-col gap-3">
      <dl className="grid grid-cols-[120px_1fr_auto] max-md:grid-cols-[1fr_auto] gap-x-6 max-md:gap-x-3 gap-y-1.5 max-w-[520px] text-caption whitespace-nowrap max-md:whitespace-normal">
        <CostLine
          label="Compute time"
          detail={`${entry.uptimeHours.toFixed(2)} h × $${hourlyComputeRate.toFixed(5)}/hr`}
          amount={entry.computeCost}
        />
        <CostLine
          label="Reserved IP"
          detail={`${entry.uptimeHours.toFixed(2)} h × $${hourlyIpRate.toFixed(5)}/hr`}
          amount={entry.ipCost}
        />
        <CostLine
          label="Data egress"
          detail={`${bytesSentGb.toFixed(4)} GB × $${egressRatePerGb.toFixed(4)}/GB`}
          amount={entry.egressCost}
        />
        <CostLine
          label="Block storage"
          detail={`${entry.storageGb} GB × $${storageHourlyRate.toFixed(6)}/hr`}
          amount={entry.storageCost}
        />
        <dt className="col-span-2 max-md:col-span-1 pt-1.5 border-t border-bd-faint text-fg-lighter">
          Total
        </dt>
        <dd className="pt-1.5 border-t border-bd-faint text-right text-fg-lighter tabular-nums">
          ${entry.estimatedCost.toFixed(4)}
        </dd>
      </dl>
      <p className="text-caption text-fg-moderate">
        Sent {formatBytes(entry.bytesSent)} · Received {formatBytes(entry.bytesReceived)}
      </p>
    </div>
  );
}

interface CostLineProps {
  label: string;
  detail: string;
  amount: number;
}

function CostLine({ label, detail, amount }: CostLineProps) {
  return (
    <>
      <dt className="text-fg-medium max-md:col-start-1">
        {label}
        <span className="hidden max-md:block text-fg-moderate">{detail}</span>
      </dt>
      <dd className="text-fg-moderate max-md:hidden">{detail}</dd>
      <dd className="text-right text-fg-lighter tabular-nums">${amount.toFixed(4)}</dd>
    </>
  );
}
