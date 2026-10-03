import { Button } from "../primitives/Button";
import { Logo } from "../common/Logo";

interface WelcomeStepProps {
  onGetStarted: () => void;
}

export function WelcomeStep({ onGetStarted }: WelcomeStepProps) {
  return (
    <div className="h-full flex flex-col items-center justify-center gap-10">
      <div className="w-[72px] h-[72px] rounded-2xl bg-gray-700 border border-gray-500/60 flex items-center justify-center">
        <Logo className="w-9 h-9" />
      </div>

      <div className="flex flex-col items-center gap-3 text-center">
        <h1 className="text-3xl font-medium text-primary">Welcome to ByocVPN</h1>
        <p className="text-lg text-gray-300">A VPN you actually own.</p>
      </div>

      <Button
        variant="primary"
        size="none"
        onClick={onGetStarted}
        className="px-4 py-2 text-sm"
      >
        Get started
      </Button>
    </div>
  );
}
