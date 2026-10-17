interface SpinnerProps {
  size?: string;
  color?: string;
  thickness?: string;
}

export function Spinner({
  size = "w-4 h-4",
  color = "border-current",
  thickness = "border-2",
}: SpinnerProps) {
  return (
    <div
      className={`${size} ${thickness} ${color} shrink-0 animate-spin rounded-full border-t-transparent`}
    />
  );
}
