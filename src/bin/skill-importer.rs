use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
struct Skill {
    name: String,
    description: String,
    body: String,
    source: PathBuf,
}

fn discover_skill_files(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path().to_path_buf();
            if p.file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
                out.push(p);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn parse_frontmatter(body: &str) -> (&str, &str) {
    let trimmed = body.trim_start();
    if let Some(rest) = trimmed.strip_prefix("---") {
        if let Some(end) = rest.find("---") {
            let fm = &rest[..end];
            let body = &rest[end + 3..];
            return (fm.trim(), body.trim());
        }
    }
    ("", trimmed)
}

fn import_skill(path: PathBuf) -> Result<Skill> {
    let body = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let (fm, rest) = parse_frontmatter(&body);
    let mut name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let mut description = String::new();
    for line in fm.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("name:") {
            name = rest.trim().trim_matches('"').to_string();
        }
        if let Some(rest) = t.strip_prefix("description:") {
            description = rest.trim().trim_matches('"').to_string();
        }
    }
    if description.is_empty() {
        description = "Imported skill".to_string();
    }
    Ok(Skill {
        name,
        description,
        body: rest.to_string(),
        source: path,
    })
}

fn write_kilo(skill: &Skill, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    let p = dest.join(format!("{}.md", skill.name));
    let content = format!(
        r#"---
name: {}
description: {}
---

{}
"#,
        skill.name, skill.description, skill.body
    );
    fs::write(&p, content).with_context(|| format!("writing {}", p.display()))?;
    Ok(())
}

fn write_claude(skill: &Skill, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    let p = dest.join(format!("{}.md", skill.name));
    let content = format!(
        r#"# {}

{}

## Instructions

{}

## Examples

- Run the skill workflow as documented in the source.
"#,
        skill.name, skill.description, skill.body
    );
    fs::write(&p, content)?;
    Ok(())
}

fn write_dotagents(skill: &Skill, dest: &Path) -> Result<()> {
    let p = dest.join(&skill.name).join("SKILL.md");
    fs::create_dir_all(p.parent().unwrap())?;
    let content = format!(
        r#"---
name: {}
description: {}
license: MIT
compatibility: cross-agent
metadata:
  imported: true
  source: {}
---

{}
"#,
        skill.name,
        skill.description,
        skill.source.display(),
        skill.body
    );
    fs::write(&p, content)?;
    Ok(())
}

fn write_generic_md(skill: &Skill, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    let p = dest.join(format!("{}.md", skill.name));
    let content = format!(
        r#"# {}

**Description**: {}

## Source

- {}

---

{}

"#,
        skill.name, skill.description, skill.source.display(), skill.body
    );
    fs::write(&p, content)?;
    Ok(())
}

fn write_agent_json(skills: &[Skill], dest: &Path, agent: &str) -> Result<()> {
    fs::create_dir_all(dest)?;
    let p = dest.join(format!("{}-skills.json", agent));
    let out: Vec<_> = skills
        .iter()
        .map(|s| serde_json::json!({"name": s.name, "description": s.description, "source": s.source.to_string_lossy().to_string()}))
        .collect();
    fs::write(&p, serde_json::to_string_pretty(&out)?)?;
    Ok(())
}

fn main() -> Result<()> {
    let roots = vec![
        PathBuf::from("/home/mdupont/.kilocode/skills"),
        PathBuf::from("/home/mdupont/.claude/skills"),
        PathBuf::from("/home/mdupont/.agents/skills"),
        PathBuf::from("/home/mdupont/.opencode/skills"),
        PathBuf::from("./skills"),
        PathBuf::from("./packages/agents/dotagents/.dotagents/skills"),
        PathBuf::from("./packages/agents/dotagents/.opencode/skills"),
    ];
    let files = discover_skill_files(&roots);
    if files.is_empty() {
        eprintln!("No skills found under {}", roots.len());
    } else {
        eprintln!("Found {} skill files", files.len());
    }

    let mut skills = Vec::new();
    for f in files {
            match import_skill(f.clone()) {
                Ok(s) => skills.push(s),
                Err(e) => eprintln!("skip {}: {e}", f.display()),
        }
    }

    let out = PathBuf::from("./skill-exports");
    for s in &skills {
        write_kilo(s, &out.join("kilo"))?;
        write_claude(s, &out.join("claude"))?;
        write_dotagents(s, &out.join("dotagents"))?;
        write_generic_md(s, &out.join("generic"))?;
    }

    let mut by_agent: HashMap<String, Vec<Skill>> = HashMap::new();
    for s in &skills {
        by_agent.entry(s.name.clone()).or_default().push(s.clone());
    }
    write_agent_json(&skills, &out.join("json"), "all")?;

    fs::write(
        out.join("manifest.md"),
        format!(
            "# Skills Manifest\n\nExported {} skills from {} sources.\n",
            skills.len(),
            roots.len(),
        ),
    )?;

    println!("Exported {} skills to {}", skills.len(), out.display());
    Ok(())
}
