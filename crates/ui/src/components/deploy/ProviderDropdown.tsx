import { useEffect, useRef, useState } from "react";
import { ChevronDown } from "lucide-react";
import { CloudProviderName } from "../../types";
import { PROVIDER_METADATA } from "../../constants/providers";
import { ProviderIcon } from "../providers/ProviderIcon";

interface ProviderDropdownProps {
  providers: CloudProviderName[];
  selectedProvider: CloudProviderName;
  onSelectProvider: (provider: CloudProviderName) => void;
}

export function ProviderDropdown({
  providers,
  selectedProvider,
  onSelectProvider,
}: ProviderDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) {
      return undefined;
    }
    function handleClickOutside(event: MouseEvent) {
      if (event.target instanceof Node && containerRef.current?.contains(event.target)) {
        return;
      }
      setIsOpen(false);
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isOpen]);

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        onClick={() => setIsOpen((previous) => !previous)}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        className="flex h-8 items-center gap-2 rounded-md border border-bd-moderate bg-bg-medium px-3 text-body-sm whitespace-nowrap text-fg-lighter transition-colors hover:bg-bg-light"
      >
        <ProviderIcon provider={selectedProvider} className="size-4" />
        {buildProviderLabel(selectedProvider)}
        <ChevronDown size={14} className="text-fg-medium" />
      </button>

      {isOpen && (
        <ul
          role="listbox"
          className="absolute right-0 z-10 mt-1 w-48 rounded-md border border-bd-moderate bg-bg-medium py-1 shadow-xl"
        >
          {providers.map((provider) => (
            <li key={provider}>
              <button
                type="button"
                role="option"
                aria-selected={provider === selectedProvider}
                onClick={() => {
                  onSelectProvider(provider);
                  setIsOpen(false);
                }}
                className={`flex w-full items-center gap-2 px-3 py-2 text-left text-body-sm transition-colors hover:bg-bg-light ${
                  provider === selectedProvider ? "text-fg-lighter" : "text-fg-medium"
                }`}
              >
                <ProviderIcon provider={provider} className="size-4" />
                {buildProviderLabel(provider)}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function buildProviderLabel(provider: CloudProviderName): string {
  return `${PROVIDER_METADATA[provider].shortLabel} Account`;
}
