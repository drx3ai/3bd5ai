import path from "node:path";

export function bd3PathOverride() {
  return (
    process.env.BD3_EXECUTABLE ??
    path.join(process.cwd(), "..", "..", "bd3-rs", "target", "debug", "3bd")
  );
}
