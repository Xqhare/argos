
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

A move to a real CLI tool would be awesome.

See cli_design.md

### Better error reporting

Output text captured does not really say what is failing.

## Performance

I have built in a sleep of one minute between repos to allow the homelab to work on more important work even while argos is running.

While a nice consideration, it comes with drawbacks.

I now think that relying on the CPU scheduler alone would already be more than sufficient .
