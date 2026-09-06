# AI-assisted development

The root [AGENTS.md](../AGENTS.md) is the concise repository instruction entry point.
The specification owns behavior, ADRs own conforming decisions, the implementation
plan owns work-package order, and the status ledger owns current evidence. Keep facts
in those sources rather than copying them into every prompt or skill.

Codex discovers repository skills under `.agents/skills`; each entry point uses `name`
and `description` frontmatter. Root AGENTS.md supplies shared instructions. These choices
follow the official [skill documentation](https://learn.chatgpt.com/docs/build-skills)
and [AGENTS.md documentation](https://learn.chatgpt.com/docs/agent-configuration/agents-md),
checked on 6 September 2026. Skills are instruction-only and need no connector or model
configuration. If new skills are not visible in an existing session, restart it.

| Skill | Use | Output |
| --- | --- | --- |
| [chart-work-package](../.agents/skills/chart-work-package/SKILL.md) | Implement or plan an assigned WP slice | Bounded scope, contract changes, appropriate evidence, ledger and next action. |
| [chart-contract-review](../.agents/skills/chart-contract-review/SKILL.md) | Review behavior or readiness against requirements | Source-backed findings and Pass/Fail/Uncertain evidence; no implicit edits. |

Example routing checks:

- “Prepare infrastructure only” permits manifests/docs/scripts and leaves semantic
  contracts and G0 open, even though the general library plan authorizes later coding.
- “Implement WP-04” first checks WP-02 acceptance; package directories alone do not
  satisfy its prerequisite. Preserve unrelated changes and report missing contracts.
- “Review quantile behavior” selects GRA-04/FIX-04 and independent expectations; it does
  not edit the implementation or demand a GPUI screenshot for a pure numerical result.
- “Certify binding parity” requires actual Python/WASM execution, ownership and large-ID
  cases; target compilation alone yields Uncertain for runtime parity.

These are manual routing/behavioral walkthroughs, not results from a launched independent
agent. Frontmatter validation checks discoverability structure, not future agent behavior.

Add a new skill only for a distinct repeated workflow. Keep provider-specific metadata
optional; no global configuration changes, automatic delegation, model pinning, paid
API calls, approval bypasses or duplicated instruction files are part of this setup.
