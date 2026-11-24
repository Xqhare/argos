# Eshu

A zero-dependency Rust library for building robust CLI tools with automatic help and man-page generation.

It follows my "All code written by me or part of rust's standard library and libc" philosophy.
You can learn more about that [here](https://blog.xqhare.net/posts/why_solve_problems/).

## Roadmap

`Eshu` uses my [nomos](https://github.com/xqhare/nomos) project management system.

The roadmap for this project can be found in the [nomos.md](nomos.md) file.

All nomos files follow the syntax defined [here](https://github.com/Xqhare/nomos/blob/master/spec/).

## Features

- _**No dependencies**_: All code is written by me or part of std.
- _**`XffValue` Integration**_: Easy casting of returned values into `XffValues`.
- _**`Nemesis` Error Handling**_: Easy error handling with `Nemesis`. First of all my projects to make use of my [error handling library](https://github.com/xqhare/nemesis).
- _**Extensive Testing and Documentation**_: Extensive testing and documentation.

## Naming

As with all my projects, `Eshu` is named after an ancient deity.
Learn more about my naming scheme [here](https://blog.xqhare.net/posts/explaining_the_pantheon/).

`Eshu` is an East African divine spirit of communication and language, acting as a messenger between humans and the deities.

## Quick Reference

```bash
$ tool_name [-a] [-bC] [-d required_arg] [-e [optional_arg]] [-f=required_arg] [-g[=optional_arg]] [-h arg0] [-h argN] [-i key=value] [--long-flags] [--long-flag[=optional_arg]] [--long-flag=required_arg] [--flag_with=key=value] [-- optional_positional_args] [subcommand] [subcommand_args_and_flags]
```

- Short Flags (Upper and Lowercase)
- Long Flags
- Grouped Flags
- Key Value Pairs
- Named Key Value Pairs
- Flags with optional or required arguments
- End-Of-Flags Marker (`--`)
- Positional Arguments
- Subcommands

> [!NOTE]
> Partial long flags are supported as long as they are unique from other long flags.
>
> For example, `--long` and `--long-flag` are both valid ways to call `--long-flag` as long as there is no other flag beginning with `--long`.

> [!note]
> All flags accepting an argument can store any number of arguments.
> 
> For example, `--long-flag arg0 arg1 arg2` is equivalent to `--long-flag arg0 --long-flag arg1 --long-flag arg2`.

> [!note]
> A Flag accepting an argument can be part of grouped flags if only one is used and placed at the end of the group.
>
> For example, `-ab -C=Argument` can be written as `-abC=Argument`.

### Always Available / in-Use Flags

These flags will only print to `stdout` and exit.

- `--help` and `-h`: Prints the help message
- `--version`: Prints the version number (Note: `-v` and `-V` are free to be used for custom flags)

> [!NOTE] 
> These flags cannot be overwritten by other flags. Usage of them is mandatory and automatic.

### Flag Name Definitions

`Eshu` encourages you to make use of flag names defined by the [GNU Coding Standards](https://web.mit.edu/gnu/doc/html/standards_18.html).

## Supported CLI Syntax

It is important to know that there is no formalised syntax agreed upon for CLI tools.
If you want to dive into the details, I have written a blog post [A Guide to the Syntax of CLI Tooling: Modern Problems Require Old Solutions](https://blog.xqhare.net/posts/cli_modern_problems_old_solutions/).

Please note, I use the term "flag" to refer to all elements that start with a dash `-` or two dashes `--`.
The term "key value pair" refers to all elements that contain at least one equal sign `=`. Key value pairs may never have more than two equal signs. A pair may only contain two equal signs if it is an attached value.
The term "argument" refers to all elements that are passed to a specific flag or command. These may be required or optional.

In general terms the syntax offered by `Eshu` is:

```bash
$ tool_name [-a] [-bC] [-d required_arg] [-e [optional_arg]] [-f=required_arg] [-g[=optional_arg]] [-h arg0] [-h argN] [-i key=value] [--long-flags] [--long-flag[=optional_arg]] [--long-flag=required_arg] [--flag_with=key=value] [--] [optional_positional_args] [subcommand] [subcommand_args]
```

`Eshu` also supports partial long flags, as long as they are unique from other long flags.

A flag accepting an argument can store any number of arguments.
Also, a flag accepting an argument can be part of grouped flags if only one is used and placed at the end of the group.

In other terms, `Eshu` supports the POSIX and GNU syntax in general, along with a few modern best practices.

## Usage

`Eshu` only has two ways of adding functionality to your tool.

- Flags
- Commands

### Unified Flag Theory

Often the term flag, option, and argument are thrown around interchangeably, and meaning different things to different people.

`Eshu` simplifies this by using a unified flag for its API.

Any flag may be defined to be able to hold values, OR key value pairs as well as defining these values as being either required or optional.

### Best Practices Using `Eshu`

`Eshu` is best used by following the general best practices for CLI tools.
You can find some [here](https://blog.xqhare.net/posts/cli_modern_problems_old_solutions/#further-reading).

`Eshu` encourages you to make use of flag names defined by the [GNU Coding Standards](https://web.mit.edu/gnu/doc/html/standards_18.html).

### Importing

Add the following to your `Cargo.toml`:

```toml
[dependencies]
eshu = { git = "https://github.com/xqhare/eshu" }
```

### Example

```rust
use eshu::{Cli, CliFlag, StoreSyntax, StoreType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define flags and options
    let verbose_flag = CliFlag::new("verbose")
        .with_flag_char('v')
        .with_about("Verbose mode", "Print detailed debug information.")
        .build()?;

    let file_flag = CliFlag::new("file")
        .with_flag_char('f')
        .with_about("Target file path", "Specify the path to the input file.")
        .with_store(StoreType::Value, StoreSyntax::Detached)
        .build()?;

    // 2. Configure and parse CLI
    let cli = Cli::new("my-tool")
        .with_version("1.0.0")
        .with_about("A command line interface built with Eshu")
        .add_flag(verbose_flag)
        .add_flag(file_flag)
        .parse();

    // 3. Extract and use parsed values
    if cli.is_flag_entered("verbose") {
        println!("Verbose logging is enabled.");
    }

    if let Some(store) = cli.get_flag_store("file") {
        if let Some(files) = store.as_value() {
            println!("Processing file: {:?}", files.first());
        }
    }

    // Read any stray positional arguments
    let positionals = cli.get_stray_positional_args();
    if !positionals.is_empty() {
        println!("Additional inputs: {:?}", positionals);
    }

    Ok(())
}
```

## License

`Eshu` is distributed under the [MIT](https://github.com/xqhare/eshu/blob/master/LICENSE) license.

## Contributing

See [CONTRIBUTING](https://github.com/xqhare/eshu/blob/master/CONTRIBUTING.md) for contribution guidelines.
