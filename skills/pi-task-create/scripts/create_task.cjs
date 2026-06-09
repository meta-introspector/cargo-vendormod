const fs = require('fs');
const path = require('path');

const SKILL_DIR = __dirname; // Current directory (scripts)
const ASSETS_DIR = path.join(SKILL_DIR, '../assets');

async function main() {
  const args = process.argv.slice(2);
  const taskName = getArg(args, '--task-name');
  const taskObjective = getArg(args, '--task-objective'); // This will be the GEMINI.md content
  const targetDirectory = getArg(args, '--target-directory');
  const skillsToLoad = getArg(args, '--skills-to-load');

  if (!taskName || !taskObjective || !targetDirectory || !skillsToLoad) {
    console.error('Usage: node create_task.cjs --task-name <name> --task-objective <objective> --target-directory <dir> --skills-to-load <skills>');
    process.exit(1);
  }

  const taskDirPath = path.join(targetDirectory, taskName);
  fs.mkdirSync(taskDirPath, { recursive: true });
  console.log(`Created task directory: ${taskDirPath}`);

  // --- Generate flake.nix ---
  let flakeNixContent = fs.readFileSync(path.join(ASSETS_DIR, 'flake.nix.template'), 'utf8');
  flakeNixContent = flakeNixContent
    .replace(/{{TASK_NAME}}/g, taskName)
    // Assuming cargo-vendormod reference is fixed
    .replace(/{{CARGO_VENDORMOD_LOCAL_PATH}}/g, '/home/mdupont/git/solana.solfunmeme.com/cargo-vendormod'); // Hardcoding for now

  fs.writeFileSync(path.join(taskDirPath, 'flake.nix'), flakeNixContent);
  console.log(`Generated ${taskDirPath}/flake.nix`);

  // --- Generate runme.sh ---
  let runmeShContent = fs.readFileSync(path.join(ASSETS_DIR, 'runme.sh.template'), 'utf8');
  runmeShContent = runmeShContent
    .replace(/{{TASK_NAME}}/g, taskName)
    .replace(/{{SKILLS_TO_LOAD}}/g, skillsToLoad);

  fs.writeFileSync(path.join(taskDirPath, 'runme.sh'), runmeShContent);
  fs.chmodSync(path.join(taskDirPath, 'runme.sh'), 0o755); // Make executable
  console.log(`Generated ${taskDirPath}/runme.sh`);

  // --- Generate GEMINI.md ---
  let geminiMdContent = fs.readFileSync(path.join(ASSETS_DIR, 'GEMINI.md.template'), 'utf8');
  geminiMdContent = geminiMdContent.replace(/{{TASK_OBJECTIVE}}/g, taskObjective);
  fs.writeFileSync(path.join(taskDirPath, 'GEMINI.md'), geminiMdContent);
  console.log(`Generated ${taskDirPath}/GEMINI.md`);
}

function getArg(args, name) {
  const index = args.indexOf(name);
  if (index > -1 && index + 1 < args.length) {
    return args[index + 1];
  }
  return null;
}

main();
