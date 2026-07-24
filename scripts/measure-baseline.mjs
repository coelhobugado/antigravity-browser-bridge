import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const args = new Set(process.argv.slice(2));
const outputIndex = process.argv.indexOf("--output");
const output = outputIndex >= 0 ? path.resolve(process.cwd(), process.argv[outputIndex + 1]) : path.join(root, "artifacts", "antigravity-baseline.json");
const packageJson = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));

function walkBytes(target) {
  if (!fs.existsSync(target)) return { bytes: 0, files: 0 };
  const stat = fs.statSync(target);
  if (stat.isFile()) return { bytes: stat.size, files: 1 };
  return fs.readdirSync(target, { withFileTypes: true }).reduce((total, entry) => {
    const result = walkBytes(path.join(target, entry.name));
    return { bytes: total.bytes + result.bytes, files: total.files + result.files };
  }, { bytes: 0, files: 0 });
}

function measureDirectory(target) {
  if (process.platform !== "win32" && fs.existsSync(target)) {
    try {
      const output = execFileSync("du", ["-sb", target], { encoding: "utf8" });
      const bytes = Number.parseInt(output.trim().split(/\s+/u)[0], 10);
      if (Number.isFinite(bytes)) return { bytes, files: null, method: "du -sb" };
    } catch {
      // Fall back to a portable recursive walk below.
    }
  }
  return walkBytes(target);
}

function measureStartup(executable) {
  const start = process.hrtime.bigint();
  let peakRssBytes = null;
  let result;
  const timeBinary = process.platform !== "win32" && fs.existsSync("/usr/bin/time") ? "/usr/bin/time" : null;
  let rssFile;
  if (timeBinary) {
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "antigravity-baseline-"));
    rssFile = path.join(tempDir, "rss");
    result = spawnSync(timeBinary, ["-f", "%M", "-o", rssFile, executable, "--help"], { cwd: root, stdio: "ignore", windowsHide: true });
    if (fs.existsSync(rssFile)) {
      const rssKiB = Number.parseInt(fs.readFileSync(rssFile, "utf8").trim(), 10);
      if (Number.isFinite(rssKiB)) peakRssBytes = rssKiB * 1024;
    }
    fs.rmSync(tempDir, { recursive: true, force: true });
  } else {
    result = spawnSync(executable, ["--help"], { cwd: root, stdio: "ignore", windowsHide: true });
  }
  const elapsedMs = Number(process.hrtime.bigint() - start) / 1e6;
  if (result.error || result.status !== 0) return { executable, ok: false, elapsedMs: Math.round(elapsedMs * 100) / 100, peakRssBytes: null, error: result.error?.message ?? `exit ${result.status}` };
  return { executable, ok: true, elapsedMs: Math.round(elapsedMs * 100) / 100, peakRssBytes, memoryNote: peakRssBytes === null ? "This platform does not expose a portable child-process RSS API." : "Measured with /usr/bin/time -f %M." };
}

function runtimeBaseline() {
  const candidates = process.platform === "win32"
    ? ["bin/agent-browser-win32-x64.exe", "cli/target/release/agent-browser.exe", "cli/target/debug/agent-browser.exe"]
    : ["bin/agent-browser", "bin/agent-browser-linux-x64", "cli/target/release/agent-browser", "cli/target/debug/agent-browser"];
  const relative = candidates.find((candidate) => {
    const file = path.join(root, candidate);
    return fs.existsSync(file) && fs.statSync(file).isFile() && fs.statSync(file).size > 0;
  });
  return relative ? measureStartup(path.join(root, relative)) : { ok: false, executable: null, elapsedMs: null, peakRssBytes: null, reason: "no built agent-browser executable found; run the release baseline job" };
}

const binaryCandidates = process.platform === "win32"
  ? ["bin/agent-browser-win32-x64.exe", "cli/target/release/agent-browser.exe", "cli/target/debug/agent-browser.exe"]
  : ["bin/agent-browser", "bin/agent-browser-linux-x64", "cli/target/release/agent-browser", "cli/target/debug/agent-browser"];
const binaries = Object.fromEntries(binaryCandidates.map((relative) => {
  const file = path.join(root, relative);
  return [relative, fs.existsSync(file) && fs.statSync(file).isFile() && fs.statSync(file).size > 0 ? walkBytes(file) : { bytes: 0, files: 0, present: false }];
}));
const extensionZip = path.join(root, "artifacts", "antigravity-browser-bridge-extension.zip");

const tracked = execFileSync("git", ["ls-files", "-z"], { cwd: root }).toString().split("\0").filter(Boolean);
const trackedBytes = tracked.reduce((total, relative) => total + walkBytes(path.join(root, relative)).bytes, 0);
const extension = walkBytes(path.join(root, "extension"));
const cacheDirectories = ["cli/target", "node_modules", "packages/dashboard/node_modules", ".pnpm-store", "artifacts"];
const caches = Object.fromEntries(cacheDirectories.map((relative) => {
  const target = path.join(root, relative);
  return [relative, args.has("--include-caches") ? measureDirectory(target) : { present: fs.existsSync(target), bytes: null, files: null }];
}));
const result = {
  schemaVersion: 1,
  generatedAt: new Date().toISOString(),
  packageVersion: packageJson.version,
  tracked: { files: tracked.length, bytes: trackedBytes },
  extension,
  extensionZip: fs.existsSync(extensionZip) ? walkBytes(extensionZip) : { bytes: 0, files: 0, present: false },
  binaries,
  runtime: args.has("--measure-runtime") ? runtimeBaseline() : { ok: false, executable: null, elapsedMs: null, peakRssBytes: null, reason: "runtime measurement not requested" },
  caches,
  budgets: {
    trackedBytes: 150 * 1024 * 1024,
    extensionBytes: 5 * 1024 * 1024,
    cacheBytes: null
  },
  note: "Cache directories are diagnostic only and are never included in a release artifact."
};
fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(output, `${JSON.stringify(result, null, 2)}\n`, "utf8");
console.log(JSON.stringify(result, null, 2));
if (args.has("--check") && (trackedBytes > result.budgets.trackedBytes || extension.bytes > result.budgets.extensionBytes)) {
  console.error("Baseline size budget exceeded");
  process.exitCode = 1;
}
