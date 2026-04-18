interface FlagIconProps {
  countryCode: string;
  className?: string;
}

export function FlagIcon({ countryCode, className = "" }: FlagIconProps) {
  if (!countryCode) return null;
  return <span className={`fi fi-${countryCode} ${className}`} />;
}
