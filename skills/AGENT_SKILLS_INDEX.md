# Agent Skills Index

This index maps each agent to the skills that govern its behavior in this checkout.

| Agent | Skill location | Notes |
|---|---|---|
| Kilo | `~/.kilocode/skills/*` and repo-root `skills/*` | Kilo loads builtins + `~/.kilocode/skills/` |
| Hermes | `skills/dotagents-providers-hermes/*` | OpenCode project doc style |
| Gemini | `skills/dotagents-providers-gemini/*` | OpenCode project doc style |
| Pi | `skills/dotagents-providers-pi/*` | OpenCode project doc style |
| Claude | `packages/agents/dotagents/.dotagents/skills/*` and `~/.claude/skills/*` | Dotagents + OpenCode-compatible `SKILL.md` |
| dotagents / op | `.opencode/skills/cargo-vendormod-*` | repo-local fallback skills |

## Canonical source of truth

- `skills/AGENT_SKILLS_INDEX.md` — this file
- `skills/README.md` — human-readable skill directory map
- `src/bin/skill-importer.rs` — tool to import and regenerate cross-agent skill forms
