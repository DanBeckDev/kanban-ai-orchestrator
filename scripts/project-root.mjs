import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

export function projectRootFor(moduleUrl, workingDirectory) {
  return moduleUrl.startsWith("file:")
    ? resolve(fileURLToPath(new URL("..", moduleUrl)))
    : workingDirectory;
}
