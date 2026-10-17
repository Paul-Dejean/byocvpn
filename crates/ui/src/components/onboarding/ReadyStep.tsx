import { Instance } from "../../types";
import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";
import { Button } from "../primitives/Button";
import { Banner } from "../primitives/Banner";
import { ServerLocation } from "../servers/ServerLocation";
import { OnboardingHeading } from "./OnboardingHeading";

interface ReadyStepProps {
  instance: Instance;
  onConnected: () => void;
}

export function ReadyStep({ instance, onConnected }: ReadyStepProps) {
  const { connectToVpn, isConnecting, error } = useVpnConnectionContext();

  async function handleSecureConnection() {
    await connectToVpn(instance);
    onConnected();
  }

  return (
    <div className="flex h-full flex-col items-center justify-center gap-10">
      <OnboardingHeading
        title="Ready to go!"
        subtitle="Your server is ready. Connect to activate your VPN protection."
      />

      <div className="flex w-full max-w-[486px] flex-col gap-4 rounded-xl border border-bd-moderate bg-bg-medium p-4">
        <ServerLocation provider={instance.provider} region={instance.region} size="lg" />
        <Button
          variant="primary"
          size="lg"
          loading={isConnecting}
          onClick={handleSecureConnection}
          className="w-full"
        >
          Secure my connection
        </Button>
        {error && <Banner variant="danger">{error}</Banner>}
      </div>
    </div>
  );
}
