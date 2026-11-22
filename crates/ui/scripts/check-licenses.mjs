import { execFileSync } from "node:child_process";

const ALLOWED_LICENSES = new Set([
  "0BSD",
  "Apache-2.0",
  "BlueOak-1.0.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "CC-BY-4.0",
  "CC0-1.0",
  "ISC",
  "MIT",
  "MPL-2.0",
  "OFL-1.1",
  "Unlicense",
]);
const LICENSE_ALTERNATIVE_PATTERN = /\s+OR\s+/i;

process.exitCode = main();

function main() {
  const packagesByLicense = listPackagesByLicense();
  const rejected = [];
  let packageCount = 0;

  for (const [licenseExpression, packages] of Object.entries(packagesByLicense)) {
    packageCount += packages.length;
    if (isAllowed(licenseExpression)) {
      continue;
    }
    for (const dependency of packages) {
      rejected.push(`${dependency.name}@${dependency.versions.join(", ")}: ${licenseExpression}`);
    }
  }

  console.log(`Licenses: ${packageCount} packages checked, ${rejected.length} rejected`);
  for (const line of rejected) {
    console.log(`  ${line}`);
  }
  return rejected.length > 0 ? 1 : 0;
}

function listPackagesByLicense() {
  const output = execFileSync("pnpm", ["licenses", "list", "--json"], { encoding: "utf8" });
  return JSON.parse(output);
}

function isAllowed(licenseExpression) {
  return licenseExpression
    .replace(/^\(|\)$/g, "")
    .split(LICENSE_ALTERNATIVE_PATTERN)
    .some((alternative) => ALLOWED_LICENSES.has(alternative.trim()));
}
