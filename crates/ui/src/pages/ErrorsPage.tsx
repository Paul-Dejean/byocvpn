import { useEffect, useState } from "react";
import {
  CommandError,
  getCommandErrors,
  subscribeToCommandErrors,
} from "../lib/commandErrorStore";

export function ErrorsPage() {
  const [errors, setErrors] = useState<readonly CommandError[]>(() =>
    getCommandErrors()
  );

  useEffect(() => {
    return subscribeToCommandErrors(() => {
      setErrors([...getCommandErrors()]);
    });
  }, []);

  const [copyConfirmed, setCopyConfirmed] = useState(false);

  const copyToClipboard = async () => {
    const text = errors
      .map((error) => `[${error.timestamp}] ${error.command}: ${error.message}`)
      .join("\n");
    await navigator.clipboard.writeText(text);
    setCopyConfirmed(true);
    setTimeout(() => setCopyConfirmed(false), 2000);
  };

  return (
    <div className="flex flex-col h-full bg-gray-900">
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-700/50 flex-shrink-0">
        <div className="flex items-center gap-2">
          <h1 className="text-sm font-semibold text-gray-100">
            Application Errors
          </h1>
          {errors.length > 0 && (
            <span className="text-xs text-gray-500">{errors.length}</span>
          )}
        </div>
        <button
          onClick={copyToClipboard}
          disabled={errors.length === 0}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-gray-300 bg-gray-700 hover:bg-gray-600 disabled:opacity-40 disabled:cursor-not-allowed rounded transition-colors"
        >
          <CopyIcon />
          {copyConfirmed ? "Copied!" : "Copy All"}
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-3 font-mono text-xs leading-relaxed">
        {errors.length === 0 && (
          <p className="text-gray-500 text-center mt-8">No errors recorded.</p>
        )}
        {errors.map((error, index) => (
          <div key={index} className="mb-3 border border-red-900/40 rounded p-2 bg-red-950/20">
            <div className="flex items-center gap-2 mb-1">
              <span className="text-red-400 font-semibold">{error.command}</span>
              <span className="text-gray-600 text-[10px]">{error.timestamp}</span>
            </div>
            <div className="text-red-300 whitespace-pre-wrap break-all">
              {error.message}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function CopyIcon() {
  return (
    <svg
      className="w-3.5 h-3.5"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
      />
    </svg>
  );
}
