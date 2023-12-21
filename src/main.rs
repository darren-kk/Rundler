use rslint_parser::{parse_module, SyntaxKind};
use path_absolutize::*;

use std::path::Path;
use std::fs;
use std::env;

#[derive(Debug, Clone)]
struct Module {
    file_path: String,
    module_content: String,
    dependencies: Vec<Module>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let entry_point = args.get(1).expect("No entry point specified");
    let abs_path = change_to_abs_path(&String::from(&args[1]));

    println!("Entry point!: {}", &entry_point);
    parse_to_ast(&abs_path);
}


fn parse_to_ast(file_path: &String) {
    let contents = fs::read_to_string(file_path).expect("error reading");
    let parse = parse_module(&contents, 0);

    println!("parsed syntax: {}", parse.syntax());

    for node in parse.syntax().descendants() {
        if node.kind() == SyntaxKind::IMPORT_DECL {
            println!("node: {}", node);
            if let Some(literal) = node.children().find(|n| n.kind() == SyntaxKind::LITERAL) {
                println!("literal: {}", change_to_abs_path(&literal.to_string()));
                println!("literal: {}", literal);
            }
        }
    }

    println!("string contents: {:?}", contents);
}

fn change_to_abs_path(file_path: &String) -> String {
    let relative_path = Path::new(file_path);
    let absolute_path = relative_path.absolutize().unwrap().to_str().unwrap().to_string();

    println!("Absolute path: {}", absolute_path);

    absolute_path
}
