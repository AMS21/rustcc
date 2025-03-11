use clap::Arg;
use clap::ArgAction;
use clap::Command;
use clap::ValueHint;
use clap::crate_authors;
use clap::crate_description;
use clap::crate_name;
use clap::crate_version;

pub const ARG_INPUT_FILE: &str = "source_file";
pub const ARG_PRINT_TOKENS: &str = "PRINT_TOKENS";

pub const GROUP_DEBUG: &str = "debug";

pub fn command_line() -> Command {
    Command::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::new(ARG_INPUT_FILE)
                .required(true)
                .help("The source file to compile")
                .value_hint(ValueHint::FilePath),
        )
        .arg(
            Arg::new(ARG_PRINT_TOKENS)
                .long("print-tokens")
                .action(ArgAction::SetTrue)
                .help("Print all tokens")
                .group(GROUP_DEBUG),
        )
}
