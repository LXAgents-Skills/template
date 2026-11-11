/*
 * The text edits `init` makes. Pure: each takes a file's text and returns the new text.
 */

/** A JavaScript string literal for `value`. */
const literal = (value) => JSON.stringify(value);

/**
 * Set one `export const NAME = ...;` in `src/identity.js`. The value may span lines; it ends at
 * the first `;` that closes the statement.
 */
function setConstant(source, name, value) {
  const pattern = new RegExp(`(export const ${name} =)[\\s\\S]*?;`);
  if (!pattern.test(source)) throw new Error(`src/identity.js has no "export const ${name}".`);
  return source.replace(pattern, `$1 ${literal(value)};`);
}

/** `src/identity.js` with the five identity constants replaced. */
export function rewriteIdentity(source, identity) {
  return [
    ["ORG_NAME", identity.org],
    ["REPO_NAME", identity.repo],
    ["SERVER_ID", identity.serverId],
    ["SERVER_TITLE", identity.title],
    ["SERVER_SUMMARY", identity.summary],
  ].reduce((text, [name, value]) => setConstant(text, name, value), source);
}

/**
 * `package.json` for the new repository: its name, bins and links, and version `0.0.0`. The
 * first release of the new repository sets a real version.
 */
export function rewritePackage(source, identity) {
  const pkg = JSON.parse(source);
  const slug = `${identity.org}/${identity.repo}`;
  pkg.name = `@${identity.org.toLowerCase()}/${identity.repo}`;
  pkg.version = "0.0.0";
  pkg.description = identity.summary;
  pkg.bin = { [identity.serverId]: "src/cli.js", [`${identity.serverId}-server`]: "src/index.js" };
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
export function replaceAll(text, pairs) {
  return pairs.reduce((out, [from, to]) => (from === to ? out : out.split(from).join(to)), text);
}

/**
 * Drop every `<!-- template-only -->` … `<!-- /template-only -->` block, markers included,
 * with the line break after it. Text that is true only of the template itself lives there.
 */
export function stripTemplateOnly(text) {
  return text.replace(/<!-- template-only -->[\s\S]*?<!-- \/template-only -->\r?\n?/g, "");
}
