
While having argos run (or fail to do so) I noticed several issues to fix:

1. Local files
2. Error reporting

## Local Files

The biggest issue:

Editing of the `repo_list.json`.

Need to remember the location (inside .local/argos), cd to it and edit.
Bodge fixed by a symlink into the home dir. (to argos_repo_list.json).

## Error Reporting

Further reading back out a status was bad; Fixed with the scripts `ci_report.sh` and `show_ci_failures.sh`


## Things I Like

The caches work, the internally needed data is in a good place (organisation wise) really.

## Things to Rework / Change

Firstly, a move to a real CLI tool would be awesome.

```bash
$ argos
All passing: [True/False]
Installed location : [PATH]
Tracked Projects: [LIST PROJECT NAMES]
```

```bash
$ argos add PROJECTNAME
Repolist (@ PATH) updated to track "PROJECTNAME"
Make sure its available at GITROOT

$ argos remove PROJECTNAME
Repolist (@ PATH) no longer tracks "PROJECTNAME"
```

```bash
$ argos status
[List of all projects, each project as]
Project: [NAME]
All passing: [TRUE/FALSE]
Integrations: [LIST of TESTNAME:TRUE/FALSE]
Failing: [LIST of TESTNAME:TEXTOUTPUT CMD]
```

```bash
$ argos failing
[List of all failing projects, each project as]
Project: [NAME]
Failing: [LIST of TESTNAME:TEXTOUTPUT CMD]
```

```bash
$ argos report FILEPATH
# Complete output for each project, FILEPATH as a DIR only; One file per project
$ argos report FILEPATH --onefile
# Complete output for each project, FILEPATH as a FILE only, one giant file for all projects
```

### Better error reporting

output text captured does not really say what is failing.
