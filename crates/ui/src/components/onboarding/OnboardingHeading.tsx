import { Logo } from "../common/Logo";

interface OnboardingHeadingProps {
  title: string;
  subtitle: string;
}

export function OnboardingHeading({ title, subtitle }: OnboardingHeadingProps) {
  return (
    <div className="flex flex-col items-center gap-6 text-center">
      <div className="w-12 h-12 rounded-xl bg-gray-700 border border-gray-500/60 flex items-center justify-center">
        <Logo className="w-6 h-6" />
      </div>
      <div className="flex flex-col items-center gap-2">
        <h1 className="text-2xl font-medium text-primary">{title}</h1>
        <p className="text-sm text-gray-300">{subtitle}</p>
      </div>
    </div>
  );
}
