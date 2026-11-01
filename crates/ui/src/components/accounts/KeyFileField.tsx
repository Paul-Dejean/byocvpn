import { useRef } from "react";
import { Upload } from "lucide-react";
import { toast } from "react-hot-toast";
import { toAppError } from "../../lib/appError";
import { Button } from "../primitives/Button";
import { readTextFile } from "../../lib/readTextFile";

interface KeyFileFieldProps {
  label: string;
  accept: string;
  value: string;
  onChange: (content: string) => void;
  onFileLoaded?: (content: string) => void;
  rows?: number;
  placeholder?: string;
  alreadySetMessage?: string | undefined;
  emptyHint?: string;
}

export function KeyFileField({
  label,
  accept,
  value,
  onChange,
  onFileLoaded,
  rows = 6,
  placeholder,
  alreadySetMessage,
  emptyHint,
}: KeyFileFieldProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    try {
      const content = await readTextFile(file);
      onChange(content);
      onFileLoaded?.(content);
    } catch (error) {
      toast.error(`Could not read ${file.name}: ${toAppError(error).message}`);
    }
  };

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center justify-between">
        <label className="text-caption text-fg-medium">{label}</label>
        <Button
          variant="secondary"
          size="md"
          type="button"
          onClick={() => fileInputRef.current?.click()}
          icon={<Upload size={14} />}
        >
          Load from file
        </Button>
        <input
          ref={fileInputRef}
          type="file"
          accept={accept}
          onChange={handleFileChange}
          className="hidden"
        />
      </div>
      {!value && alreadySetMessage && (
        <p className="text-caption text-fg-success-moderate">{alreadySetMessage}</p>
      )}
      {!value && !alreadySetMessage && emptyHint && (
        <p className="text-caption text-fg-moderate">{emptyHint}</p>
      )}
      <textarea
        value={value}
        onChange={(event) => onChange(event.target.value)}
        rows={rows}
        placeholder={placeholder}
        className="input resize-none font-mono text-caption"
      />
    </div>
  );
}
