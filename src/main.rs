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
    let mut collected = collect_modules(result);
    let module_map = to_module_map(&mut collected);
    let runtime = add_runtime(&module_map, &abs_path);

    println!("result {:?}", collected);
    println!("module map {:?}", module_map);
    println!("runtime format {:?}", runtime);
}

fn change_to_abs_path(file_path: &String) -> String {
    let relative_path = Path::new(file_path);
    let absolute_path = relative_path.absolutize().unwrap().to_str().unwrap().to_owned();

    println!("Absolute path: {}", absolute_path);

    absolute_path
}

fn new_module(file_path: &String) -> Module {
    let abs_path = change_to_abs_path(file_path);
    let contents = fs::read_to_string(file_path).expect("error reading");
    let imports = parse_module_imports(&contents);
    let mut mods = Vec::new();

    for import in imports {
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

fn transform_module_interface(module: &mut Module) {
    let mod_copy = copy_module(&module);
    let mut _iter = |_node: &SyntaxNode| -> bool {
        if _node.kind() == SyntaxKind::IMPORT_DECL {
            let mut _import_node = _node.first_child();

            'import: loop {
                while let Some(_in) = _import_node {
                    match _in.kind() {
                        SyntaxKind::LITERAL => {
                            let src = _in.text().to_string().replace(&['\'', '\"', ' ', '\t'][..], "").to_owned();
                            let abs_path = Path::new(&module.file_path).parent().unwrap().join(src).absolutize().unwrap().to_str().unwrap().to_string();
                            let var_name = _in.prev_sibling().unwrap().text().to_string();
                            let new_stmt = format!("const {{default: {}}} = require(\"{}\");", var_name, abs_path);
                            module.module_content = module.module_content.replace(&_node.text().to_string(), &new_stmt);
                            break 'import;
                        }

                        SyntaxKind::NAMED_IMPORTS => {
                            let mut vars = _in.text().to_string().replace(&['{', '}'][..], "");
                            if let Some(v) = _in.prev_sibling() {
                                let mut new_var = String::from("default: ");
                                new_var.push_str(&v.text().to_string());
                                new_var.push_str(&format!(",{}", vars));
                                vars = new_var;
                            }
                            let src = _in.next_sibling().unwrap().text().to_string().replace(&['\'', '\"', ' ', '\t'][..], "").to_owned();
                            let abs_path = Path::new(&module.file_path).parent().unwrap().join(src).absolutize().unwrap().to_str().unwrap().to_string();
                            let new_stmt = format!("const {{{}}} = require(\"{}\");", vars, abs_path);
                            module.module_content = module.module_content.replace(&_node.text().to_string(), &new_stmt);
                            break 'import;
                        }
                        _ => _import_node = _in.next_sibling(),
                    }
                }

                break 'import;
            }
        } else if _node.kind() == SyntaxKind::EXPORT_DECL {
            // export { name }
            // or
            // export const name = obj;
            let mut _export_node = _node.first_child();
            'export: loop {
                while let Some(_en) = _export_node {
                    match _en.kind() {
                        SyntaxKind::EXPORT_NAMED => {
                            // export { name }
                            let vars_str = _en.text().to_string().replace(&['{', '}'][..], "");
                            let vars = vars_str.split(",");
                            let mut new_stmt = String::from("");
                            for var in vars {
                                let var_trim = var.replace(&[' ', ';'][..], "");
                                new_stmt.push_str(
                                    &format!("exports.{} = {};\n", var_trim, var_trim)[..],
                                );
                            }
                            module.module_content = module
                                .module_content
                                .replace(&_node.text().to_string(), &new_stmt);
                            break 'export;
                        }
                        SyntaxKind::VAR_DECL => {
                            // export const name = obj;
                            let mut new_stmt = String::from("");
                            let stmt = _en
                                .text()
                                .to_string()
                                .replace(&[' ', ';'][..], "")
                                .replace("let", "")
                                .replace("const", "")
                                .replace("var", "");
                            let decls = stmt.split(",");
                            for decl in decls {
                                let mut decl_split = decl.split("=");
                                let (name, value) =
                                    (decl_split.next().unwrap(), decl_split.next().unwrap());
                                new_stmt.push_str(&format!("exports.{} = {};\n", name, value)[..]);
                            }
                            module.module_content = module
                                .module_content
                                .replace(&_node.text().to_string(), &new_stmt);
                            break 'export;
                        }
                        _ => _export_node = _en.next_sibling(),
                    }
                }
                break 'export;
            }
        } else if _node.kind() == SyntaxKind::EXPORT_DEFAULT_EXPR {
            // export default name
            let var = _node.first_child().unwrap().text();
            let new_stmt = String::from(format!("exports.default = {};\n", var));
            module.module_content = module
                .module_content
                .replace(&_node.text().to_string(), &new_stmt);
        } else {
            // println!("{:?}", _node);
        }
        return true;
    };

    parse_iterate_module(&mod_copy.module_content.to_string(), &mut _iter);
}

fn to_module_map(modules: &mut Vec<Module>) -> String {
    let mut module_map = String::from("{");

    for module in modules.iter_mut() {
        transform_module_interface(module);

        module_map.push_str(
            &format!(
                "\"{}\": function(exports, require) {{  {} }},",
                module.file_path, module.module_content
            )[..],
        );
    }

    module_map.push_str("}");

    return module_map;
}

fn add_runtime(module_map: &String, entry_point: &String) -> String {
    let runtime = String::from(format!(
        "
    (function() {{
            // Modules will be added here
            const modules = {};
            const entry = \"{}\";

            // Module cache to store instantiated modules
            const moduleCache = {{}};

            // Custom require function to load modules
            const require = moduleName => {{
                // Check if module is in cache
                if (moduleCache[moduleName]) {{
                    return moduleCache[moduleName].exports;
                }}

                // If not, initialize and load the module
                const module = {{ exports: {{}} }};
                moduleCache[moduleName] = module;

                try {{
                    modules[moduleName](module.exports, require);
                }} catch (error) {{
                    throw new Error('Module load error in ' + moduleName + ': ' + error.message);
                }}

                // Return the exports from the module
                return module.exports;
            }};

            // Start the application by requiring the entry module
            try {{
                require(entry);
            }} catch (error) {{
                console.error('Application failed to start: ' + error.message);
            }}
        }})();
    ",
        module_map, entry_point
    ));

    return runtime;
}
