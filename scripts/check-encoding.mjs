import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const textExtensions = new Set([".js", ".mjs", ".json", ".md", ".mdx", ".rs", ".toml", ".yml", ".yaml", ".ps1", ".py", ".ts", ".tsx", ".html", ".css"]);
const files = execFileSync("git", ["ls-files", "-z"], { cwd: root }).toString().split("\0").filter(Boolean).filter((file) => textExtensions.has(path.extname(file).toLowerCase())).filter((file) => file !== "cli/src/native/a11y/axe.min.js");
const mojibake = /\u00c3[\u0080-\u00bf]|\u00c2[\u0080-\u00bf]|\u00e2[\u0080-\u00bf]|\ufffd/u;
const failures = [];
for (const relative of files) {
  const file = path.join(root, relative);
  const bytes = fs.readFileSync(file);
  let content;
  try {
    content = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    failures.push(`${relative}: invalid UTF-8`);
    continue;
  }
  if (mojibake.test(content)) failures.push(`${relative}: suspicious mojibake sequence`);
}
if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}
console.log(`Encoding check passed for ${files.length} tracked text files`);
