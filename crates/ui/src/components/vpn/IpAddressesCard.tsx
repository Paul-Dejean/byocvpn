import { PanelField } from "./PanelField";

interface IpAddressesCardProps {
  ipV4: string | null;
  ipV6: string | null;
}

export function IpAddressesCard({ ipV4, ipV6 }: IpAddressesCardProps) {
  return (
    <section className="w-full rounded-lg bg-gray-700 p-3 flex flex-col gap-3">
      <h3 className="text-sm text-primary">IP</h3>
      <PanelField label="IPv4" value={ipV4 || "—"} mono />
      <PanelField label="IPv6" value={ipV6 || "—"} mono />
    </section>
  );
}
