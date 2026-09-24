// Cross-platform build wrapper: runs a command while teeing stdout/stderr into
// logs/build-<stamp>.log so build failures leave a trace (win/mac/linux).

import { spawn } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const args = process.argv.slice(2);
const command = args.join(" ");

if (!command) {
  console.error("usage: node build-with-log.mjs <command...>");
  process.exit(2);
}

const logsDir = join(process.cwd(), "logs");
mkdirSync(logsDir, { recursive: true });
const stamp = new Date().toISOString().replace(/[:.]/g, "-");
const logPath = join(logsDir, `build-${stamp}.log`);
const lines = [`$ ${command}`, ""];
writeFileSync(logPath, lines.join("\n"));

const write = (chunk) => {
  const text = chunk.toString();
  process.stdout.write(text);
  writeFileSync(logPath, text, { flag: "a" });
};

// shell:true is required on all platforms: `npm` is a shell script on
// Linux/macOS and a .cmd batch file on Windows — it can't be spawned directly.
const child = spawn(command, { shell: true, stdio: ["inherit", "pipe", "pipe"] });
child.stdout.on("data", write);
child.stderr.on("data", write);
child.on("error", (err) => {
  write(`\n[spawn error] ${err.message}\n`);
  process.exit(1);
});
child.on("close", (code) => {
  write(`\n[exit ${code ?? "signal"}]\n`);
  process.exit(code ?? 1);
});