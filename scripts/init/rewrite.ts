import type { Identity } from "./identity.js";

/*
 * The text edits `init` makes. Pure: each takes a file's text and returns the new text.
 */

/** A JavaScript string literal for `value`. */
const literal = (value: string): string => JSON.stringify(value);

/**
 * Set one `export const NAME = ...;` in `src/identity.ts`. The value may span lines; it ends at
 * the first `;` that closes the statement.
 */
function setConstant(source: string, name: string, value: string): string {
  const pattern = new RegExp(`(export const ${name} =)[\\s\\S]*?;`);
  if (!pattern.test(source)) throw new Error(`src/identity.ts has no "export const ${name}".`);
  return source.replace(pattern, `$1 ${literal(value)};`);
}

/** `src/identity.ts` with the five identity constants replaced. */
export function rewriteIdentity(source: string, identity: Identity): string {
  const constants: [string, string][] = [
    ["ORG_NAME", identity.org],
    ["REPO_NAME", identity.repo],
    ["SERVER_ID", identity.serverId],
    ["SERVER_TITLE", identity.title],
    ["SERVER_SUMMARY", identity.summary],
  ];
  return constants.reduce((text, [name, value]) => setConstant(text, name, value), source);
}

/**
 * `package.json` for the new repository: its name, bins and links, and version `0.0.0`. The
 * first release of the new repository sets a real version.
 */
export function rewritePackage(source: string, identity: Identity): string {
  const pkg = JSON.parse(source) as Record<string, unknown> & { keywords?: string[] };
  const slug = `${identity.org}/${identity.repo}`;
  pkg.name = `@${identity.org.toLowerCase()}/${identity.repo}`;
  pkg.version = "0.0.0";
  pkg.description = identity.summary;
  pkg.bin = { [identity.serverId]: "dist/cli.js", [`${identity.serverId}-server`]: "dist/index.js" };
  pkg.repository = { type: "git", url: `git+https://github.com/${slug}.git` };
  pkg.homepage = `https://github.com/${slug}#readme`;
  pkg.bugs = { url: `https://github.com/${slug}/issues` };
  pkg.keywords = (pkg.keywords ?? []).filter((word) => word !== "template");
  return `${JSON.stringify(pkg, null, 2)}\n`;
}

/**
 * Replace every whole occurrence of each `[from, to]` pair, in order. Used for the tool
 * prefix, the `org/repo` slug and the server ID in markdown and the Dockerfile.
 */
export function replaceAll(text: string, pairs: readonly (readonly [string, string])[]): string {
  return pairs.reduce((out, [from, to]) => (from === to ? out : out.split(from).join(to)), text);
}

/**
 * Drop every `<!-- template-only -->` … `<!-- /template-only -->` block, markers included,
 * with the line break after it. Text that is true only of the template itself lives there.
 */
export function stripTemplateOnly(text: string): string {
  return text.replace(/<!-- template-only -->[\s\S]*?<!-- \/template-only -->\r?\n?/g, "");
}
