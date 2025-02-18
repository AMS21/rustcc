mod ast;
mod lexer;
mod parser;
mod token;

fn main() {
    // With no command line arguments, print usage
    if std::env::args().count() == 1 {
        println!("Usage: rustcc <source file>");
        return;
    }

    // Get the first command line argument as the file path
    let file_path = std::env::args().nth(1).unwrap();

    // Read the file
    let source = std::fs::read_to_string(&file_path).expect("Failed to read file");

    // Create a lexer
    let mut lexer = lexer::Lexer::new(&source);

    // Print all tokens
    loop {
        let token = lexer.next_token();
        println!("{:?}", token);

        if token == token::Token::EndOfFile {
            break;
        }
    }
}
