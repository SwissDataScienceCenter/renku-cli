# Feature: Project Scope Activation

## Overview

Add a `rnk project activate` command (like `conda activate`) that sets a global active project. All subsequent commands use that project as their default scope, and `launcher ls` / `job ls` automatically filter to that project.

## Design Decisions

| # | Decision | Choice |
|---|---|---|
| 1 | Scope | Global + per-directory + CLI flag, with precedence |
| 2 | Command names | `rnk project activate` / `deactivate` / `current` with aliases `p a` / `p d` / `p c` |
| 3 | Scoping scope | Only `launcher ls` and `job ls` (existing `resolve_project_context()` mechanism) |
| 4 | Storage location | `~/.config/renku-cli/active_project.toml` |
| 5 | Deactivate | Clears only the global config file |
| 6 | Per-directory vs global conflict | Per-directory `.renku/config.toml` wins |
| 7 | Auto-activate on clone | No — `project clone` already writes per-directory config |
| 8 | Config contents | Just `project_id` (TOML format) |
| 9 | Validation | Validate via API on activate, store resolved ID |
| 10 | Deactivate + env var | No — env var is user's responsibility |
| 11 | Status command | `rnk project current` shows full project details |
| 12 | URL in config | Store both `project_id` and `renku_url` |
| 13 | Precedence chain | CLI flag > env var > per-directory `.renku/config.toml` > global |
| 14 | Warning on override | Only on `rnk project current`, not on every command |
| 15 | Current output | Project ID, namespace, slug, renku URL, source |
| 16 | Activate output | Short confirmation: `Active project set to: namespace/project (id)` |
| 17 | Invalid project | Error message + non-zero exit |
| 18 | No project active | Actionable message with command hint |
| 19 | Always show source | `rnk project current` always shows where the project came from |
| 20 | Scope expansion | Out of scope for v1 |

## Precedence Chain

```
1. --project-context CLI flag (highest)
2. RENKU_CLI_PROJECT_CONTEXT env var
3. .renku/config.toml in CWD (per-directory)
4. Global active project (~/.config/renku-cli/active_project.toml) (lowest)
```

## Commands

### `rnk project activate <project>`

- Accepts same input as `project clone`: project ID, `namespace/project`, or full URL
- Validates the project exists via API (`get_project_by_slug` / `get_project_by_id`)
- Writes `~/.config/renku-cli/active_project.toml`:
  ```toml
  project_id = "abc123"
  renku_url = "https://renkulab.io"
  ```
- Prints: `Active project set to: namespace/project (abc123)`
- Exits non-zero if project not found or API error

### `rnk project deactivate`

- Removes `~/.config/renku-cli/active_project.toml`
- Does NOT touch env vars or per-directory configs
- Silent on success

### `rnk project current`

- Shows active project details:
  ```
  Project: my-namespace/my-project
  ID: abc123
  Renku URL: https://renkulab.io
  Source: global config
  ```
- Source is one of: `global config`, `per-directory config`, `environment variable`, `CLI flag`
- If no project is active: `No active project set. Use 'rnk project activate <project>' to set one.`
- Warns if per-directory config overrides global active project

## Checklist

### `rnk project activate <project>`

- [ ] **A.1** Create `src/cli/cmd/project/activate.rs`
  - Parse project input (ID, namespace/slug, or URL)
  - Resolve to project ID via API (`get_project_by_slug` / `get_project_by_id`)
  - Write global config file via `project_active::write_global()`
  - Print confirmation: `Active project set to: namespace/project (id)`
  - Exit non-zero on validation/API error

- [ ] **A.2** Wire into `src/cli/cmd/project.rs`
  - Add `Activate` variant to `ProjectCommand` enum
  - Add `alias = "a"` so `rnk p a` works

- [ ] **A.3** Unit tests
  - Valid project ID → success
  - Valid namespace/slug → success
  - Valid URL → success
  - Non-existent project → error + non-zero exit
  - API error → error + non-zero exit

- [ ] **A.4** Help text
  - `rnk project activate --help` shows usage and examples

---

### `rnk project deactivate`

- [ ] **D.1** Create `src/cli/cmd/project/deactivate.rs`
  - Remove `~/.config/renku-cli/active_project.toml` if it exists
  - Silent on success, warn if file doesn't exist
  - Does NOT touch env vars or per-directory configs

- [ ] **D.2** Wire into `src/cli/cmd/project.rs`
  - Add `Deactivate` variant to `ProjectCommand` enum
  - Add `alias = "d"` so `rnk p d` works

- [ ] **D.3** Unit tests
  - Active global config → removed
  - No active config → silent success or warning

- [ ] **D.4** Help text
  - `rnk project deactivate --help` shows usage

---

### `rnk project current`

- [ ] **C.1** Create `src/cli/cmd/project/current.rs`
  - Determine active project using precedence chain (flag > env > per-directory > global)
  - Display: project ID, namespace, slug, renku URL, source
  - Source is one of: `global config`, `per-directory config`, `environment variable`, `CLI flag`
  - Warn if per-directory config overrides global active project
  - If no project active: `No active project set. Use 'rnk project activate <project>' to set one.`

- [ ] **C.2** Wire into `src/cli/cmd/project.rs`
  - Add `Current` variant to `ProjectCommand` enum
  - Add `alias = "c"` so `rnk p c` works

- [ ] **C.3** Unit tests
  - Global active project → shows details + source
  - Per-directory active project → shows details + source
  - No active project → actionable message
  - Per-directory overrides global → warning

- [ ] **C.4** Help text
  - `rnk project current --help` shows usage

---

### Global Config Module

- [ ] **G.1** Create `src/project_active.rs`
  - `read_global() -> Option<ActiveProject>`
  - `write_global(project_id, renku_url) -> Result`
  - `remove_global() -> Result`
  - TOML format: `project_id = "..."` / `renku_url = "..."`
  - Config path: `~/.config/renku-cli/active_project.toml`
  - Use `directories::ProjectDirs::from("io.renku", "sdsc", "renku-cli")` for path

- [ ] **G.2** Unit tests
  - Write, read, remove global config
  - TOML serialization/deserialization

---

### Integration: Precedence Chain

- [ ] **I.1** Update `get_project_context()` in `src/cli/opts.rs`
  - New precedence: CLI flag > env var > per-directory `.renku/config.toml` > global `project_active`
  - Global config read via `project_active::read_global()`

- [ ] **I.2** Verify `resolve_project_context()` in `src/cli/cmd.rs`
  - No changes needed — already calls `opts.get_project_context()`

- [ ] **I.3** Verify `project clone` does NOT set global active project
  - Current behavior: only writes per-directory `.renku/config.toml`
  - No changes needed

- [ ] **I.4** Unit tests for precedence chain
  - CLI flag overrides all
  - Env var overrides per-directory and global
  - Per-directory overrides global
  - Global is fallback

- [ ] **I.5** End-to-end verification
  - `launcher ls` and `job ls` use the new precedence chain (no code changes needed)
  - Test: activate → launcher ls shows filtered results

---

### Polish

- [ ] **P.1** Update `rnk --help` to show new subcommands under `project`
- [ ] **P.2** Update README.md with new commands
- [ ] **P.3** `cargo clippy` and `cargo fmt` pass