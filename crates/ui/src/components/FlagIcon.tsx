interface FlagIconProps {
  countryCode: string;
  round?: boolean;
  className?: string;
}

export function FlagIcon({
  countryCode,
  round = false,
  className = "",
}: FlagIconProps) {
  if (!countryCode) return null;
  const shapeClasses = round ? "fis rounded-full !w-5 !h-5" : "";
  return (
    <span
      className={`fi fi-${countryCode} ${shapeClasses} flex-shrink-0 ${className}`}
    />
  );
}
