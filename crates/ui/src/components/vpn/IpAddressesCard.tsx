import { PanelField } from "./PanelField";

interface IpAddressesCardProps {
  ipV4: string | null;
  ipV6: string | null;
}

export function IpAddressesCard({ ipV4, ipV6 }: IpAddressesCardProps) {
  return (
    <section className="rounded-lg bg-gray-700 p-4 flex flex-col gap-3">
      <h3 className="text-sm text-primary">IP</h3>
      <div className="flex flex-col gap-3">
        <PanelField label="IPv4" value={ipV4 || "—"} wrap />
        <PanelField label="IPv6" value={ipV6 || "—"} wrap />
      </div>
    </section>
  );
}
