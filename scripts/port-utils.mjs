import { execSync } from "node:child_process";
import process from "node:process";

/**
 * Kill any process currently listening on the specified TCP port.
 */
export function killPort(port) {
  if (!port) return;
  try {
    if (process.platform === "win32") {
      const output = execSync(`netstat -ano -p tcp`, {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "ignore"],
      });
      const lines = output.trim().split("\n");
      const pids = new Set();
      for (const line of lines) {
        const parts = line.trim().split(/\s+/);
        if (
          parts.length >= 5 &&
          parts[1].endsWith(`:${port}`) &&
          parts[3] === "LISTENING"
        ) {
          const pid = parts[4];
          if (pid && pid !== "0" && pid !== String(process.pid)) {
            pids.add(pid);
          }
        }
      }
      for (const pid of pids) {
        try {
          execSync(`taskkill /F /PID ${pid}`, { stdio: "ignore" });
        } catch {}
      }
    } else {
      try {
        execSync(`lsof -ti :${port} | xargs kill -9`, { stdio: "ignore" });
      } catch {}
    }
  } catch {}
}
