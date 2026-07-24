import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const extensionDir = path.join(root, "extension");
const manifestPath = path.join(extensionDir, "manifest.json");
const expectedId = "menkdnglfaljkgofohmhpblgiaehdibc";

const readJson = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
const fail = (message) => {
  console.error(`Extension validation failed: ${message}`);
  process.exitCode = 1;
};

try {
  const manifest = readJson(manifestPath);
  if (manifest.manifest_version !== 3) throw new Error("manifest_version must be 3");
  if (!manifest.key) throw new Error("manifest.key is required for a stable Chrome ID");
  if (manifest.background?.service_worker !== "background.js") {
    throw new Error("background.service_worker must be background.js");
  }
  if (!manifest.permissions?.includes("nativeMessaging")) {
    throw new Error("nativeMessaging permission is missing");
  }

  const digest = crypto.createHash("sha256").update(Buffer.from(manifest.key, "base64")).digest();
  const alphabet = "abcdefghijklmnop";
  const derivedId = [...digest.subarray(0, 16)].map((byte) => alphabet[byte >> 4] + alphabet[byte & 15]).join("");
  if (derivedId !== expectedId) throw new Error(`derived Chrome ID ${derivedId} does not match ${expectedId}`);

  const requiredFiles = [
    "manifest.json",
    "background.js",
    "content.js",
    "popup.html",
    "popup.js",
    "_locales/en/messages.json",
    "_locales/pt_BR/messages.json",
    "icons/icon-16.png",
    "icons/icon-32.png",
    "icons/icon-48.png",
    "icons/icon-128.png",
  ];
  for (const relative of requiredFiles) {
    if (!fs.existsSync(path.join(extensionDir, relative))) throw new Error(`missing ${relative}`);
  }

  if (manifest.action?.default_popup !== "popup.html") {
    throw new Error("action.default_popup must be popup.html");
  }
  if (manifest.default_locale !== "pt_BR") {
    throw new Error("default_locale must be pt_BR");
  }
  readJson(path.join(extensionDir, "_locales", "en", "messages.json"));
  readJson(path.join(extensionDir, "_locales", "pt_BR", "messages.json"));

  for (const source of ["background.js", "content.js", "popup.js"]) {
    const result = spawnSync(process.execPath, ["--check", path.join(extensionDir, source)], { encoding: "utf8" });
    if (result.status !== 0) throw new Error(`${source} syntax check failed: ${result.stderr.trim()}`);
  }

  const hostTemplate = path.join(extensionDir, "native_messaging", "host_manifest.json");
  const host = readJson(hostTemplate);
  if (host.name !== "com.antigravity.agent_browser") throw new Error("native host name is incorrect");
  if (!host.allowed_origins?.some((origin) => origin.includes(expectedId))) {
    throw new Error("native host allowed_origins does not include the stable extension ID");
  }

  console.log(JSON.stringify({ ok: true, extensionId: derivedId, version: manifest.version, files: requiredFiles }, null, 2));
} catch (error) {
  fail(error instanceof Error ? error.message : String(error));
}
