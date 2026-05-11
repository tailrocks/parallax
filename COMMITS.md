# Commits

This file covers commit message format and AI-agent attribution.

## Commit Messages

Use [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/).

Subject format:

```text
<type>[optional scope][!]: <description>
```

Allowed types:

| Type | Use for |
| --- | --- |
| `feat` | New user-visible feature |
| `fix` | Bug fix |
| `docs` | Documentation-only change |
| `style` | Formatting, whitespace; no content or logic change |
| `refactor` | Restructuring without behavior change |
| `perf` | Performance improvement |
| `test` | Adding or updating tests |
| `build` | Build system, tooling, or dependencies |
| `ci` | CI configuration |
| `chore` | Routine maintenance |
| `revert` | Reverts a prior commit |

Keep subjects short and imperative:

```text
docs: add market landscape
```

Use a body when the reason is not obvious from the subject, when the change
rewrites history, or when the commit captures a strategic decision.

## Agent Attribution

Every AI-authored commit must include exactly one `Co-authored-by` trailer for
the agent that created the commit:

```text
Co-authored-by: Codex <codex@openai.com>
Co-authored-by: Claude <noreply@anthropic.com>
Co-authored-by: Amp <amp@ampcode.com>
```

Use only one of those on a normal commit. If the agent is unclear, ask before
committing.

## Research Commits

Research commits should name the artifact, not the conversation:

```text
docs: add market landscape
docs: update project structure
docs: record Datadog comparison
```

When findings are time-sensitive, include dates and source links in the Markdown
file itself rather than stuffing them into the commit message.
