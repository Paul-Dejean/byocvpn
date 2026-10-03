import { useMemo, useState } from "react";
import { Search, Star } from "lucide-react";
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
  const { favoriteRegions, isFavorite, toggleFavorite } =
    useFavoriteRegions(provider);

  const rows = useMemo<RegionRowData[]>(
    () =>
      disambiguateDuplicateLabels(
        regions.map((region) => buildRow(provider, region)),
      ).sort(compareRows),
    [regions, provider],
  );

  const visibleRows = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    if (!query) {
      return rows;
    }
    return rows.filter(
      (row) =>
        row.countryName.toLowerCase().includes(query) ||
        row.city.toLowerCase().includes(query) ||
        row.region.name.toLowerCase().includes(query),
    );
  }, [rows, searchQuery]);

  const favoriteRows = visibleRows.filter((row) =>
    favoriteRegions.includes(row.region.name),
  );

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
    <div className="flex-1 flex flex-col gap-4 min-h-0">
      <div className="flex items-center gap-2">
        <label className="flex-1 flex items-center gap-2 px-3 py-2 rounded-lg bg-bg-medium text-body-sm text-fg-medium focus-within:ring-1 focus-within:ring-bd-brand">
          <Search size={16} className="flex-shrink-0" />
          <input
            type="search"
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
            placeholder="Search"
            className="flex-1 bg-transparent outline-none text-fg-lighter placeholder:text-fg-medium"
          />
        </label>
        {headerAccessory}
      </div>

      {isLoading ? (
        <div className="flex justify-center py-10">
          <Spinner size="w-6 h-6" color="border-bd-strong" />
        </div>
      ) : (
        <div className="flex flex-col gap-4 overflow-y-auto min-h-0 pr-1">
          {favoriteRows.length > 0 && (
            <RegionSection title="Favorite" count={favoriteRows.length}>
              {favoriteRows.map(renderRow)}
            </RegionSection>
          )}
          <RegionSection title="All" count={visibleRows.length}>
            {visibleRows.map(renderRow)}
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
    first.countryName.localeCompare(second.countryName) ||
    first.city.localeCompare(second.city)
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
      <h3 className="px-4 py-2 text-body-sm text-fg-medium">
        {title} <span className="text-fg-medium">[{count}]</span>
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
      className={`group flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
        isSelected
          ? "bg-bg-medium ring-1 ring-bd-brand"
          : "hover:bg-bg-medium"
      } ${isSelectable ? "cursor-pointer" : ""}`}
    >
      <FlagIcon countryCode={row.countryCode} round />
      <div className="flex-1 min-w-0 flex flex-col">
        <span className="text-body-sm text-fg-lighter truncate">{row.countryName}</span>
        <span className="text-caption text-fg-medium truncate">{row.city || row.region.name}</span>
      </div>
      {onDeploy && (
        <Button
          variant="secondary"
          size="lg"
          onClick={(event) => {
            event.stopPropagation();
            onDeploy();
          }}
          className="opacity-0 group-hover:opacity-100 focus-visible:opacity-100 transition-opacity"
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
        className="w-7 h-7 rounded-md border border-bd-moderate flex items-center justify-center text-fg-lighter hover:bg-bg-lighter transition-colors"
      >
        <Star size={14} className={isFavorite ? "fill-current" : ""} />
      </button>
    </div>
  );
}
