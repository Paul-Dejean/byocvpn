import { spawnSync } from "node:child_process";

const RELEASE_CONFIG_BY_PLATFORM = {
  darwin: "tauri.macos.release.conf.json",
};

const releaseConfiguration = RELEASE_CONFIG_BY_PLATFORM[process.platform];
const configurationArguments = releaseConfiguration
  ? ["--config", releaseConfiguration]
  : [];

const buildResult = spawnSync(
  "tauri",
  ["build", ...configurationArguments, ...process.argv.slice(2)],
  { stdio: "inherit", shell: process.platform === "win32" },
);

process.exit(buildResult.status ?? 1);
