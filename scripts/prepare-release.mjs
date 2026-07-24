import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const output = path.join(root, "artifacts", "release");
fs.rmSync(output, { recursive: true, force: true });
fs.mkdirSync(output, { recursive: true });
const run = (command, args) => execFileSync(command, args, { cwd: root, stdio: "inherit" });
run(process.execPath, ["scripts/validate-extension.mjs"]);
run(process.execPath, ["scripts/check-encoding.mjs"]);
run(process.execPath, ["scripts/validate-evals.mjs"]);
run("python", ["scripts/package-extension.py", "--output", path.join(output, "antigravity-browser-bridge-extension.zip")]);
run(process.execPath, ["scripts/generate-sbom.mjs", "--output", path.join(output, "sbom.cdx.json")]);
run(process.execPath, ["scripts/measure-baseline.mjs", "--measure-runtime", "--output", path.join(output, "baseline.json")]);
console.log(`Release artifacts prepared in ${output}`);
