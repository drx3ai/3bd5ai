export type Bd3ConfigValue = string | number | boolean | Bd3ConfigValue[] | Bd3ConfigObject;

export type Bd3ConfigObject = { [key: string]: Bd3ConfigValue };

export type Bd3Options = {
  bd3PathOverride?: string;
  baseUrl?: string;
  apiKey?: string;
  /**
   * Additional `--config key=value` overrides to pass to the Bd3 CLI.
   *
   * Provide a JSON object and the SDK will flatten it into dotted paths and
   * serialize values as TOML literals so they are compatible with the CLI's
   * `--config` parsing.
   */
  config?: Bd3ConfigObject;
  /**
   * Raw `--config key=value` overrides to pass unchanged to the Bd3 CLI after
   * structured configuration and before SDK-managed or thread-specific overrides.
   */
  configOverrides?: string[];
  /**
   * Environment variables passed to the Bd3 CLI process. When provided, the SDK
   * will not inherit variables from `process.env`.
   */
  env?: Record<string, string>;
};
