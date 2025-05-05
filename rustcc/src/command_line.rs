use clap::{
    Arg, ArgAction, Command, ValueHint, crate_authors, crate_description, crate_name, crate_version,
};

pub const ARG_INPUT_FILE: &str = "source_file";
pub const ARG_PRINT_TOKENS: &str = "PRINT_TOKENS";
pub const ARG_PRINT_AST: &str = "PRINT_AST";
pub const ARG_PRINT_IR: &str = "PRINT_IR";

#[must_use]
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
                .help("Print all tokens"),
        )
        .arg(
            Arg::new(ARG_PRINT_AST)
                .long("print-ast")
                .action(ArgAction::SetTrue)
                .help("Print the abstract syntax tree"),
        )
        .arg(
            Arg::new(ARG_PRINT_IR)
                .long("print-ir")
                .action(ArgAction::SetTrue)
                .help("Print the LLVM intermediate representation"),
        )
}
