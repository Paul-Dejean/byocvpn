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
      return;
    }
    function handleClickOutside(event: MouseEvent) {
      if (!containerRef.current?.contains(event.target as Node)) {
        setIsOpen(false);
      }
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
        className="flex items-center gap-2 h-8 px-3 rounded-md bg-bg-medium border border-bd-moderate text-body-sm text-fg-lighter whitespace-nowrap hover:bg-bg-light transition-colors"
      >
        <ProviderIcon provider={selectedProvider} className="w-4 h-4" />
        {buildProviderLabel(selectedProvider)}
        <ChevronDown size={14} className="text-fg-medium" />
      </button>

      {isOpen && (
        <ul
          role="listbox"
          className="absolute right-0 mt-1 w-48 rounded-md bg-bg-medium border border-bd-moderate shadow-xl py-1 z-10"
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
                className={`w-full flex items-center gap-2 px-3 py-2 text-body-sm text-left hover:bg-bg-light transition-colors ${
                  provider === selectedProvider ? "text-fg-lighter" : "text-fg-medium"
                }`}
              >
                <ProviderIcon provider={provider} className="w-4 h-4" />
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
