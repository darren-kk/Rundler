use path_absolutize::*;
use std::path::Path;


// path crate example

fn main() {
    let relative_path = Path::new("./test/shapes/index.js");
    // 인자로 받은 상대 경로 기반의 Path 인스턴스를 생성합니다!
    let absolute_path = relative_path.absolutize().unwrap();
    // 해당 경로를 절대 경로로 변경해줍니다! expect 메소드와 다르게 별도의 에러 메시지를 던지진 않지만,
    // 마찬가지로 panic!을 일으킵니다.
    println!("Absolute path: {}", absolute_path.display());
}
