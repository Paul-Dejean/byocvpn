import { useState } from "react";
import { Check, Search, Star } from "lucide-react";
import { CloudProviderName, Region } from "../../types";
import { getRegionInfo } from "../../constants/regionInfo";
import { getCountryName } from "../../lib/countryName";
import { useFavoriteRegions } from "../../hooks/useFavoriteRegions";
import { useProviderRegions } from "../../hooks/useProviderRegions";
import { FlagIcon } from "../FlagIcon";
import { Button } from "../primitives/Button";
import { Spinner } from "../primitives/Spinner";

interface RegionListProps {
  provider: CloudProviderName;
  selectedRegion?: Region | null;
  onSelectRegion?: (region: Region) => void;
  onDeployRegion?: (region: Region) => void;
  headerAccessory?: React.ReactNode;
}

interface RegionRowData {
  region: Region;
  countryName: string;
  city: string;
  countryCode: string;
}

export function RegionList({
  provider,
  selectedRegion = null,
  onSelectRegion,
  onDeployRegion,
  headerAccessory,
}: RegionListProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const { regions, isLoading } = useProviderRegions(provider);
  const { favoriteRegions, isFavorite, toggleFavorite } = useFavoriteRegions(provider);

  const rows: RegionRowData[] = disambiguateDuplicateLabels(
    regions.map((region) => buildRow(provider, region)),
  ).toSorted(compareRows);

  const query = searchQuery.trim().toLowerCase();
  const visibleRows = query
    ? rows.filter(
        (row) =>
          row.countryName.toLowerCase().includes(query) ||
          row.city.toLowerCase().includes(query) ||
          row.region.name.toLowerCase().includes(query),
      )
    : rows;

  const favoriteRows = visibleRows.filter((row) => favoriteRegions.includes(row.region.name));
  const otherRows = visibleRows.filter((row) => !favoriteRegions.includes(row.region.name));

  function renderRow(row: RegionRowData) {
    return (
      <RegionRow
        key={row.region.name}
        row={row}
        isSelected={selectedRegion?.name === row.region.name}
        isFavorite={isFavorite(row.region.name)}
        onToggleFavorite={() => toggleFavorite(row.region.name)}
        onSelect={onSelectRegion ? () => onSelectRegion(row.region) : undefined}
        onDeploy={onDeployRegion ? () => onDeployRegion(row.region) : undefined}
      />
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4">
      <div className="flex items-center gap-2">
        <label className="flex h-8 min-w-0 flex-1 items-center gap-2 rounded-md bg-bg-medium px-3 text-body-sm text-fg-medium focus-within:ring-1 focus-within:ring-bd-brand">
          <Search size={16} className="shrink-0" />
          <input
            type="search"
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
            placeholder="Search"
            className="w-full min-w-0 flex-1 bg-transparent text-fg-lighter outline-none placeholder:text-fg-medium"
          />
        </label>
        {headerAccessory}
      </div>

      {isLoading ? (
        <div className="flex justify-center py-10">
          <Spinner size="w-6 h-6" color="border-bd-strong" />
        </div>
      ) : (
        <div className="scroll-fade flex min-h-0 flex-col gap-2 overflow-y-auto py-3 pr-1">
          {favoriteRows.length > 0 && (
            <RegionSection title="Favorite" count={favoriteRows.length}>
              {favoriteRows.map(renderRow)}
            </RegionSection>
          )}
          <RegionSection
            title={favoriteRows.length > 0 ? "Others" : "All"}
            count={otherRows.length}
          >
            {otherRows.map(renderRow)}
          </RegionSection>
        </div>
      )}
    </div>
  );
}

function buildRow(provider: CloudProviderName, region: Region): RegionRowData {
  const regionInfo = getRegionInfo(provider, region.name);
  return {
    region,
    countryName: getCountryName(regionInfo.countryCode) || region.country,
    city: regionInfo.city,
    countryCode: regionInfo.countryCode,
  };
}

function disambiguateDuplicateLabels(rows: RegionRowData[]): RegionRowData[] {
  const labelCounts = new Map<string, number>();
  for (const row of rows) {
    const label = buildLabelKey(row);
    labelCounts.set(label, (labelCounts.get(label) ?? 0) + 1);
  }
  return rows.map((row) =>
    (labelCounts.get(buildLabelKey(row)) ?? 0) > 1
      ? { ...row, city: `${row.city} · ${row.region.name}` }
      : row,
  );
}

function buildLabelKey(row: RegionRowData): string {
  return `${row.countryName}|${row.city}`;
}

function compareRows(first: RegionRowData, second: RegionRowData): number {
  return (
    first.countryName.localeCompare(second.countryName) || first.city.localeCompare(second.city)
  );
}

interface RegionSectionProps {
  title: string;
  count: number;
  children: React.ReactNode;
}

function RegionSection({ title, count, children }: RegionSectionProps) {
  return (
    <section className="flex flex-col gap-1">
      <h3 className="px-3 py-1 text-body-sm text-fg-medium">
        {title} [{count}]
      </h3>
      {children}
    </section>
  );
}

interface RegionRowProps {
  row: RegionRowData;
  isSelected: boolean;
  isFavorite: boolean;
  onToggleFavorite: () => void;
  onSelect?: () => void;
  onDeploy?: () => void;
}

function RegionRow({
  row,
  isSelected,
  isFavorite,
  onToggleFavorite,
  onSelect,
  onDeploy,
}: RegionRowProps) {
  const isSelectable = onSelect !== undefined;

  return (
    <div
      role={isSelectable ? "button" : undefined}
      tabIndex={isSelectable ? 0 : undefined}
      onClick={onSelect}
      onKeyDown={(event) => {
        if (isSelectable && (event.key === "Enter" || event.key === " ")) {
          event.preventDefault();
          onSelect?.();
        }
      }}
      className={`group flex items-center gap-3 rounded-lg px-3 py-1.5 transition-colors ${
        isSelected ? "bg-bg-medium" : "hover:bg-bg-light"
      } ${isSelectable ? "cursor-pointer" : ""}`}
    >
      <FlagIcon countryCode={row.countryCode} round size={24} />
      <div className="flex min-w-0 flex-1 flex-col">
        <span className="truncate text-body-sm text-fg-lighter">{row.countryName}</span>
        <span className="truncate text-caption text-fg-moderate">
          {row.city || row.region.name}
        </span>
      </div>
      {isSelected && <Check size={16} className="shrink-0 text-fg-brand" />}
      {onDeploy && (
        <Button
          variant="outline"
          size="lg"
          onClick={(event) => {
            event.stopPropagation();
            onDeploy();
          }}
          className="opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100 max-md:opacity-100"
        >
          Deploy
        </Button>
      )}
      <button
        type="button"
        onClick={(event) => {
          event.stopPropagation();
          onToggleFavorite();
        }}
        aria-label={isFavorite ? "Remove from favorites" : "Add to favorites"}
        aria-pressed={isFavorite}
        className="flex size-7 items-center justify-center rounded-md border border-bd-moderate text-fg-lighter transition-colors hover:bg-bg-light"
      >
        <Star size={14} className={isFavorite ? "fill-current" : ""} />
      </button>
    </div>
  );
}
