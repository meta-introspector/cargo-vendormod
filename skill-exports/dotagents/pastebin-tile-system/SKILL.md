---
name: pastebin-tile-system
description: Auto-detect, compose, render, and test interactive tiles in the Kant Pastebin. Covers tile detection from pasted content, plugin-based rendering, standalone testing, and perf measurement across 8 plugin types.
license: MIT
compatibility: cross-agent
metadata:
  imported: true
  source: /home/mdupont/.agents/skills/pastebin-tile-system/SKILL.md
---

# Pastebin Tile System

**Purpose**: When content is pasted into the Kant Pastebin, auto-detect what type it is (PlantUML, Graphviz DOT, MiniZinc model, Lean 4 proof, Tulip graph, MIDI file, etc.) and render it as an interactive tile with Run/Solve/Verify/Render/Analyze buttons.

## Architecture

```
User pastes content
       │
       ▼
┌──────────────────┐
│  detect_tile_type()│  ← checks by: MIME type, file extension, content patterns
└──────┬───────────┘
       │ matches?
       ▼
┌──────────────────┐
│  render_tile_html()│  ← generates interactive tile HTML
└──────┬───────────┘
       │
       ▼
┌──────────────────┐     ┌──────────────────┐
│  Tile displayed   │────▶│  Plugin executes  │
│  with Run button   │     │  (subprocess)     │
└──────────────────┘     └──────────────────┘
       │                          │
       ▼                          ▼
  ┌──────────┐             ┌──────────┐
  │  18 unit │             │  8 plugin │
  │  tests ✅ │             │  types    │
  └──────────┘             └──────────┘
```

## Tile Detection

The `detect_tile_type(title, mime, content)` function checks in order:

### 1. MIME type

| MIME Type | Tile |
|-----------|------|
| `text/vnd.plantuml` | plantuml |
| `text/vnd.graphviz` | graphviz |
| `text/x-minizinc` | minizinc |
| `text/x-lean` | lean |
| `text/x-tulip` | tulip |

### 2. File extension (from title)

| Extension | Tile |
|-----------|------|
| `.puml`, `.plantuml` | plantuml |
| `.dot`, `.gv` | graphviz |
| `.mzn` | minizinc |
| `.lean` | lean |
| `.tlp` | tulip |

### 3. Content patterns

| Content starts with | Tile |
|---------------------|------|
| `@startuml`, `@startdot` | plantuml |
| `digraph`, `graph ` | graphviz |
| `theorem`, `lemma`, `def ` | lean |
| `(nodes `, `(TLP` | tulip |

| Content contains | Tile |
|------------------|------|
| `constraint `, `solve satisfy` | minizinc |

## Tile Rendering

Each tile type generates an interactive HTML component:

```html
<div class="tile" data-tile="plantuml">
  <h3>📐 PlantUML: filename.puml</h3>
  <pre style="max-height:200px;overflow:auto">@startuml
  A -> B
  @enduml</pre>
  <button onclick="renderPlantUML(this)">▶ Render Diagram</button>
  <div class="tile-output"></div>
</div>
```

### Tile JavaScript Functions

| Tile | JS Function | HTTP Endpoint |
|------|-------------|---------------|
| 📐 PlantUML | `renderPlantUML(btn)` | `POST /plugin/plantuml?action=render&format=svg` |
| 📊 Graphviz | `renderGraphViz(btn)` | `POST /plugin/graphviz?action=render&format=svg` |
| 🧮 MiniZinc | `solveMiniZinc(btn)` | `POST /plugin/minizinc?action=solve` |
| 🏛️ Lean 4 | `verifyLean(btn)` | `POST /plugin/lean?action=verify` |
| 🔗 Tulip | `analyzeTulip(btn)` | `POST /plugin/tulip?action=analyze` |

Each function:
1. Disables the button, shows ⏳
2. Sends content to the plugin endpoint
3. Displays result in the output div
4. Shows ✅ or ❌ on completion

## Plugin System

Each tile type is backed by a Rust plugin implementing the `Plugin` trait:

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, input: &PluginInput) -> PluginResult;
}
```

Registered in `main.rs`:

```rust
registry.register(Box::new(plugins::plantuml::PlantUmlPlugin::new(car_index)));
registry.register(Box::new(plugins::graphviz::GraphvizPlugin::new()));
registry.register(Box::new(plugins::minizinc::MiniZincPlugin::new()));
registry.register(Box::new(plugins::lean::LeanPlugin::new()));
registry.register(Box::new(plugins::tulip::TulipPlugin::new()));
registry.register(Box::new(plugins::midi::MidiPlugin::new(car_index)));
registry.register(Box::new(plugins::tiles::TilesPlugin::new()));
```

Routes via `/plugin/{name}` handler:

```rust
.route("/plugin/{name}", web::get().to(handlers::plugin_route))
```

## Testing

### Quick test (18 tests, instant)

```bash
cd /home/mdupont/pastebin
bash tests/run_tile_tests.sh
```

Tests tile detection by extension, MIME, content, and rendering correctness. Written in Bash + Python — no Rust compilation needed.

### Rust integration tests

```bash
cd /home/mdupont/pastebin
cargo test --test tile_tests          # standalone (no deps)
cargo test -p kant-pastebin --lib tile_test  # library module (slow)
```

### Test coverage

| Category | Tests | What's verified |
|----------|-------|-----------------|
| Detection by extension | 5 | `.puml`, `.dot`, `.mzn`, `.lean`, `.tlp` |
| Detection by content | 5 | `@startuml`, `digraph`, `solve satisfy`, `theorem`, `(nodes` |
| Detection by MIME | 5 | plantuml, graphviz, minizinc, lean, tulip MIMEs |
| No match | 1 | Plain text returns empty |
| Render buttons | 5 | Each tile has correct JS function name |
| Unknown type | 1 | Returns empty string |
| HTML escaping | 1 | `<` → `&lt;`, prevents XSS |

## Files

### Pastebin source

```
src/
├── handlers.rs          # detect_tile_type(), render_tile_html(), plugin_route()
├── plugin.rs            # Plugin trait, PluginRegistry
├── plugins/
│   ├── plantuml.rs      # PlantUML renderer via subprocess
│   ├── graphviz.rs      # Graphviz DOT renderer + METIS partitioning
│   ├── minizinc.rs      # MiniZinc constraint solving
│   ├── lean.rs           # Lean 4 theorem proving
│   ├── tulip.rs          # Tulip graph analysis
│   ├── midi.rs           # MIDI file browser via locate
│   └── tiles.rs          # DAG-CBOR spec tiles viewer
├── tile_test.rs          # Library test module with perf measurement
├── car_index.rs          # Locate-based file index
└── main.rs               # Route registration + plugin init

tests/
├── tile_tests.rs          # Standalone Rust integration tests
└── run_tile_tests.sh      # Shell/Python test runner (18 tests)
```

## Adding a New Tile Type

1. **Create plugin** in `src/plugins/{name}.rs` implementing `Plugin` trait
2. **Register** in `src/main.rs`: `registry.register(Box::new(plugins::{name}::{Name}Plugin::new()));`
3. **Add detection** in `detect_tile_type()`: check MIME, extension, or content pattern
4. **Add renderer** in `render_tile_html()`: generate the interactive tile HTML
5. **Add JS function** in `handlers.rs`: button handler that calls `/plugin/{name}` endpoint
6. **Add route**: `.route("/{name}", ...)` or use the generic `/plugin/{name}` route
7. **Add tests** in `tests/run_tile_tests.sh` and `tests/tile_tests.rs`

## Commands Quick Reference

```bash
# Run tests
bash tests/run_tile_tests.sh          # 18 tests, instant
cargo test --test tile_tests           # Rust standalone

# Start pastebin
nix develop -c cargo run               # Dev server on :8090

# View tiles
curl http://localhost:8090/plugin/tiles     # DAG-CBOR spec tiles
curl http://localhost:8090/car/midi         # MIDI browser
curl http://localhost:8090/car/plantuml     # PlantUML browser
curl "http://localhost:8090/plugin/plantuml?action=render" -d "@startuml\nA->B\n@enduml"
```
