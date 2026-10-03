export type FlagSize = 16 | 20 | 24;

interface FlagIconProps {
  countryCode: string;
  round?: boolean;
  size?: FlagSize;
  className?: string;
}

const ROUND_SIZE_CLASSES: Record<FlagSize, string> = {
  16: "!w-4 !h-4",
  20: "!w-5 !h-5",
  24: "!w-6 !h-6",
};

export function FlagIcon({
  countryCode,
  round = false,
  size = 20,
  className = "",
}: FlagIconProps) {
  if (!countryCode) return null;
  const shapeClasses = round ? `fis rounded-full ${ROUND_SIZE_CLASSES[size]}` : "";
  return (
    <span
      className={`fi fi-${countryCode} ${shapeClasses} flex-shrink-0 ${className}`}
    />
  );
}
