export type FlagSize = 16 | 20 | 24;

interface FlagIconProps {
  countryCode: string;
  round?: boolean;
  size?: FlagSize;
  className?: string;
}

const FLAG_URLS_BY_PATH = import.meta.glob<string>("/node_modules/flag-icons/flags/4x3/*.svg", {
  eager: true,
  query: "?url",
  import: "default",
});

const FLAG_URLS_BY_COUNTRY_CODE = new Map(
  Object.entries(FLAG_URLS_BY_PATH).map(([path, url]) => [extractCountryCode(path), url]),
);

const RECTANGULAR_SIZE_CLASSES: Record<FlagSize, string> = {
  16: "h-4",
  20: "h-5",
  24: "h-6",
};

const ROUND_SIZE_CLASSES: Record<FlagSize, string> = {
  16: "size-4",
  20: "size-5",
  24: "size-6",
};

export function FlagIcon({ countryCode, round = false, size = 20, className = "" }: FlagIconProps) {
  const flagUrl = FLAG_URLS_BY_COUNTRY_CODE.get(countryCode.toLowerCase());
  if (!flagUrl) return null;
  const shapeClasses = round
    ? `${ROUND_SIZE_CLASSES[size]} rounded-full object-cover`
    : `${RECTANGULAR_SIZE_CLASSES[size]} w-auto`;
  return <img src={flagUrl} alt="" className={`${shapeClasses} shrink-0 ${className}`} />;
}

function extractCountryCode(flagPath: string): string {
  return flagPath.slice(flagPath.lastIndexOf("/") + 1, -".svg".length);
}
