import { readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const ASSETS_DIRECTORY = "dist/assets";
const KIBIBYTE = 1024;
const SIZE_BUDGETS_IN_KIBIBYTES = [
  { label: "JavaScript", filePattern: /\.js$/, budget: 560 },
  { label: "CSS", filePattern: /\.css$/, budget: 64 },
];

process.exitCode = main();

function main() {
  const assetFiles = readdirSync(ASSETS_DIRECTORY);
  let overBudgetCount = 0;

  for (const { label, filePattern, budget } of SIZE_BUDGETS_IN_KIBIBYTES) {
    const totalBytes = assetFiles
      .filter((fileName) => filePattern.test(fileName))
      .reduce((sum, fileName) => sum + statSync(join(ASSETS_DIRECTORY, fileName)).size, 0);
    const totalKibibytes = totalBytes / KIBIBYTE;
    const verdict = totalKibibytes <= budget ? "ok" : "OVER BUDGET";
    console.log(`${label}: ${totalKibibytes.toFixed(1)} KiB (budget ${budget} KiB) ${verdict}`);
    if (totalKibibytes > budget) {
      overBudgetCount += 1;
    }
  }

  return overBudgetCount > 0 ? 1 : 0;
}
