# dotagents-providers-hermes

Deploy dotfiles for Hermes using the Hermes provider in dotagents. Use when configuring Hermes agent commands, skills, or instructions.

## Instructions

# Dotagents Providers — Hermes

## Deploy Hermes

```bash
dotagents deploy
```

## Add Hermes Target

```toml
targets = ["claude", "codex", "opencode", "hermes"]
```

## Provider Template Reference

- `public/v1/templates/hermes/`

## Examples

- Run the skill workflow as documented in the source.
