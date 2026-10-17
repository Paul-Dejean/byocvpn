import { transformSync } from "@babel/core";
import { reactCompilerPreset } from "@vitejs/plugin-react";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const SOURCE_DIRECTORY = "src";
const SOURCE_FILE_PATTERN = /\.[tj]sx?$/;
const FAILURE_EVENT_KINDS = new Set(["CompileError", "CompileUnexpectedThrow", "PipelineError"]);

process.exitCode = main();

function main() {
  const buildPreset = reactCompilerPreset();
  const [compilerPluginPath, compilerOptions] = buildPreset.preset().plugins[0];
  const codeFilter = buildPreset.rolldown.filter.code;

  let compiledFunctionCount = 0;
  const failures = [];

  for (const filePath of listSourceFiles(SOURCE_DIRECTORY)) {
    const sourceCode = readFileSync(filePath, "utf8");
    if (!codeFilter.test(sourceCode)) {
      continue;
    }
    const events = collectCompilerEvents(filePath, sourceCode, compilerPluginPath, compilerOptions);
    for (const event of events) {
      if (event.kind === "CompileSuccess") {
        compiledFunctionCount += 1;
      } else if (FAILURE_EVENT_KINDS.has(event.kind)) {
        failures.push(describeFailure(filePath, event));
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

function collectCompilerEvents(filePath, sourceCode, compilerPluginPath, compilerOptions) {
  const events = [];
  const logger = { logEvent: (_filename, event) => events.push(event) };
  transformSync(sourceCode, {
    filename: filePath,
    configFile: false,
    babelrc: false,
    parserOpts: { plugins: ["typescript", "jsx"] },
    plugins: [[compilerPluginPath, { ...compilerOptions, logger }]],
  });
  return events;
}

function describeFailure(filePath, event) {
  const line = event.fnLoc?.start.line ?? "?";
  const reason = event.detail?.reason ?? event.data?.message ?? event.kind;
  return `${filePath}:${line} ${reason}`;
}
