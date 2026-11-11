/** Flags that take no value. */
const SWITCHES = new Set(["help", "dryRun", "keepHistory", "force"]);

/** Parsed flags: a switch is `true`, anything else its value. */
export interface Flags {
  help?: true;
  dryRun?: true;
  keepHistory?: true;
  force?: true;
  org?: string;
  repo?: string;
  serverId?: string;
  title?: string;
  summary?: string;
  [flag: string]: string | true | undefined;
}

/**
 * `--name value` pairs into an object, `--server-id` as `serverId`. A flag with no value, or
 * an argument that is not a flag, is an error rather than a guess.
 */
export function parseArgs(argv: readonly string[]): Flags {
  const flags: Flags = {};
  for (let at = 0; at < argv.length; at += 1) {
    const arg = argv[at]!;
    if (!arg.startsWith("--")) throw new Error(`Unexpected argument "${arg}".`);
    const key = arg.slice(2).replace(/-([a-z])/g, (_, letter: string) => letter.toUpperCase());
    if (SWITCHES.has(key)) {
      flags[key] = true;
      continue;
    }
    const value = argv[at + 1];
    if (value === undefined || value.startsWith("--")) throw new Error(`${arg} needs a value.`);
    flags[key] = value;
    at += 1;
  }
  return flags;
}
