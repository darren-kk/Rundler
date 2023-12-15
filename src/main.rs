use rslint_parser::{parse_module, SyntaxKind};
use path_absolutize::*;

use std::path::Path;
use std::fs;
use std::env;



// path crate example

// fn main() {
//     let relative_path = Path::new("./test/index.js");
//     // 인자로 받은 상대 경로 기반의 Path 인스턴스를 생성합니다!
//     let absolute_path = relative_path.absolutize().unwrap();
//     // 해당 경로를 절대 경로로 변경해줍니다! 경로에 문제가 있다면 에러를 던져줍니다!(panic으로 실행을 멈춰줍니다!)
//     println!("Absolute path: {}", absolute_path.display());
// }


// AST parse example

// fn parse_to_ast(file_path: &String) {
//     let contents = fs::read_to_string(file_path).expect("error reading");
//     // 인자로 받은 경로의 내용을 문자열로 읽어줍니다~!!
//     // 메소드가 자체적으로 에러를 반환하기 떄문에 파일이 없거나, 접근 권한이 없는 파일 등의 경우 에러를 던짐
//     // expect를 통해 에러를 핸들링(panic으로 실행을 멈춰줍니다!)
//     let parse = parse_module(&contents, 0);

//     println!("string contents: {:?}", contents);
//     println!("parsed AST: {:?}", parse);
// }

// fn main() {
//     let args: Vec<String> = env::args().collect();
//     //command line arguments 를 인식 할 수 있도록 제공해주는 라이브러리
//     // args()는 cli의 인자를 수집하고 iterator를 반환
//     // collect()가 iterator를 배열로 변환(정확히는 벡터)

//     println!("this is argument: {:?}", args);
//     // 벡터로 변환된 배열을 보여줍니다~

//     parse_to_ast(&String::from(&args[1]));
//     // 반환된 iterator인 args의 첫번째 인자는 파일 스스로의 경로이나
//     // cargo를 통해 프로그램 실행 -> cargo가 파일을 컴파일 후에 target/debug에 위치한 결과물을 실행
// }



// 상단 두개 합치기

fn main() {
    let args: Vec<String> = env::args().collect();

    let abs_path = change_to_abs_path(&String::from(&args[1]));

    parse_to_ast(&abs_path);
}

fn parse_to_ast(file_path: &String) {
    let contents = fs::read_to_string(file_path).expect("error reading");
    let parse = parse_module(&contents, 0);

    println!("string contents: {:?}", contents);
    println!("parsed AST: {:?}", parse);
}

fn change_to_abs_path(file_path: &String) -> String {
    let relative_path = Path::new(file_path);
    let absolute_path = relative_path.absolutize().unwrap().to_str().unwrap().to_string();


    absolute_path
}

// import 구문 찾기

// fn main() {
//     let args: Vec<String> = env::args().collect();
//     let entry_point = args.get(1).expect("No entry point specified");
//     let abs_path = change_to_abs_path(&String::from(&args[1]));

//     println!("Entry point!: {}", &entry_point);
//     parse_to_ast(&abs_path);
// }


// fn parse_to_ast(file_path: &String) {
//     let contents = fs::read_to_string(file_path).expect("error reading");
//     let parse = parse_module(&contents, 0);
//     println!("parsed syntax: {}", parse.syntax());

//     for node in parse.syntax().descendants() {
//         if node.kind() == SyntaxKind::IMPORT_DECL {
//             println!("node: {}", node);
//             if let Some(literal) = node.children().find(|n| n.kind() == SyntaxKind::LITERAL) {
//                 println!("literal: {}", literal);
//             }
//         }
//     }

//     println!("string contents: {:?}", contents);
// }

// fn change_to_abs_path(file_path: &String) -> String {
//     let relative_path = Path::new(file_path);
//     let absolute_path = relative_path.absolutize().unwrap().to_str().unwrap().to_string();

//     println!("Absolute path: {}", absolute_path);

//     absolute_path
// }
