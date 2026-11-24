
# UX

For the CLI I will be using eshu as the arg parser. 

Eshu's readme is inside v1_refactor/ but it supports: 
`$ tool_name [-a] [-bC] [-d required_arg] [-e [optional_arg]] [-f=required_arg] [-g[=optional_arg]] [-h arg0] [-h argN] [-i key=value] [--long-flags] [--long-flag[=optional_arg]] [--long-flag=required_arg] [--flag_with=key=value] [-- optional_positional_args] [subcommand] [subcommand_args_and_flags]`

Thus `$ argos run projectname1 name2` would immediately error with `unknown subcommand 'projectname1'`.

Running argos by itself would check the current service status, and check if the local files are correctly setup (first run).

For example

```bash
$ argos

Argos server not set up. No systemctl entry found.
Example `argos.service` and `argos.timer` files can be found in '.local/share/argos/systemctl_examples' or the argos repo.
[Example command to add a timer to systemctl? (Could also add a subcommand; requires sudo!)]

Argos data dir not found. Created: '.local/share/argos' and filled with needed data.

Argos repo tracking file found: '.local/share/argos/repo_list.json'.
Tracking file contains no projects. Fill by calling `argos config track my_project_name`.
Tracking file contains default git-root. Update by calling `argos config git-root ssh:git@server:2222/user/`.
```

```bash
$ argos

Argos systemctl entry found.
Last run at [DATE] reported a [Success | Failure].

[Failure output if any]

Argos data dir found at '.local/share/argos'.

Argos repo tracking file found: '.local/share/argos/repo_list.json'.
Tracking file contains 42 projects. Update by calling `argos config track my_project_name` or `argos config untrack my_project_name`.
Tracking file contains default git-root. Update by calling `argos config git-root ssh:git@server:2222/user/`.
```
## Subcommands

### Run

Run the full CI-Pipeline.

#### Locking & atomic writes:

Add a lockfile (e.g. argos.lock) around argos run so a manual run and the systemd timer can't clobber each other, and write repo_list.json / tracking files via temp file + rename so argos status never reads a half-written file.

#### Specific Projects

```bash
$ argos run -p Project1 Project2
$ argos run --project Project1 Project2
```

#### Flags

###### --idle

Use `SCHED_IDLE` and IO_NICE_CLASS `IONICE_IDLE`.

(While the `.service` file already does exactly that, I added it to that file for 'completeness' and future reference if I need it. Also, doubling up shouldn't hurt in any sense, it just violates DRY a bit.) 

###### --skip-unchanged 

Reads the git hashes and skips if no changes were made.

`argos run` is 'reserved' for manual and `argos run --idle --skip-unchanged` for the service file.

###### --dry-run

Loads repo_list.json, computes the Kahn dependency order, evaluates --skip-unchanged (hashes + periodic timer), and prints what would run / what would be skipped and why—without pulling, building Docker images, or pushing.
Great for verifying requires ordering and debugging skip logic safely.

###### --no-commit

Runs the full pipeline locally without committing or pushing clippy/fmt/update/license changes back to your git server. Very handy while testing Argos v1 or debugging a flaky repo.

###### --steps

E.g. `argos run --steps test clippy -p areia` to re-run just one or two integration steps on a failing repo without sitting through doc, update, format, etc.

### Clean

Data deletion. Needs to be explicitly told what to do, bare running doesnt do shit.

With an interactive `Y/N` interrupt (with -y flag overwrite)

#### Flags

##### --repos

Clears all cloned repos and their contained `/target`

##### --shared-cache

Clears `cargo_cache`

##### --docker-images

Deletes the docker images only

##### --history

Clears `repo_tracking` subdirs

##### --tracking

Clears all data from `repo_tracking`

##### --purge

Deletes everything but:

- repo_list.json
- Dockerfile.default
- repo_tracking/

##### --purge-all

Deletes everything.

##### --dry-run

Lists exactly which directories/images would be removed and how much disk space would be reclaimed before deleting anything.

### Status

A quick and dirty look at whats going on:

```bash

$ argos status

Server: [ServerStatus]
Git Root: [URI]
Tracking: 42 repositories

Passing: 20 | Failing: 15 | Skipped: 7

```

##### Status for a specific project

Options:
```bash
$ argos status -p PROJECTNAME ANOTHERPROJECT
$ argos status --project PROJECTNAME ANOTHERPROJECT
```

##### ServerStatus

Confirmed to work without sudo on the homelab:

```bash
master@serverle:~$ systemctl show argos.service --no-pager -p LoadState -p ActiveState -p SubState -p Result -p ExecMainStartTimestamp -p ExecMainExitTimestamp -p MemoryPeak -p CPUUsageNSec
Result=exit-code
ExecMainStartTimestamp=Mon 2026-11-23 18:41:24 CET
ExecMainExitTimestamp=Mon 2026-11-23 18:41:24 CET
MemoryPeak=1830912
CPUUsageNSec=9417000
LoadState=loaded
ActiveState=failed
SubState=failed
```
If `ActiveState=failed`, fetch the last few log lines via `journalctl -u argos.service -n 5 --no-pager -o cat` (and handle permission errors gracefully if the user isn't in systemd-journal)

query `systemctl show argos.timer -p NextElapseUSecRealtime` to display when the next run is scheduled

##### Flags

###### --full

```bash

$ argos status --full

Server: [ServerStatus]
Git Root: [URI]
Tracking: 42 repositories
Last run: YYYY-MM-DDTHH:MM:SSZ

Passing: 20 | Failing: 15 | Skipped: 5 | Never run: 2

[Each repo tracked via repo_list]:
Passing: [True | False]
[if not passing]:
Each fail as a list

```

###### --failing

Uses `--full` output, if that flag is present or not.

Print status only for projects with failing passes.

###### --file

Uses `--full` output, if that flag is present or not.

All output is recorded into a single file.

Cannot be used together with `--directory`.

```bash

$ argos status --file a/path/file.[txt | md | toml | json | csv | xff]

```

Filetype detection by passed in file extension.
Overwritten if `--format` is used.

###### --directory

Uses `--full` output, if that flag is present or not.

All output is recorded into a single directory with one file per tracked repo and one general file (the short status output).

Cannot be used together with `--file`.

```bash

$ argos status --directory a/path/

```

Defaults to text output (markdown files).

###### --format

Overwrite the file format for both `--file` and `--directory`

```bash

$ argos status --directory a/path/ --format=[txt | md | toml | json | csv | xff]

```

###### --history[=N]

Right now Argos maintains a 100-run .xff archive per repo (repo_tracking/{repo}/\*.xff), yet nothing ever
reads it!
Showing a compact timeline (`argos status --history=5 -- hades` → timestamp, short hash, pass/fail, failed
steps) finally makes that archive useful—especially for spotting flaky tests like hades / tyche.

###### --check (or -q / --quiet)

Exits 0 if all tracked repos pass (and service isn't failed), 1 otherwise. Useful if you ever wire Argos into an SSH MOTD, healthcheck, or notification script.

### Config

#### Track

If the supplied name is already tracked, output 'NAME is already tracked! No changes made.'

```bash

$ argos config --track PROJECTNAME NAME2

```

#### Untrack

If the supplied name is not tracked, output 'NAME not tracked! No changes made.'

```bash

$ argos config --untrack PROJECTNAME Name2

```

#### Gitroot

Sets the git-root with no checking if it works.

```bash

$ argos config --git-root ssh:git@server:2222/user/

```
Local URL format + trailing / normalization: RepoEnvironment::new builds URLs as format!("{}{}.git", env.git_root_url, repo). If someone passes ssh://git@serverle:2222/Xqhare without a trailing /, every single repo URL silently breaks (.../Xqhareargos.git).
Validate the scheme (ssh://, https://, http://, or user@host:) and check for trailing /.

### Deps

Prints the resolved Kahn execution order and dependency edges across all tracked repos, flagging missing/untracked dependencies or cycles
```bash

$ argos deps

```

### Logs (e.g. argos logs -n 50 / argos logs --full)

Thin wrapper around journalctl -u argos.service so you don't have to remember the systemd flags when checking why a scheduled run failed.
