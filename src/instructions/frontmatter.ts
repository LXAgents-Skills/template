/**
 * Pull the single-line scalar fields out of a markdown file's frontmatter.
 *
 * Only `key: value` on one line is read. A folded or multi-line YAML value would need a real
 * parser, and a dependency added for one call site is not worth it; a `description:` this
 * misses is reported as missing at boot rather than served empty.
 *
 * The value is matched as `\S.*` rather than `.*`, so the run of spaces before it and the
 * value itself are different character classes and a line that fails to match costs one pass.
 *
 * @returns The fields, or `null` when there is no frontmatter.
 */
export function parseFrontmatter(text: string): Record<string, string> | null {
  const block = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!block) return null;

  const fields: Record<string, string> = {};
  for (const line of block[1]!.split(/\r?\n/)) {
    const field = line.match(/^([A-Za-z_][A-Za-z0-9_]*):[ \t]*(\S.*|)$/);
    if (field) fields[field[1]!] = field[2]!.trim();
  }
  return fields;
}
