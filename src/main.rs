use path_absolutize::*;
use rslint_parser::{parse_module, SyntaxKind, SyntaxNode};

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

    let result = new_module(&abs_path);
    let collected = collect_modules(result);

    println!("result {:?}", collected);
}

fn change_to_abs_path(file_path: &String) -> String {
    let relative_path = Path::new(file_path);
    let absolute_path = relative_path.absolutize().unwrap().to_str().unwrap().to_string();

    println!("Absolute path: {}", absolute_path);

    absolute_path
}

fn new_module(file_path: &String) -> Module {
    let abs_path = change_to_abs_path(file_path);
    let contents = fs::read_to_string(file_path).expect("error reading");
    let imports = parse_module_imports(&contents);
    let mut mods = Vec::new();

    for import in imports {
        // get dirname + imported file path
        let path = Path::new(file_path).parent().unwrap().join(import);
        match path.to_str() {
            Some(s) => mods.push(new_module(&String::from(s))),
            None => panic!("cannot convert path to string"),
        }
    }

    println!("module complete: {abs_path}");

    return Module {
        file_path: abs_path,
        module_content: contents,
        dependencies: mods,
    };
}

fn copy_module(module: &Module) -> Module {
    let mut dependencies: Vec<Module> = Vec::new();

    for dep in &module.dependencies {
        dependencies.push(copy_module(dep));
    }

    let _module = Module {
        file_path: module.file_path.clone(),
        module_content: module.module_content.clone(),
        dependencies: dependencies,
    };

    return _module;
}

fn collect_modules(graph: Module) -> Vec<Module> {
    let mut mods: Vec<Module> = Vec::new();

    collect(graph, &mut mods);

    fn collect(module: Module, mods: &mut Vec<Module>) {
        mods.push(copy_module(&module));

        for dep in module.dependencies {
            collect(dep, mods);
        }
    }

    return mods;
}

fn parse_iterate_module<F: FnMut(&SyntaxNode) -> bool>(content: &String, cb: &mut F) -> () {
    let parse = parse_module(content, 0);
    let mut syntax_node = parse.syntax().first_child();

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
