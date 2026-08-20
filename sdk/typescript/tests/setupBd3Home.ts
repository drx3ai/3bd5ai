import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { afterEach, beforeEach } from "@jest/globals";

const originalBd3Home = process.env.BD3_HOME;
let currentBd3Home: string | undefined;

beforeEach(async () => {
  currentBd3Home = await fs.mkdtemp(path.join(os.tmpdir(), "bd3-sdk-test-"));
  process.env.BD3_HOME = currentBd3Home;
});

afterEach(async () => {
  const bd3HomeToDelete = currentBd3Home;
  currentBd3Home = undefined;

  if (originalBd3Home === undefined) {
    delete process.env.BD3_HOME;
  } else {
    process.env.BD3_HOME = originalBd3Home;
  }

  if (bd3HomeToDelete) {
    await fs.rm(bd3HomeToDelete, { recursive: true, force: true });
  }
});
