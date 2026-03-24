
Now, while I have set up a CI-Pipeline on my homelab, I am not using it. For it to be useful, I need a few things:

1. Automatic testing
    - Currently I support only running `cargo build` and `cargo test` together manually.
    - This leads to several Issues. Mainly: Not all repos can be tested some can only be built.
    - Some repos will need specialised CI-Pipelines for themselfs, others can share one.
2. Error reporting
    - Currently I am creating a simple log file with the errors of the last run. This file is also overwritten on every run.

While the home lab (with its 8 core 4Ghz and 32Gb RAM) is more than capable to handle several builds in parallel, I think I would purposefully want to limit myself to sequential builds.

Something like BATCH scheduler with Nice=19 - so that other services are preferred.
I really dont care how long the entire run of all repos takes, as long as it finishes them all within a day (24h) and does not impede my nginx servers, gitea etc.
consider `ionice -c 3` (idle) for the runner. Compiling Rust is notoriously I/O intensive

## Ideal Workflow

1. The homelab runs the CI every day.
2. The CI contains a list of repos
3. The CI keeps track of the last commit it tested
    - Commit hash storage
4. If the repo has a new commit since the last test, the CI will build and test the repo.
5. Success / Errors are saved as a file
6. Old files are moved into a archive folder
    - The archive should be cleaned up, lets say keep the last 100 files per repo
        - Should be way overkill, I expect no file to be larger than a few Kb's so saving a few hundred files should not be a problem
        - For arguments sake: Lets say one file will always be 10 Kb (enourmous overestimate). For one repo (100 files) this would be 1.000 kb lets round that up to 1MB.
          - With 1MB per repo, I could test 1024 repos daily and only reach one GB of disk storage used.

BONUS: `cargo clippy` checking

For reporting, I think I will add a readout to nyx (TUI system dashboard).
Simple list of all repos tested, with the status of the last run. (styled in OK and ERROR)

Now I would like to somehow use docker containers to run the tests.
This way I can more easily set up a repeatable testing environment.

I have mainly different ways of calling tests, like all-features; Some repos can be tested using doc tests, others (Talos especially) cannot be tested using doc tests.

I don't know if this is possible, I would want to run a docker container like a simple program, not as a continous service like I do with all my other projects.
IT IS: `docker run --rm -v $(pwd):/app -w /app rust:latest cargo test` executes the tests and immediately destroys the container.
I am sure there is also a way of using docker compose then, or maintaining actual Dockerfiles for repos with specialised environment needs.

Easiest way for me is to hard code how the tests need to run for each repo. This is going to be less upfront work, but more maintenance and it will be more time intensive to add new repos.
Maybe I can define a config file structure that I could read and use to set up the testing.

For most a simple array with the commands to run. (e.g. `[test, build, clippy, doc]` to test it all)
Maybe with a kv store for special requirements or something? (If I build a GUI app using `egui` for example. To build the env needs to be set up with the correct dependencies)
(e.g. `{requires: [repo_name, repo2]}`)

### In repo useage

Now, the most simple way I currently see is to have a simple configuration file containing only a list of repos. (e.g. `nabu, talos`)
From here, the repo can be cloned and checked if it has new commits and if repo specific config files are available.

These repo specific config files would then specifiy the commands to run or the needed environment to run them.

E.g.:

Basic config file:
```json
[
  "test",
  "build",
  "clippy" // short for clippy fix integration
]
OR
[
  "all"
]
```

If present this specifies the environment needed:
(Just as an example, testing also has the need for a dev-dependency; clippy needs nothing, and build doesn't request a release build, but only requires 'egui')
```json
{
  "test": {
    "args": ["all-features", "no-doc"]
    "requires": ["nabu", "athena"]
    "requires_ext": ["path_to_dockerfile"]
  },
  build": {
    requires": ["athena"]
    "requires_ext": ["path_to_dockerfile"]
  }
}
```

For dependency resolving, it would pull the current git mirror status (at worst 8h behind my gitea server) because of the way Cargo.toml works (and the way I have set up my homelab).
It still makes sense to try and build from the bottom of a tree upwards (if a projects depends on another) and not continue upwards if a repo fails to build. (Saving CPU time).

Caveat: 
1. Gitea code of athena is pulled, tested and works. GitHub is not updated yet.
2. Nabu relies on a newly added athena feature, not on github yet, leading to failing tests. (Lets say these failures are fixed with the athena update from 1)
3. All repos depending on Nabu are skipped

NEXT DAY: Because GitHub is updated now, all repos pass.

So internal and external dependencies are needed, but treated differently.

ANOTHER POINT on internal dependencies is that the resolver for the tree would be the most complex part.

Internal dependencies would only be needed to be added to the docker runtime executing the code.

## Backend

As its all on the homelab, it can be easily integrated to use the already set up git profile for ssh pulling.

### Nyx integration

Nyx is already integrated with the readout of Lasa and Shamash. In this case it would be almost exactly the same as the Lasa support (xff file readout and display)

## Wild ideas

Add support for automatic formatting with `cargo fmt`
Add support for running `cargo clippy fix...` automatically
Add support for automatic updates with `cargo update`

Add support for commiting and pushing the above.

Of course these tools can break things, so test before and after.

Also remember to mark the git commits clearly

- git commit -a -m "CI: chore: auto dependencies update"
- git commit -a -m "CI: chore: auto code formatting"
- git commit -a -m "CI: chore: auto cargo clippy fix"

Workflow:

- Argus runs daily.
- Identifies new commits.
- Executes the test pipeline.
- If successful, it applies "Wild Ideas" (fmt, fix, update).
- Re-tests.
- Only if test pipeline still successful, commits and pushes.
- Notifies nyx that the repo is both Healthy and Up-to-Date.

## Docker permissions

Can be easily solved with something like:

```bash
#!/bin/bash
# 1. Capture the host runner's User ID and Group ID
HOST_USER="$(id -u):$(id -g)"
# 2. Define a shared Cargo cache directory on the host machine
# This prevents downloading the same crates 100 times for 100 repos.
ARGUS_CARGO_CACHE="/var/lib/argus/cargo_cache"
mkdir -p "$ARGUS_CARGO_CACHE"
# 3. Execute the ephemeral container
docker run --rm \
  --user "$HOST_USER" \
  -v "$(pwd):/app" \
  -v "$ARGUS_CARGO_CACHE:/usr/local/cargo" \
  -w /app \
  -e HOME=/tmp \
  rust:latest \
  cargo test --all-features
```

## Gemini output:
Technical Synthesis of the "Argus" Runner


1. The Docker Strategy (Ephemeral Containers):
  You can absolutely run Docker like a "simple program." Using docker run --rm -v $(pwd):/app -w /app rust:latest cargo test executes the tests and immediately destroys the container, leaving only the test results on your host.
   * GUI Dependencies: For repos like those using egui, you could maintain a specialized Dockerfile (e.g., Argus-GUI) with libwayland, libx11, etc., pre-installed.


2. Configuration Structure:
  To avoid the maintenance burden of hardcoding, a simple text-based configuration (staying true to your zero-dependency rule) would work.

  Proposed argus.conf (Simple Key-Value/Array):


1     repo: nabu
2     image: rust-std
3     pipeline: [build, test, clippy]
4
5     repo: talos
6     image: rust-gui
7     pipeline: [build, test, clippy]
8     no-doc-tests: true
  A basic Rust parser using line.split(':') would be sufficient to drive the logic.


3. The Sequential Batch Scheduler:
  The runner would act as a simple loop:
   - Fetch list of repos.
   - Check local last_sha file.
   - If mismatch, docker run the commands in the pipeline array sequentially.
   - Capture stdout/stderr to logs/<repo>/<timestamp>.log.
   - Update nyx status file (e.g., status.xff or status.json).


4. State & Archiving:
  Using a directory structure like /var/lib/argus/state/ for SHAs and /var/log/argus/archive/ for logs makes rotation trivial:


1     # Simple rotation logic for the runner
2     ls -t /var/log/argus/archive/nabu/ | tail -n +101 | xargs rm


Your "Special Requirements" Logic
The {requires: [repo_name, repo2]} idea is excellent for dependency chains. If "Hermes" (IPC) needs to be built before "Nabu" (which uses it), Argus can resolve this by checking the success status of the requirements before starting
the dependent build.

### Notes on output

1. Docker it is then
2. Instead of a fully custom, human readable toml/yaml like format, I will just use `Mawu` for JSON

## Dependency sorting

Ok, this is a big one.
If a repo requires another repo, we need to make sure that the required repo is built before the depending repo.

This will be hard, and everything I can do to make it simpler is on the table (performance is strictly secondary for now)

1. Loop through all repos
  - For each, check if `config.json` exists
  - If yes, check if `requires` key exists and return its value
  - If no, return empty array

Now we should have a structure that looks like this:

```json
{
  "repo1": ["repo2", "repo3"],
  "repo2": [],
  "repo3": ["repo4"],
}
```

As rusts cargo does not allow cyclic dependencies, I can just ignore them for now.

The structure I have produced, already looks ready to go for "Kahn's algorithm" (https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm). Need to implement that inside athena though.

I HAVE RETURNED WITH GREAT NEWS! Kahns is implemented in athena and ready to go, feature flag: sorting

