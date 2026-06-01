---
name: dotagents-providers-pi
description: Deploy dotfiles for Pi using the Pi provider in dotagents. Use when configuring Pi agent commands, skills, or instructions.
license: MIT
compatibility: cross-agent
metadata:
  imported: true
  source: ./skills/dotagents-providers-pi/SKILL.md
---

# Dotagents Providers — Pi

## Deploy Pi

```bash
dotagents deploy
```

## Add Pi Target

```toml
targets = ["claude", "codex", "opencode", "pi"]
```

## Provider Template Reference

- `public/v1/templates/pi/`
