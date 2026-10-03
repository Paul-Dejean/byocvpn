import { Instance } from "../../types";
import { useVpnConnectionContext } from "../../contexts/VpnConnectionContext";
import { Button } from "../primitives/Button";
import { Alert } from "../primitives/Alert";
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

      <div className="w-[486px] rounded-xl bg-gray-750 border border-gray-500/50 p-4 flex flex-col gap-4">
        <ServerLocation provider={instance.provider} region={instance.region} size="lg" />
        <Button
          variant="primary"
          size="none"
          loading={isConnecting}
          onClick={handleSecureConnection}
          className="w-full py-2 text-sm"
        >
          Secure my connection
        </Button>
        {error && <Alert variant="error">{error}</Alert>}
      </div>
    </div>
  );
}
