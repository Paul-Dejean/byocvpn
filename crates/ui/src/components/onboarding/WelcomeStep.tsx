import { Button } from "../primitives/Button";
import { Logo } from "../common/Logo";

interface WelcomeStepProps {
  onGetStarted: () => void;
}

export function WelcomeStep({ onGetStarted }: WelcomeStepProps) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-10">
      <div className="flex size-[72px] items-center justify-center rounded-2xl border border-bd-moderate bg-bg-medium">
        <Logo className="size-9" />
      </div>

      <div className="flex flex-col items-center gap-2 text-center">
        <h1 className="text-heading-lg font-medium text-fg-lighter">Welcome to ByocVPN</h1>
        <p className="text-feature text-fg-medium">A VPN you actually own.</p>
      </div>

      <Button variant="primary" size="lg" onClick={onGetStarted}>
        Get started
      </Button>
    </div>
  );
}
