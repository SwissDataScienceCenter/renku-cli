# Renku CLI

[![CI](https://github.com/SwissDataScienceCenter/renku-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SwissDataScienceCenter/renku-cli/actions/workflows/ci.yml)
[![Version](https://img.shields.io/github/v/release/SwissDataScienceCenter/renku-cli)](https://github.com/SwissDataScienceCenter/renku-cli/releases)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE.txt)

Manage your Renku projects, datasets, and jobs from the terminal.

## Installation

### Quick Install

```bash
curl -sfSL https://raw.githubusercontent.com/SwissDataScienceCenter/renku-cli/main/install.sh | bash
```

### Nix

```bash
nix profile install github:SwissDataScienceCenter/renku-cli
```

For other installation methods (manual binary, NixOS integration), see the [Installation Guide](docs/install.md).

## Quick Start

Get up and running in three steps:

```bash
# 1. Authenticate to your Renku instance
rnk login

# 2. Clone a project
rnk clone <project-ref>

# 3. List running jobs
rnk job list
```

## Commands

### Authentication

| Command | Description |
|---------|-------------|
| `rnk login` | Perform interactive login to Renku |
| `rnk logout` | Remove stored credentials |

### Projects

| Command | Description |
|---------|-------------|
| `rnk clone <project-ref>` | Clone a project by ID, namespace/slug, or URL |
| `rnk project clone <project-ref>` | Same as above (full command form) |

### Datasets

| Command | Description |
|---------|-------------|
| `rnk dataset deposit` | Manage dataset deposits |

### Jobs

| Command | Description |
|---------|-------------|
| `rnk job list` | List jobs |
| `rnk job start` | Start a job |
| `rnk job stop` | Stop a job |
| `rnk job logs` | View job logs |

### Other

| Command | Description |
|---------|-------------|
| `rnk version` | Show client and server version info |
| `rnk update` | Check for and install updates |

## Configuration

All commands share these environment variables:

| Variable | Description |
|----------|-------------|
| `RENKU_CLI_RENKU_URL` | Base URL to your Renku instance |
| `RENKU_CLI_PROJECT_CONTEXT` | Default project context (`<username>/<project>` or project ID) |
| `RENKU_CLI_ACCESS_TOKEN` | Manual access token (skips `rnk login` if set) |

Global flags available on every command:

- `--format json` — structured JSON output instead of human-readable
- `--proxy <url>` / `--proxy none` — control proxy usage
- `-v` / `-q` — increase / decrease log verbosity

For a full list of options, run `rnk --help`.

## Shell Completions

See the [Installation Guide](docs/install.md) for shell completion setup (bash, fish, zsh, powershell).

## Documentation

Full documentation is available in the [docs/](docs/) directory.

## Links

- [Source Code](https://github.com/SwissDataScienceCenter/renku-cli)
- [Issue Tracker](https://github.com/SwissDataScienceCenter/renku-cli/issues)
- [Installation Guide](docs/install.md)
- [Contributing](CONTRIBUTING.md)