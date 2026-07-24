import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outputIndex = process.argv.indexOf("--output");
const output = outputIndex >= 0 ? path.resolve(process.cwd(), process.argv[outputIndex + 1]) : path.join(root, "artifacts", "sbom.cdx.json");
const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--format-version", "1", "--locked"], { cwd: path.join(root, "cli"), maxBuffer: 64 * 1024 * 1024 }));
const components = metadata.packages.map((pkg) => ({ type: "library", name: pkg.name, version: pkg.version, purl: `pkg:cargo/${pkg.name}@${pkg.version}`, licenses: pkg.license ? (pkg.license.includes(" OR ") || pkg.license.includes(" AND ") ? [{ expression: pkg.license }] : [{ license: { id: pkg.license } }]) : undefined })).map((component) => Object.fromEntries(Object.entries(component).filter(([, value]) => value !== undefined)));
const rootPackage = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
components.push({ type: "application", name: rootPackage.name, version: rootPackage.version, purl: `pkg:npm/${rootPackage.name}@${rootPackage.version}` });
const bom = { bomFormat: "CycloneDX", specVersion: "1.5", serialNumber: `urn:uuid:${crypto.randomUUID()}`, version: 1, metadata: { timestamp: new Date().toISOString(), tools: [{ vendor: "Antigravity Browser Bridge", name: "generate-sbom.mjs" }] }, components };
fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(output, `${JSON.stringify(bom, null, 2)}\n`, "utf8");
console.log(`SBOM written to ${output} (${components.length} components)`);
