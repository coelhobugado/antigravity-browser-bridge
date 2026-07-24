import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const file = path.join(root, "evals", "antigravity-baseline.json");
const data = JSON.parse(fs.readFileSync(file, "utf8"));
if (!Array.isArray(data.tasks) || data.tasks.length !== 20) throw new Error("baseline must contain exactly 20 tasks");
const required = ["id", "category", "risk", "requiresApproval", "preconditions", "action", "evidence", "result"];
const ids = new Set();
for (const task of data.tasks) {
  for (const key of required) if (!(key in task)) throw new Error(`${task.id ?? "unknown"} missing ${key}`);
  if (ids.has(task.id)) throw new Error(`duplicate task id: ${task.id}`);
  ids.add(task.id);
  if (!task.result?.status || !task.result?.reason) throw new Error(`${task.id} needs a current result status and reason`);
}
console.log(`Evaluation baseline valid: ${data.tasks.length} tasks`);
