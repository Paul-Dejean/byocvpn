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
    <div className="h-full flex flex-col items-center justify-center gap-10">
      <OnboardingHeading
        title="Ready to go!"
        subtitle="Your server is ready. Connect to activate your VPN protection."
      />

      <div className="w-full max-w-[486px] rounded-xl bg-bg-medium border border-bd-moderate p-4 flex flex-col gap-4">
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
