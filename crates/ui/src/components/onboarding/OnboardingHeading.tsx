import { Logo } from "../common/Logo";

interface OnboardingHeadingProps {
  title: string;
  subtitle: string;
}

export function OnboardingHeading({ title, subtitle }: OnboardingHeadingProps) {
  return (
    <div className="flex flex-col items-center gap-8 text-center">
      <div className="flex size-12 items-center justify-center rounded-xl border border-bd-moderate bg-bg-medium">
        <Logo className="size-6" />
      </div>
      <div className="flex flex-col items-center gap-2">
        <h1 className="text-heading-md font-medium text-fg-lighter">{title}</h1>
        <p className="text-body-sm text-fg-medium">{subtitle}</p>
      </div>
    </div>
  );
}
