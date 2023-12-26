use rslint_parser::{parse_module, SyntaxKind, SyntaxNode};
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
    let abs_path = change_to_abs_path(entry_point);

    parse_to_ast(entry_point)
}

fn parse_iterate_module<F: FnMut(&SyntaxNode) -> bool>(content: &String, cb: &mut F) -> () {
    let parse = parse_module(content, 0);
    let mut syntax_node = parse.syntax().first_child();
  
    // println!("parsed AST: {:?}", parse);
  
    loop {
        let mut _node = syntax_node.unwrap();
        let cont = cb(&_node);
  
        if !cont {
            break;
        }
  
        syntax_node = match _node.next_sibling() {
            Some(next) => Some(next),
            _ => break,
        }
    }
  }
  
  
  fn parse_module_imports(content: &String) -> Vec<String> {
    let mut sources = Vec::new();
    let mut _iter = |_node: &SyntaxNode| -> bool {
        if _node.kind() == SyntaxKind::IMPORT_DECL {
            let mut _import_node = _node.first_child();
  
            'import: loop {
                while let Some(_in) = _import_node {
                    if _in.kind() == SyntaxKind::LITERAL {
                        let src = _in
                            .text()
                            .to_string()
                            .replace(&['\'', '\"', ' ', '\t'][..], "")
                            .to_owned();
  
                        println!("src:{:?}", &src);
                        sources.push(src);
  
                        break 'import;
                    }
  
                    _import_node = _in.next_sibling();
                }
            }
        }
  
        return true;
    };
  
    parse_iterate_module(content, &mut _iter);
  
    return sources;
  }
  

fn parse_to_ast(file_path: &String) {
    let contents = fs::read_to_string(file_path).expect("error reading");
    let a = parse_module_imports(&contents);

    println!("this is imported {:?}", a);
}

fn change_to_abs_path(file_path: &String) -> String {
    let relative_path = Path::new(file_path);
    let absolute_path = relative_path.absolutize().unwrap().to_str().unwrap().to_string();

    println!("Absolute path: {}", absolute_path);

    absolute_path
}
