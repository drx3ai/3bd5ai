import { Bd3Options } from "./3bdOptions";
import { Bd3Exec } from "./exec";
import { Thread } from "./thread";
import { ThreadOptions } from "./threadOptions";

/**
 * Bd3 is the main class for interacting with the Bd3 agent.
 *
 * Use the `startThread()` method to start a new thread or `resumeThread()` to resume a previously started thread.
 */
export class Bd3 {
  private exec: Bd3Exec;
  private options: Bd3Options;

  constructor(options: Bd3Options = {}) {
    const { bd3PathOverride, env, config, configOverrides } = options;
    this.exec = new Bd3Exec(bd3PathOverride, env, config, configOverrides);
    this.options = options;
  }

  /**
   * Starts a new conversation with an agent.
   * @returns A new thread instance.
   */
  startThread(options: ThreadOptions = {}): Thread {
    return new Thread(this.exec, this.options, options);
  }

  /**
   * Resumes a conversation with an agent based on the thread id.
   * Threads are persisted in ~/.bd3/sessions.
   *
   * @param id The id of the thread to resume.
   * @returns A new thread instance.
   */
  resumeThread(id: string, options: ThreadOptions = {}): Thread {
    return new Thread(this.exec, this.options, options, id);
  }
}
