import { readdirSync, readFileSync } from "node:fs";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { join, relative, sep } from "node:path";
import { CONTENT_DIR } from "../paths.js";
import { parseFrontmatter } from "./frontmatter.js";
import { OVERRIDE_BASES, nameProblem, toolNameFor } from "./naming.js";

/*
 * The tool surface, derived from `content/`: one tool per `.md` file.
 *
 * Adding a markdown file to `content/` is adding a tool. There is no tool module to write and
 * no list to edit. Everything is read and checked here, once, at import, so a malformed set
 * stops the process instead of answering the first caller wrongly.
 */

/** Every `.md` under `dir`, as set-relative forward-slash paths, sorted. */
export function markdownFiles(dir: string = CONTENT_DIR, base: string = dir): string[] {
  const found: string[] = [];

  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) found.push(...markdownFiles(full, base));
    else if (entry.name.endsWith(".md")) found.push(relative(base, full).split(sep).join("/"));
  }

  return found.sort((a, b) => a.localeCompare(b));
}

/** One served file: the tool's name and description, and the handler that returns the file. */
export interface ContentInstruction {
  config: { name: string; description: string };
  handler: () => Promise<CallToolResult>;
}

/** The surface, and which file each tool name serves. */
export interface ContentInstructions {
  tools: ContentInstruction[];
  files: Map<string, string>;
}

/**
 * Build the surface from a set directory.
 *
 * Each handler returns its file whole, frontmatter included, byte for byte. No handler takes
 * an argument, so there is no path for a caller to traverse with.
 */
export function buildContentInstructions(
  dir: string = CONTENT_DIR,
  overrides: Readonly<Record<string, string>> = OVERRIDE_BASES,
): ContentInstructions {
  const paths = markdownFiles(dir);
  const tools: ContentInstruction[] = [];
  const files = new Map<string, string>();

  for (const key of Object.keys(overrides)) {
    if (!paths.includes(key)) {
      throw new Error(`OVERRIDE_BASES names ${key}, which is not in the set. Remove the override or restore the file.`);
    }
  }

  for (const path of paths) {
    const name = toolNameFor(path, overrides);

    const problem = nameProblem(name);
    if (problem) {
      throw new Error(`${path}: ${problem}. Shorten its base name in OVERRIDE_BASES (src/instructions/naming.ts).`);
    }
    if (files.has(name)) {
      throw new Error(`${path} and ${files.get(name)} both derive "${name}". Give one an entry in OVERRIDE_BASES.`);
    }

    const text = readFileSync(join(dir, path), "utf8");
    const description = parseFrontmatter(text)?.description;
    if (!description) {
      throw new Error(`${path} has no frontmatter \`description:\`. It would publish a tool no client can route on.`);
    }

    files.set(name, path);
    tools.push({
      config: { name, description },
      handler: async () => ({ content: [{ type: "text", text }] }),
    });
  }

  if (tools.length === 0) {
    throw new Error(`No markdown files under ${dir}. This server would expose nothing.`);
  }

  return { tools, files };
}

const { tools, files } = buildContentInstructions();

export const CONTENT_INSTRUCTIONS: readonly ContentInstruction[] = Object.freeze(tools);

/** Tool name -> the set-relative file it serves. The tests pin both directions. */
export const INSTRUCTION_FILES: Map<string, string> = files;
