/*
 * What a new repository's identity is, worked out from the flags. Pure: values in, values out.
 */

/** `my-repo` is `My Repo`. */
function titleCase(name) {
  return name
    .split(/[^A-Za-z0-9]+/)
    .filter(Boolean)
    .map((word) => word[0].toUpperCase() + word.slice(1))
    .join(" ");
}

/**
 * The full identity for `org`/`repo`. `serverId`, `title` and `summary` default from them.
 *
 * `serverId` defaults to the first word of the org and the repo: `LXAgents-Skills` and
 * `security` give `lxagents-security`, the way the existing servers are named.
 *
 * @param {{ org: string, repo: string, serverId?: string, title?: string, summary?: string }} flags
 */
export function identityFor({ org, repo, serverId, title, summary }) {
  if (!org || !repo) throw new Error("Both --org and --repo are required.");
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(org) || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(repo)) {
    throw new Error("--org and --repo take GitHub names: letters, digits, '.', '_' and '-'.");
  }

  const brand = org.split(/[^A-Za-z0-9]+/)[0];
  return {
    org,
    repo,
    serverId: serverId ?? `${brand}-${repo}`.toLowerCase(),
    title: title ?? `${brand} ${titleCase(repo)}`,
    summary: summary ?? `The ${repo} instruction set, served read-only.`,
  };
}
