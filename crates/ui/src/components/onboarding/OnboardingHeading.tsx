import { Logo } from "../common/Logo";

interface OnboardingHeadingProps {
  title: string;
  subtitle: string;
}

export function OnboardingHeading({ title, subtitle }: OnboardingHeadingProps) {
  return (
    <div className="flex flex-col items-center gap-8 text-center">
      <div className="w-12 h-12 rounded-xl bg-bg-medium border border-bd-moderate flex items-center justify-center">
        <Logo className="w-6 h-6" />
      </div>
      <div className="flex flex-col items-center gap-2">
        <h1 className="text-heading-md font-medium text-fg-lighter">{title}</h1>
        <p className="text-body-sm text-fg-medium">{subtitle}</p>
      </div>
    </div>
  );
}
