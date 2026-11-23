
# UX

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

### Server / Service

#### Locking & atomic writes:

Add a lockfile (e.g. argos.lock) around argos run so a manual run and the systemd timer can't clobber each other, and write repo_list.json / tracking files via temp file + rename so argos status never reads a half-written file.

Really just symlink like with the webserver; updating symlinks remains atomic (and lockfile the access of course)

##### Flags

Unsure if anything besides `--no-idle` will actually be implemented. Would be nice to have though I think

###### --no-idle / --system-defaults

Will *not* set the scheduler to `SCHED_IDLE` and IO_NICE_CLASS to `IONICE_IDLE`.
Nothing else, system defaults.

```bash
$ argos server --no-idle
```

###### --io-nice

With key=values:

```bash
$ argos server --io-nice=class=[realtime | best-effort] --io-nice=data=[0..=7 (0 == Max prio)]
```

###### --scheduler

With key=values:

```bash
$ argos server --scheduler=policy=[standard | batch] --scheduler=prio=[-20..=19 (-20 = Max prio)]
```

###### --force & -f

Ignores the git commit hashes for all repos

###### --no-sleep

If I don't remove the sleep between repos, make it optional at least

### Run

To have argos just run (without touching the server itself; Spinning up another argos instance / process handle)

#### Flags

###### --force & -f

Ignores the git commit hashes for all repos

###### --sleep

How long to sleep between repos, in seconds.

```bash

$ argos run --sleep=60 -f

```

### Clean

Cleans out the caches (keeping all data relevant for argos to work, just deleting the code)

#### Flags

##### --purge

Deletes everything (repos, data, whatever; an uninstall of sorts)

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
A:
$ argos status -- PROJECTNAME ANOTHERPROJECT

B:
$ argos status -p PROJECTNAME -p ANOTHERPROJECT
B (alt):
$ argos status --project PROJECTNAME --project ANOTHERPROJECT
```

##### ServerStatus

Confirmed to work without sudo on the homelab:

```bash
master@serverle:~$ systemctl status argos
× argos.service - Argos CI Pipeline
     Loaded: loaded (/etc/systemd/system/argos.service; static)
     Active: failed (Result: exit-code) since Mon 2026-11-23 08:45:05 CET; 9h ago
 Invocation: 7d8f01d0dad242028548e707302ba2e1
TriggeredBy: ● argos.timer
    Process: 1539423 ExecStart=/usr/bin/argos (code=exited, status=1/FAILURE)
   Main PID: 1539423 (code=exited, status=1/FAILURE)
   Mem peak: 1.8M
        CPU: 14ms

Nov 23 08:45:05 serverle argos[1539423]: Error: Xff("No such file or directory (os error 2)")
```

Really only the `Active: …` line is needed (if it contains 'failed', the last line would be also nice. The rest I dunno yet.)

Mem peak and CPU time would be nice too now that I am looking at it.

##### Flags

###### --full

Alternate names (from: https://web.mit.edu/gnu/doc/html/standards_18.html)
`verbose`: Print more information about progress. Many programs support this.
`all`:'-a' in du, ls, nm, stty, uname, and unexpand. 
`show-all`: '-A' in cat.

```bash

$ argos status --full

Server: [ServerStatus]
Git Root: [URI]
Tracking: 42 repositories

Passing: 20 | Failing: 15 | Skipped: 7

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

```bash

$ argos status --file a/path/file.[txt | md]

```

TODO: Consider output file type. TOML, JSON, and CSV would also be possible with mawu.

###### --directory

Uses `--full` output, if that flag is present or not.

All output is recorded into a single directory with one file per tracked repo and one general file (the short status output).

```bash

$ argos status --directory a/path/

```

TODO: Consider output file type. TOML, JSON, and CSV would also be possible with mawu.


### Config

#### Track

If the supplied name is already tracked, output 'NAME is already tracked! No changes made.'

```bash

$ argos config track PROJECTNAME

```

#### Untrack

If the supplied name is not tracked, output 'NAME not tracked! No changes made.'

```bash

$ argos config untrack PROJECTNAME

```

#### Gitroot

Sets the git-root with no checking if it works (because I don't know how)

```bash

$ argos config git-root ssh:git@server:2222/user/

```

Well, managed to fail2ban myself while playing around with ways of using ssh to confirm the link

However, I found this interesting behaviour in `git clone`

```bash
xqhare@xqhare-desktop ~/A/P/r/argos (v1) [1]> g clone ssh://git@serverle:2222/Xqhare/
Cloning into 'Xqhare'...
error:
error: Invalid repo name
error:
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
xqhare@xqhare-desktop ~/A/P/r/argos (v1) [128]> g clone ssh://git@no_exist:2222/Xqhare
Cloning into 'Xqhare'...
ssh: Could not resolve hostname no_exist: Name or service not known
fatal: Could not read from remote repository.

Please make sure you have the correct access rights
and the repository exists.
```

So just a `.contains(Could not resolve hostname)` could work?
