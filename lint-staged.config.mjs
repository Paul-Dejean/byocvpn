const FRONTEND_DIRECTORY = "crates/ui";

function quoteFilePaths(stagedFiles) {
  return stagedFiles.map((filePath) => `"${filePath}"`).join(" ");
}

function runInFrontend(command, stagedFiles) {
  return `pnpm --dir ${FRONTEND_DIRECTORY} exec ${command} ${quoteFilePaths(stagedFiles)}`;
}

export default {
  "crates/ui/src/**/*.{ts,tsx}": (stagedFiles) => [
    runInFrontend("oxlint --type-aware --fix", stagedFiles),
    runInFrontend("eslint --fix", stagedFiles),
    runInFrontend("oxfmt", stagedFiles),
  ],
  "*.rs": (stagedFiles) => [
    `rustfmt --edition 2024 ${quoteFilePaths(stagedFiles)}`,
    "cargo clippy --workspace --all-targets -- -D warnings",
  ],
};
