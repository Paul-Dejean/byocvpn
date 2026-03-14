import { LedgerEntryWithCost } from "../../types/ledger";

interface InstanceCostRowProps {

  entry: LedgerEntryWithCost;
}

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatUptime(hours: number): string {
  if (hours < 1) return `${Math.round(hours * 60)}m`;
  if (hours < 24) return `${hours.toFixed(1)}h`;
  return `${(hours / 24).toFixed(1)}d`;
}

export function InstanceCostRow({ entry }: InstanceCostRowProps) {
  const isActive = entry.terminatedAt === null;

  return (
    <tr className="border-b border-gray-700/60 hover:bg-gray-700/30 transition-colors">
      <td className="py-3 px-4 font-mono text-xs text-gray-400">
        {entry.instanceId.length > 22
          ? `${entry.instanceId.slice(0, 22)}…`
          : entry.instanceId}
      </td>
      <td className="py-3 px-4 text-sm text-gray-300">{entry.region}</td>
      <td className="py-3 px-4 text-sm font-mono text-blue-300">
        {entry.instanceType}
      </td>
      <td className="py-3 px-4 text-sm text-gray-400">
        {formatDate(entry.launchedAt)}
      </td>
      <td className="py-3 px-4 text-sm">
        {isActive ? (
          <span className="inline-flex items-center gap-1.5 text-green-400">
            <span className="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse inline-block" />
            Active
          </span>
        ) : (
          <span className="text-gray-400">
            {formatDate(entry.terminatedAt!)}
          </span>
        )}
      </td>
      <td className="py-3 px-4 text-sm text-gray-300">
        {formatUptime(entry.uptimeHours)}
      </td>
      <td className="py-3 px-4 text-sm font-semibold text-yellow-300">
        ${entry.estimatedCost.toFixed(4)}
      </td>
    </tr>
  );
}
