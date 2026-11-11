---
name: example-guide
description: An example served file — shows the frontmatter and shape every file in content/ takes. Replace it with your own set.
---

# Example Guide

This file is served as the tool `lxagents_skills_template_example_guide`. Its ID comes from
its filename, `example-guide.md`, behind the repository's `{org}_{repo}` prefix.

Every file in `content/` follows the same shape:

* **Frontmatter** with a one-line `name` and a one-line `description`. The `description` is
  what a client reads in its tool list to decide whether to call the tool, so write it as
  the condition and the payoff, not as a title.
* **One subject.** A tool is complete on its own and points to no other tool. If it needs a
  fact from another file, it states that fact in a sentence of its own.
* **A row in `automation.md`**, naming this tool's ID and the one condition that activates
  it. The test suite fails if the row is missing.

Delete this file when the repository has its own content, and remove its row from
`automation.md` in the same commit.
