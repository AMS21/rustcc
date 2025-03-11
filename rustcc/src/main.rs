use command_line::command_line;

mod ast;
mod command_line;
mod lexer;
mod parser;
mod token;

fn main() {
    // Handle command line arguments
    let command_line_matches = command_line::command_line().get_matches();

    // Get the first command line argument as the file path
    let file_path: &String = command_line_matches
        .get_one(command_line::ARG_INPUT_FILE)
        .unwrap();

    // Read the file
    let source = std::fs::read_to_string(file_path).expect("Failed to read file");

    // Create a lexer
    let mut lexer = lexer::Lexer::new(&source);

    // Print all tokens
    if command_line_matches.get_flag(command_line::ARG_PRINT_TOKENS) {
        loop {
            let token = lexer.next_token();
            println!("{:?}", token);

            if token == token::Token::EndOfFile {
                break;
            }
        }
    }
}
