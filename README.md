# Argos

Argos is a custom home-lab CI pipeline.

## Naming

Argos is named after the legendary faithful dog of Odysseus, who waits for his masters return to Ithaca for 20 years. Upon seing his master return home, Argos dies.

The reason is simple: Just as his namesake, Argos will be left alone, and will wait for the seldom return of his master.

## Usage

Argos reads a list of repos from the `{dataDirectory}/repo_list.json` file.

> **Note on `{dataDirectory}`:** On Linux, this is typically `$XDG_DATA_HOME` or `~/.local/share`.

This file is a JSON object containing the git url root and a list of repo names.
To change the git root url, provide a `git_root` key in the repo config.

Adding a repo to the CI pipeline is as simple as adding it to the `repos` array.

```json
{
  "git_root": "https://github.com/xqhare/",
  "repos": [
    "nabu",
    "athena"
  ]
}
```

### Repo specific configuration

Every repo may provide further configuration for Argos.

These must be stored in a directory called `ArgosCI` in the root of the repo.

> If no configuration is given (or as a fallback in the most extreme cases), Argos will use the default configuration.

#### Default configuration

By default, Argos will run all commands it is capable of with no special environment or dependency checking. This includes `update` and `clippy --fix`, aswell as pushing their results if the tests keep passing.

#### Basic configuration

Basic configuration is provided in a file called `argos.json` stored inside the `ArgosCI` directory:

```json
[
  "test",
  "build",
]
```

Instead of listing all available commands, you can also use `all` as a shorthand or omit the file alltogether.

Supported commands are:

- `test` - Runs `cargo test`
- `build` - Runs `cargo build`
- `doc` - Runs `cargo doc --no-deps` to ensure documentation builds correctly.
- `doc-test` - Runs `cargo test --doc`
- `clippy` - Runs `cargo clippy` and tries to automatically fix the errors
- `format` - Runs `cargo fmt`
- `update` - Runs `cargo update`
- `license` - Updates the license year, only if the previous commit was in the current year (and the current year is missing from the license) - supports MIT only for now
- `all` - Runs all commands

> Note on `clippy`, `format` and `update`:
> These commands are run after the test pipeline.
> The test pipeline is then run again. Changes are only applied if the test pipeline succeeds.

> Basic configuration does not support the use of dockerfiles.

#### Advanced configuration

If you want to provide more advanced configuration, you can create a file called `config.json` stored inside the `ArgosCI` directory:

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
  "all": Null
}
```

The `requires` field is a list of repo names that must pass before the current repo can be tested. You may only list dependencies required to run all commands. There is no way of specifiying that `test` and `build` need different dependencies for example.

> Please note that supplying just a few commands with specific arguments, followed by "all" to enable all *other* commands with *default* behaviour is supported and expected behaviour.

If using "all" in conjunction with other commands, please note that the value of the "all" key does not matter and is not read. The value of the "all" key is set to `Null` by convention.

The `args` field is a list of strings that are passed directly to the command. This allows for fine-grained control over how your code is tested and built.

##### Common Arguments & Use Cases:

| Argument | Command | Description |
| :--- | :--- | :--- |
| --locked | Any (except `update`) | Ensures `Cargo.lock` is not updated during the CI run. |
| `--all-features` | `build`, `clippy`, `test`, `doc` | Enables all features of the crate for the run. |
| `--all-targets` | `build`, `clippy`, `test` | Ensures all targets (bins, libs, tests, examples, benches) are processed. **Note: This skips doc-tests.** |
| `-D warnings` | `build`, `clippy`, `doc` | (Passed after `--`) Forces the CI to fail if any warnings are detected. |
| `--no-deps` | `doc` | Builds documentation only for your crate (ignores dependencies). |
| `--no-default-features`| `build`, `test`, `doc` | Disables default features (useful for `no_std` testing). |
| `--target <TRIPLE>` | `build`, `test` | Cross-compiles for a specific target (e.g., `wasm32-unknown-unknown`). |
| `--release` | `build`, `test`, `doc` | Runs the command in release mode (optimised). |
| `--` | `clippy`, `test`, `doc` | Separator for passing arguments to the underlying binary or tool. |
| `-D clippy::pedantic`| `clippy` | (Passed after `--`) Enables pedantic lints for the run. |
| `-D clippy::restriction`| `clippy` | (Passed after `--`) Enables restriction lints for the run. |
| `--nocapture` | `test` | (Passed after `--`) Shows `println!` output in the CI logs. |

If `requires_ext` is set to `true`, please provide a `Dockerfile` to set up the environment. If one file is enough for all commands, provide one `Dockerfile` inside the `ArgosCI` directory. If you need to provide different dockerfiles for different commands, please provide one file for each command inside their own subdirectories, named like the command, inside `ArgosCI`.

> Note that if you provide specialised dockerfiles, you can also provide a `Dockerfile` inside the `ArgosCI` directory as a fallback / default one to use.

> Please also note, that if both a `config.json` and `argos.json` is provided, the `argos.json` file is used only as a fallback if the `config.json` cannot be read or the needed Dockerfiles are missing.

#### Directory structure example:

`ArgosCI`\
├── `argos.json`\
├── `config.json`\
├── `Dockerfile`\
├── `test`\
│   └── `Dockerfile`\
└── `build`\
&nbsp;&nbsp;&nbsp;└── `Dockerfile`

### Output

The output for the latest run of each repository is saved in both human-readable and binary formats within the tracking directory.

- **JSON Report:** `{dataDirectory}/argos/repo_tracking/{repo}.json`
- **XFF Data:** `{dataDirectory}/argos/repo_tracking/{repo}.xff`

Historical data for the last 100 runs is maintained in repository-specific subdirectories:
`{dataDirectory}/argos/repo_tracking/{repo}/{dateTime}.xff`

## Directory Structure

```text
[Data Directory] (from areia::BaseDirs)
└── argos_data_path
    ├── repo_list.json (repo_list_path)
    └── argos/ (argos_root_path)
        ├── repo_tracking/ (argos_repo_tracking_path)
        │   ├── {repo}/ - History of the last 100 runs
        │   │   └── {dateTime}.xff
        │   ├── {repo}.xff (repo_tracking_xff) - Latest run data
        │   └── {repo}.json (repo_tracking_json) - Latest run report
        └── {repo}/ (repo_path)
            └── ArgosCI/ (repo_config_dir_path)
                ├── argus.json (repo_basic_config_path)
                └── config.json (repo_advanced_config_path)
```

---

For more information on the start of Argos, see the [startup notes](startup-notes.md).

