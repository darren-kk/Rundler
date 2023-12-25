use rslint_parser::{parse_module, SyntaxKind};
use path_absolutize::*;

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
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
    let abs_path = change_to_abs_path(entry_point);

    let mut processed_files = HashSet::new();
    let bundled_content = process_file(&abs_path, &mut processed_files);

    let output_file_path = "bundle.js";
    let mut output_file = File::create(output_file_path).expect("Unable to create file");
    output_file.write_all(bundled_content.as_bytes()).expect("Unable to write data");
    println!("Bundle written to {}", output_file_path);

}


fn trim_quotes(s: &str) -> String {
    if s.starts_with('\'') && s.ends_with('\'') || s.starts_with('\"') && s.ends_with('\"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn parse_to_ast(file_path: &String) -> Vec<String> {
    let contents = fs::read_to_string(file_path).expect("error reading");
    let parse = parse_module(&contents, 0);
    let mut imports = Vec::new();

    println!("parsed syntax: {}", parse.syntax());

    for node in parse.syntax().descendants() {
        if node.kind() == SyntaxKind::IMPORT_DECL {
            if let Some(literal) = node.children().find(|n| n.kind() == SyntaxKind::STRING) {
                let import_path = trim_quotes(&literal.text().to_string());
                imports.push(change_to_abs_path(&import_path));
            }
        }
    }

    println!("string contents: {:?}", contents);

    imports
}

fn change_to_abs_path(file_path: &String) -> String {
    let relative_path = Path::new(file_path);
    let absolute_path = relative_path.absolutize().unwrap().to_str().unwrap().to_string();

    println!("Absolute path: {}", absolute_path);

    absolute_path
}

fn process_file(file_path: &String, processed_files: &mut HashSet<String>) -> String {
    if !processed_files.insert(file_path.clone()) {
        return String::new(); // Skip already processed files to avoid infinite loops
    }

    let contents = fs::read_to_string(file_path).expect("error reading file");
    let import_paths = parse_to_ast(file_path);
    let mut combined_content = contents;

    for import_path in import_paths {
        let import_content = process_file(&import_path, processed_files);
        combined_content.push_str(&import_content);
    }

    combined_content
}
