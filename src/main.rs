use std::{env, fs, process};
use colorex::Colorize;
use codegen::Interpreter;
use lexer::Lexer;
use parser::parse;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("{} {}",
                  "Usage:".color("255,71,71"),
                  "loa <command> [arguments]");

        eprintln!("{}",
                  "Commands:".color("145,161,2"));

        eprintln!("  {}    {}",
                  "run <file>".color("38,139,235"),
                  "Execute the specified Loa file");

        eprintln!("  {}     {}",
                  "--version".color("38,139,235"),
                  "Show the CLI version");
        process::exit(1);
    }

    match args[1].as_str() {
        "--version" | "-V" => {
            println!("{}",
                     VERSION.color("2,161,47"));
            return;
        }
        "run" => unsafe {
            if args.len() < 3 {
                eprintln!("{} {}",
                          "Usage:".color("255,71,71"),
                          "loa run <file>");
                process::exit(1);
            }

            let file_path = &args[2];
            run_loa_file(file_path);
        }
        "repl" => repl_mode(),
        "help" => {
            println!("{}", "Options:".color("145,161,2"));
            println!("      {}\n      {}\n      {}\n",
                     "run <file>".color("38,139,235"),
                     "repl".color("38,139,235"),
                     "Run the Loa code.");

            println!("{}", "Commands:".color("145,161,2"));
            println!("      {}\n      {}\n",
                     "-V, --version".color("38,139,235"),
                     "Verified the version of the Loa interpreter.");
            return;
        }
        _ => {
            eprintln!("{} {}",
                      "Unknown command:".color("255,71,71"),
                      args[1]);
            eprintln!("{}",
                      "Use 'loa --version' or 'loa run <file>'".color("145,161,2"));
            process::exit(1);
        }
    }
}

unsafe fn run_loa_file(file_path: &str) {
    let code = fs::read_to_string(file_path).expect("Failed to read file");

    let mut lexer = Lexer::new(&code);
    let tokens = lexer.tokenize();

    let ast = parse(&tokens).expect("Failed to parse Loa code");

    // println!("code: \n{}\n", code);


    // println!("AST:\n{:#?}", ast);

    let mut interpreter = Interpreter::new();
    interpreter.execute(&ast);
}


fn repl_mode() {
    use std::io::{self, Write};

    let mut interpreter = Interpreter::new();

    loop {
        print!("Loa > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim();

        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        let mut lexer = Lexer::new(trimmed);
        let tokens = lexer.tokenize();

        if tokens.is_empty() {
            continue;
        }

        match parse(&tokens) {
            Some(ast) => {
                interpreter.execute(&ast);
            }
            None => {
                println!("Parse error: failed to parse input.");
            }
        }
    }
}
