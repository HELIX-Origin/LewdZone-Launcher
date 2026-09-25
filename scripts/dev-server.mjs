import { spawn } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { killPort } from "./port-utils.mjs";

const PORT = 24240;

async function main() {
  // Free any previous stale process on port 24240
  killPort(PORT);

  process.env.VITE_PORT = String(PORT);
  process.env.PORT = String(PORT);

  console.log(`[dev-server] Starting Vite dev server on dedicated port ${PORT}...`);

  const viteScript = path.resolve("node_modules/vite/bin/vite.js");

  const vite = spawn(
    process.execPath,
    [viteScript, "dev", "--port", String(PORT), "--strictPort"],
    {
      stdio: "inherit",
      env: { ...process.env, VITE_PORT: String(PORT), PORT: String(PORT) },
      shell: false,
    }
  );

  let cleanedUp = false;
  function cleanup() {
    if (cleanedUp) return;
    cleanedUp = true;
    console.log(`[dev-server] Shutting down dev server and freeing port ${PORT}...`);
    try {
      vite.kill("SIGTERM");
    } catch {}
    killPort(PORT);
  }

  process.on("SIGINT", () => {
    cleanup();
    process.exit(0);
  });
  process.on("SIGTERM", () => {
    cleanup();
    process.exit(0);
  });
  process.on("exit", cleanup);

  vite.on("exit", (code) => {
    cleanup();
    process.exit(code ?? 0);
  });
}

main().catch((err) => {
  console.error("[dev-server] Failed to start:", err);
  process.exit(1);
});
