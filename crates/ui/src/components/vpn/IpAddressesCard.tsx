import { PanelField } from "./PanelField";

interface IpAddressesCardProps {
  ipV4: string | null;
  ipV6: string | null;
}

export function IpAddressesCard({ ipV4, ipV6 }: IpAddressesCardProps) {
  return (
    <section className="rounded-lg bg-bg-medium p-4 flex flex-col gap-3">
      <h3 className="text-body-sm text-fg-lighter">IP</h3>
      <div className="flex flex-col gap-3">
        <PanelField label="IPv4" value={ipV4 || "—"} wrap />
        <PanelField label="IPv6" value={ipV6 || "—"} wrap />
      </div>
    </section>
  );
}
