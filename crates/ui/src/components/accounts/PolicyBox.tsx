import { useState } from "react";
import { Check, Copy, Download } from "lucide-react";
import { save } from "@tauri-apps/plugin-dialog";
import { toast } from "react-hot-toast";
import { commands } from "../../bindings";
import { ProviderPolicy } from "../../constants/providerSetup";
import { summarizeError } from "../../lib/describeError";
import { fromPromise } from "../../lib/result";
import { Button } from "../primitives/Button";

interface PolicyBoxProps {
  policy: ProviderPolicy;
}

export function PolicyBox({ policy }: PolicyBoxProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(policy.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleDownload = async () => {
    const dialogResult = await fromPromise(save({ defaultPath: policy.filename }));
    if (dialogResult.status === "error") {
      toast.error(dialogResult.error);
      return;
    }
    const savePath = dialogResult.data;
    if (!savePath) {
      return;
    }
    const saveResult = await commands.saveFile(savePath, policy.content);
    if (saveResult.status === "error") {
      toast.error(summarizeError(saveResult.error));
    }
  };

  return (
    <div className="flex max-h-80 flex-col overflow-hidden rounded-lg border border-bd-moderate bg-bg-medium">
      <div className="flex items-center justify-between gap-3 border-b border-bd-faint px-4 py-2">
        <span className="text-caption text-fg-medium">
          Paste this when the console asks for a policy
        </span>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            size="md"
            onClick={handleCopy}
            icon={
              copied ? <Check size={14} className="text-fg-success-moderate" /> : <Copy size={14} />
            }
          >
            {copied ? "Copied" : "Copy"}
          </Button>
          <Button
            variant="secondary"
            size="md"
            onClick={handleDownload}
            icon={<Download size={14} />}
          >
            Download
          </Button>
        </div>
      </div>
      <pre className="flex-1 overflow-auto p-4 font-mono text-caption leading-relaxed text-fg-medium">
        {policy.content}
      </pre>
    </div>
  );
}
