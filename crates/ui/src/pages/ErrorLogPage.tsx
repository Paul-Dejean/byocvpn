import { useErrorLogContext } from "../contexts";

export function ErrorLogPage() {
  const { entries, clearEntries } = useErrorLogContext();

  return (
    <div className="flex flex-col h-full bg-gray-900 text-white">
      <div className="flex items-center justify-between px-6 py-4 border-b border-gray-700/60 flex-shrink-0">
        <h1 className="text-sm font-semibold text-gray-300 uppercase tracking-wider">Error Log</h1>
        {entries.length > 0 && (
          <button
            onClick={clearEntries}
            className="text-xs px-3 py-1.5 bg-gray-700 hover:bg-gray-600 text-gray-300 hover:text-white rounded-lg transition-colors"
          >
            Clear
          </button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {entries.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full gap-3 text-gray-500">
            <svg xmlns="http://www.w3.org/2000/svg" className="h-10 w-10 opacity-40" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p className="text-sm">No errors recorded</p>
          </div>
        ) : (
          <div className="max-w-2xl mx-auto space-y-2">
            {entries.map((entry) => (
              <div key={entry.id} className="bg-gray-800 rounded-lg p-4 border border-gray-700/50">
                <div className="flex items-center gap-2 mb-1.5">
                  <span className="text-xs font-mono text-gray-500">
                    {entry.timestamp.toLocaleTimeString()}
                  </span>
                  <span className="text-xs px-2 py-0.5 bg-gray-700 text-gray-300 rounded-full">
                    {entry.source}
                  </span>
                </div>
                <p className="text-sm text-red-300 leading-relaxed">{entry.message}</p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
