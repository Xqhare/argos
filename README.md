# Argos

Argos is a custom home-lab CI pipeline.

## Naming

Argos is named after the legendary faithful dog of Odysseus, who waits for his masters return to Ithaca for 20 years. Upon seing his master return home, Argos dies.

The reason is simple: Just as his namesake, Argos will be left alone, and will wait for the seldom return of his master.

## Prerequisites

Argos expects the following tools to be available on the host system:

- **Docker**: Used for executing all pipeline commands in isolation.
- **Git**: Used for cloning, pulling, and committing changes.
- **`id` utility**: Used to detect User and Group IDs for Docker user mapping.

Additionally, ensure your host user has permissions to run Docker commands and that SSH keys are configured for Git operations if using SSH URLs.

## Usage

Argos reads a list of repos from the `{dataDirectory}/argos/repo_list.json` file.

> **Note on `{dataDirectory}`:** On Linux, this is typically `$XDG_DATA_HOME` or `~/.local/share`.

This file is a JSON object containing the git url root and a list of repo names.
To change the git root url, provide a `git_root` key in the repo config.

Adding a repo to the CI pipeline is as simple as adding it to the `repos` array.

```json
{
  "git_root": "ssh://git@server:2222/Xqhare/",
  "repos": [
    "nabu",
    "athena"
  ]
}
```

## Docker Integration

Argos uses Docker to provide an isolated and consistent environment for all Cargo commands. 

### How it works

Every command (except `license`) is executed within an ephemeral Docker container (`docker run --rm`). 

1.  **Shared Cache**: Argos maintains a shared Cargo cache directory on the host (`{dataDirectory}/argos/cargo_cache`) which is mounted into every container. This prevents redundant crate downloads.
2.  **User Mapping**: Argos automatically detects your host User ID and Group ID and maps them into the container. This ensures that any files created (like `target/` or `Cargo.lock` updates) are owned by you on the host.
3.  **Automatic Build**: Before running a command, Argos builds the required Docker image to ensure it is up-to-date.

### Dockerfile Hierarchy

Argos searches for Dockerfiles in the following order:

1.  **Command Specific**: `ArgosCI/{command}/Dockerfile` (e.g., `ArgosCI/test/Dockerfile`)
2.  **Repository Wide**: `ArgosCI/Dockerfile`
3.  **System Fallback**: `{dataDirectory}/argos/Dockerfile.default`

Argos automatically creates a `Dockerfile.default` based on `rust:latest` if it doesn't exist. You can find a template in the root of the Argos repository named `sample.Dockerfile`.

## Process Prioritization

To ensure that Argos does not interfere with the responsiveness of your home-lab or development environment, it automatically configures itself to run with **Idle** priority:

- **CPU Scheduling**: Uses `SCHED_IDLE`, the lowest possible priority.
- **I/O Priority**: Uses `IDLE` class, ensuring it only performs disk operations when the system is otherwise quiet.

## Repo specific configuration

Every repo may provide further configuration for Argos. These must be stored in a directory called `ArgosCI` in the root of the repo.

### Basic configuration

Basic configuration is provided in a file called `argos.json` stored inside the `ArgosCI` directory. It is a simple array of commands to run:

```json
[
  "test",
  "build"
]
```

### Advanced configuration

For more control, use `config.json` inside the `ArgosCI` directory:

```json
{
  "requires": ["nabu", "athena"],
  "test": {
    "args": ["--all-features", "--locked"],
    "requires_ext": true
  },
  "clippy": {
    "args": ["--", "-D", "warnings"]
  },
  "all": null
}
```

- **requires**: A list of repositories that must pass their CI before this repository is integrated.
- **args**: Custom arguments passed directly to the `cargo` command.
- **requires_ext**: If `true`, the command **must** find a repository-specific Dockerfile in `ArgosCI/` or it will fail. This prevents silent fallbacks to the default system Dockerfile.
- **all**: If present, runs all supported commands.

### Supported Commands

- `test` - Runs `cargo test`
- `build` - Runs `cargo build`
- `doc` - Runs `cargo doc --no-deps`
- `doc-test` - Runs `cargo test --doc`
- `clippy` - Runs `cargo clippy --fix`. Note: This command always enforces `clippy::all` and `clippy::pedantic` lints.
- `format` - Runs `cargo fmt`
- `update` - Runs `cargo update`
- `license` (**Local**): Updates MIT license years based on git history. Supports `LICENSE` and `LICENSE-MIT` files.

> **Note on `clippy`, `format` and `update`**: These commands are run after the test pipeline. Changes are only committed and pushed if the tests pass after the modification.

### Automatic Cleanup

To prevent disk space exhaustion, Argos automatically runs `cargo clean` inside the Docker container for a repository if:
- Any command in the integration pipeline fails.
- The `target/` directory for that repository exceeds **2 GB**.

## Output

The output for the latest run of each repository is saved in:

- **JSON Report:** `{dataDirectory}/argos/repo_tracking/{repo}.json`
- **XFF Data:** `{dataDirectory}/argos/repo_tracking/{repo}.xff`

Historical data for the last 100 runs is maintained in:
`{dataDirectory}/argos/repo_tracking/{repo}/{dateTime}.xff`

## Directory Structure

```text
[Data Directory] (usually ~/.local/share)
└── argos/
    ├── repo_list.json
    ├── Dockerfile.default
    ├── cargo_cache/
    ├── repo_tracking/
    │   ├── {repo}.json
    │   ├── {repo}.xff
    │   └── {repo}/
    │       └── {dateTime}.xff
    └── {repo}/ (Cloned repositories)
        └── ArgosCI/
            ├── argos.json
            ├── config.json
            └── Dockerfile
```

---

For more information on the start of Argos, see the [startup notes](startup-notes.md).
