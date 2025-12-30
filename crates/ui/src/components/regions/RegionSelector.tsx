import { useInstancesContext, useRegionsContext } from "../../contexts";
import { AwsRegion } from "../../types";
import { useState } from "react";

interface RegionSelectorProps {
  onClose: () => void;
}

export function RegionSelector({ onClose }: RegionSelectorProps) {
  const [selectedRegion, setSelectedRegion] = useState<AwsRegion | null>(null);
  const { groupedRegions } = useRegionsContext();
  const { spawnInstance, instances } = useInstancesContext();

  const handleDeploy = () => {
    if (selectedRegion) {
      spawnInstance(selectedRegion.name);
      onClose(); // Close immediately to show placeholder card
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900">
      {/* Header with Back Button */}
      <div className="bg-gray-800 p-6 border-b border-gray-700">
        <div className="flex items-center gap-4">
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-700 rounded-lg transition"
          >
            <svg
              className="w-6 h-6 text-gray-300"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 19l-7-7 7-7"
              />
            </svg>
          </button>
          <div>
            <h1 className="text-3xl font-bold text-blue-400">
              Deploy New Server
            </h1>
            <p className="text-gray-300 mt-1">
              Select a region to deploy your VPN server
            </p>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="space-y-6">
          {groupedRegions.map((group, idx) => (
            <div key={idx}>
              <h3 className="text-xs uppercase text-gray-400 font-semibold mb-3 px-2">
                {group.continent}
              </h3>
              <div className="grid grid-cols-2 gap-3">
                {group.regions.map((region) => (
                  <button
                    key={region.name}
                    onClick={() => setSelectedRegion(region)}
                    className={`p-4 bg-gray-800 border rounded-lg transition-all text-left ${
                      selectedRegion?.name === region.name
                        ? "border-blue-500 bg-blue-900/20"
                        : "border-gray-700 hover:border-gray-600 hover:bg-gray-700"
                    }`}
                  >
                    <div className="flex items-center gap-3 mb-2">
                      <span className="text-2xl">{(region as any).flag}</span>
                      <div>
                        <p className="font-medium text-sm">{region.name}</p>
                        <p className="text-xs text-gray-400">
                          {region.country}
                        </p>
                      </div>
                    </div>
                    {instances.filter(
                      (instance) => instance.region === region.name
                    ).length > 0 && (
                      <div className="flex items-center gap-1 text-xs text-gray-400">
                        <svg
                          className="w-3 h-3"
                          fill="currentColor"
                          viewBox="0 0 20 20"
                        >
                          <path
                            fillRule="evenodd"
                            d="M2 5a2 2 0 012-2h12a2 2 0 012 2v10a2 2 0 01-2 2H4a2 2 0 01-2-2V5zm3.293 1.293a1 1 0 011.414 0l3 3a1 1 0 010 1.414l-3 3a1 1 0 01-1.414-1.414L7.586 10 5.293 7.707a1 1 0 010-1.414zM11 12a1 1 0 100 2h3a1 1 0 100-2h-3z"
                            clipRule="evenodd"
                          />
                        </svg>
                        <span>
                          {
                            instances.filter(
                              (instance) => instance.region === region.name
                            ).length
                          }{" "}
                          active
                        </span>
                      </div>
                    )}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Deploy Button - Fixed at bottom */}
        {selectedRegion && (
          <div className="sticky bottom-0 bg-gray-900 border-t border-gray-700 p-6">
            <div className="max-w-4xl mx-auto">
              <button
                onClick={handleDeploy}
                className="w-full px-6 py-4 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition font-medium text-lg shadow-lg hover:shadow-xl flex items-center justify-center gap-2"
              >
                <svg
                  className="w-5 h-5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                Deploy Server in {selectedRegion.name}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
