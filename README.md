# Argos

Argos is a custom home-lab CI pipeline.

## Naming

Argos is named after the legendary faithful dog of Odysseus, who waits for his masters return to Ithaca for 20 years. Upon seing his master return home, Argos dies.

The reason is simple: Just as his namesake, Argos will be left alone, and will wait for the seldom return of his master.

## Usage

Argos reads a list of repos from the `{runtimeDirectory}/repo_list.json` file.

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
- `doc-tests` - Runs `cargo test --doc`
- `clippy` - Runs `cargo clippy` and tries to automatically fix the errors
- `format` / `fmt` - Runs `cargo fmt`
- `update` - Runs `cargo update`
- `all` - Runs all commands

> Note on `clippy`, `format` and `update`:
> These commands are run after the test pipeline.
> The test pipeline is then run again. Changes are only applied if the test pipeline succeeds.

#### Advanced configuration

If you want to provide more advanced configuration, you can create a file called `conf.json` stored inside the `ArgosCI` directory:

```json
{
  "test": {
    "args": ["all-features", "no-doc"],
    "requires": ["nabu", "athena"],
    "requires_ext": true
  },
  "build": {
    "requires": ["athena"],
    "requires_ext": true
  }
}
```

If `requires_ext` is set to `true`, please provide a `Dockerfile` to set up the environment. If one file is enough for all commands, provide one `Dockerfile` inside the `ArgosCI` directory. If you need to provide different dockerfiles for different commands, please provide one file for each command inside their own subdirectories inside `ArgosCI`.

> Note that if you provide specialised dockerfiles, you can also provide a `Dockerfile` inside the `ArgosCI` directory as a fallback / default one to use.

> Please also note, that if both a `conf.json` and `argos.json` is provided, the `argos.json` file is used as a fallback if the `conf.json` cannot be read or the needed Dockerfiles are missing.

#### Directory structure example:

`ArgosCI`
├── `argos.json`
├── `conf.json`
├── `Dockerfile`
├── `test`
│   └── `Dockerfile`
└── `build`
    └── `Dockerfile`

---

For more information on the start of Argos, see the [startup notes](startup-notes.md).
