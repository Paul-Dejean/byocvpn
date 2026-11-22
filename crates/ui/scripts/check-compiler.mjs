import { transformSync } from "@babel/core";
import { readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join, relative } from "node:path";

const SOURCE_DIRECTORY = "src";
const SOURCE_FILE_PATTERN = /\.[tj]sx?$/;
const GENERATED_FILES = new Set(["src/bindings.ts"]);
const REACT_CODE_PATTERN = /forwardRef|memo|\b(?:[A-Z]|use[A-Z0-9])/;
const FAILURE_EVENT_KINDS = new Set(["CompileError", "CompileUnexpectedThrow", "PipelineError"]);

const require = createRequire(import.meta.url);
const COMPILER_PLUGIN_PATH = require.resolve("babel-plugin-react-compiler");

process.exitCode = main(process.argv.slice(2));

function main(requestedFiles) {
  const filePaths = requestedFiles.length > 0 ? requestedFiles : listSourceFiles(SOURCE_DIRECTORY);
  let compiledFunctionCount = 0;
  const failures = [];

  for (const filePath of filePaths) {
    const displayPath = relative(process.cwd(), filePath);
    if (GENERATED_FILES.has(displayPath) || !SOURCE_FILE_PATTERN.test(displayPath)) {
      continue;
    }
    const sourceCode = readFileSync(filePath, "utf8");
    if (!REACT_CODE_PATTERN.test(sourceCode)) {
      continue;
    }
    for (const event of collectCompilerEvents(filePath, sourceCode)) {
      if (event.kind === "CompileSuccess") {
        compiledFunctionCount += 1;
      } else if (FAILURE_EVENT_KINDS.has(event.kind)) {
        failures.push(describeFailure(displayPath, event));
      }
    }
  }

  console.log(`React Compiler: ${compiledFunctionCount} functions compiled, ${failures.length} failed`);
  for (const failure of failures) {
    console.log(`  ${failure}`);
  }
  return failures.length > 0 ? 1 : 0;
}

function listSourceFiles(directory) {
  return readdirSync(directory, { recursive: true })
    .filter((entry) => SOURCE_FILE_PATTERN.test(entry))
    .map((entry) => join(directory, entry));
}

function collectCompilerEvents(filePath, sourceCode) {
  const events = [];
  const logger = { logEvent: (_filename, event) => events.push(event) };
  transformSync(sourceCode, {
    filename: filePath,
    configFile: false,
    babelrc: false,
    parserOpts: { plugins: ["typescript", "jsx"] },
    plugins: [[COMPILER_PLUGIN_PATH, { logger }]],
  });
  return events;
}

function describeFailure(displayPath, event) {
  const line = event.fnLoc?.start.line ?? "?";
  const reason = event.detail?.reason ?? event.data?.message ?? event.kind;
  return `${displayPath}:${line} ${reason}`;
}
