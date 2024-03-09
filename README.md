# Rundler

Rust로 만든 JS 번들러입니다.

Entry point와 output 경로를 입력하면, 코드를 분석해 모듈 의존성 그래프의 생성 및 하나의 파일로 번들링을 해줍니다.

ESM 기반의 모듈 시스템을 CJS로 변경 및 해당 CJS 구문들이 브라우저에서 잘 실행될 수 있도록 번들링 됩니다.

# Contents

- [Motivation](#motivation)
- [Flow-chart](#flow-chart)
- [crates](#crates)
- [Challenges](#challenges)
- [개선방향](#개선방향)
- [Memoir](#memoir)
  <br></br>

# Motivation

rust 언어 학습을 목적으로 언어에 대한 이해와 실제 사용을 통한 학습을 진행하기 위해 미니 프로젝트를 진행하였습니다.

제가 학습해왔던 JS와 웹 기반 개발 생태계와 거리가 멀지 않았으면 좋겠다는 생각에 웹개발에서 자주, 또 필수적으로 쓰이는 번들러를 만들고자 하였으며, 이를 통해 번들러에 대한 깊은 이해, 번들링 과정, 기타 다른 번들러의 동작방식(webpack, vite)에 대하여 깊이 학습 할 수 있었습니다.

보다 자세한 개발과정 및 각 로직별 흐름과 다른 번들러와의 비교는 아래 링크에서 확인 하실 수 있습니다.

[Rundler 구현 기록](https://poised-moon-73b.notion.site/Toy-Bundler-project-254fdf0acc3247ccbaaf23cfdad80085?pvs=4)

# Flow Chart

<img width="1054" alt="스크린샷 2024-01-11 오후 7 27 37" src="https://github.com/darren-kk/Jaenitel-client/assets/111283378/427f8dd8-7c48-4f02-bb5b-fc56674afceb">

# Crates

- path-absolutize
  전달받은 상대 경로를 절대 경로로 변환해주기 위하여 사용한 crate 입니다.
- rslint-parser
  코드 구문을 분석해 AST-node로 parsing 해주기 위하여 사용한 crate입니다.

# Challenges

## JS처럼 함수를 인자로 넘겨줄 순 없을까?

반복되는 코드의 사용을 줄이고, 재사용성이 높은 함수를 만들기 위해 각 AST 노드를 순회하며, 어떤 구문인지 파악한 후 해당 구문에 맞는 동작을 해주는 iterator 함수를 만들고자 하였습니다.

- AST로 변환
- 찾고자 하는 state 찾기(import, export 등)
- 해당 state에 따라 ESM 에서 CJS로 변환 혹은 경로 기반으로 Module 구조체 생성등 다른 로직이 실행
  <br></br>
  <img width="180" alt="스크린샷 2024-01-12 오후 4 05 05" src="https://github.com/darren-kk/Jaenitel-client/assets/111283378/fc80dbff-0f73-4ad2-b11e-ea63eab2453e">

위와 같은 흐름의 함수를 고려했을때, 매 로직마다 별도의 함수를 생성하는건 추상화측면에서 매우 비효율적이라고 생각해 전달받은 파일의 contents를 AST로 변환 후 각 AST 마다 실행되는 콜백을 수행하는 함수를 구현하였습니다.

```jsx
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
```

> Javascript 에서 함수는 일급 객체입니다. 다시 말해, 함수는 변수에 담길 수도 있고, 다른 함수의 반환값으로 사용될 수 도 있으며, 다른 함수에게 인자로 전달 될 수도 있습니다.

> Javascript에서 클로저는 현상입니다. 함수가 생성될때 생성되는 주변의 lexical environment를 기억하고 캡쳐하여 참조가 가능한 현상을 뜻합니다.

rust에선 이 두가지 개념이 모두 통용되지 않습니다.

rust의 함수는 변수에 할당 될 수 없으며, 인자로도, 반환값으로도 전달될 수 없습니다. 또한 생성당시의 주변환경을 기억하는것 역시 rust의 소유권을 생각하면 매우 어려운 일입니다.

그러나 이를 가능하게 해주는것이 rust에서의 closure입니다. Javascript에서의 그것과 다르게 rust의 closure는 함수입니다.

보다 정확히 말하면 일반적인 함수와 비슷하게 생긴 익명 함수입니다.

위의 함수에서 추후에 실행될 콜백에 해당하는 `cb` 를 실행시키기 위해 closure를 활용했습니다.

클로저에 대한 보다 자세한 학습 기록은 아래 링크에서 확인 하실 수 있습니다.

[Closure in Rust 학습 기록](https://poised-moon-73b.notion.site/Closure-in-Rust-8470c30b90b44c99bda1a52f42fb2ae0?pvs=4)

```jsx
let mut _iter = |_node: &SyntaxNode| -> bool {
        if _node.kind() == SyntaxKind::IMPORT_DECL {
            let mut _import_node = _node.first_child();

            'import: loop {
                while let Some(_in) = _import_node {
                    if _in.kind() == SyntaxKind::LITERAL {
                        let src = _in
                            .text()
                            .to_string()
                            .replace(&['\'', '\"', ' ', '\t'][..], "");

                        sources.push(src);

                        break 'import;
                    }

                    _import_node = _in.next_sibling();
                }
            }
        }
```

클로저의 형태는 위와 같습니다. 익명 함수를 선언하여 변수에 할당해줌으로, `parse_iterate_module(content, &mut _iter);` 이렇게 인자로 넘겨주어 각 AST를 순회하며 `cb`을 실행 시킬 수 있었습니다.

## 생성한 구조체를 많은 곳에서 사용하는 방법

러스트는 소유권이라는 강력한 개념으로 언어로써 메모리 안정성과 속도 두가지를 보장해줍니다.

러스트의 큰 장점이지만 동시에 러스트의 높은 러닝커브의 이유이기도 합니다.

저는 파일을 읽어 해당 파일의 내용과, 경로, 의존성 그래프를 담고 있는 구조체를 만들어 해당 구조체를 다양한 곳에서 활용하고자 했습니다.

```rust
struct Module {
    file_path: String,
    module_content: String,
    dependencies: Vec<Module>,
}
```

위 구조체는 아래와 같은 경우에 사용됩니다.

- 중첩된 의존성 Vec를 평탄화 할때
- import 구문을 찾아 ESM → CJS로 변경할때

위 로직들이 수행될때 마다 내부의 dependecies 와 module_content는 변하고 각기 다른 스코프에서 사용되기 때문에 소유권으로 인하여 더이상 접근하지 못하는 경우가 생깁니다.

Javascript처럼 한번 생성해놓으면 편하게 가져다 쓸 수 없으니 어떻게 이부분을 고칠 수 있을지 고민했습니다.

그리하여 내부 값들을 참조해 새로운 Module을 생성 및 변수에 새로 담아주기로 결정했습니다.

```rust
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
```

인자로 하나의 Module 구조체를 받은 뒤, 중첩되어 있는 Module을 재귀적으로 모두 새로운 dependencies에 넣어줍니다.

그 후, clone() 메소드로 새로운 소유권과 함께 Module 구조체를 반환해줍니다.

- 이를 통해 의존성 그래프를 평탄화 해주는 곳에서
  `mods.push(copy_module(&module));`
- ESM을 CJS로 바꿔주는 곳에서
  `let mod_copy = copy_module(&module);`

다음과 같이 모듈을 복사해 사용함으로 소유권으로 제한되던 기능의 확장이 가능했습니다.

여기서 등장하는 소유권에 대한 학습 기록은 아래 링크에서 확인하실 수 있습니다.

[Ownership in Rust 학습기록](https://poised-moon-73b.notion.site/Closure-in-Rust-8470c30b90b44c99bda1a52f42fb2ae0?pvs=4)

## 문자열의 타입이.. 하나가 아니다?

`let absolute_path = relative_path.absolutize().unwrap().to_str().unwrap().to_string;`

위 코드는 전달받은 상대경로를 절대 경로로 변경해주는 로직입니다.

자세히 살펴보면 비슷해보이는 메소드가 두가지 있습니다.

to_str과 to_string입니다.

둘다 문자열을 나타내는 단어이며 둘다 문자열로 바꿔주는 메소드인듯 한데, 왜 두번이나 반복적으로 쓰였을까요?

그건 바로 러스트의 복잡한 string 타입 때문입니다.

Javascript에서 문자열은 단순합니다. String 이라는 원시타입 하나로 모든 문자열을 나타낼 수 있습니다.

이 문자열이 어떻게 메모리에 저장되고 관리되는지 우리는 사실 크게 신경쓰지 않고 개발을 할 수 있습니다.

Garbage Collector가 대신 저희를 위해 메모리를 관리해주기 때문입니다.

러스트에서의 문자열은 약간 다릅니다.

러스트는 정적타입 언어로, 개발자는 미리 어떤 데이터타입을 사용할지 정하여 컴파일시에 필요한 만큼만의 메모리가 할당되게끔 코드를 작성합니다.

그리고 소유권의 개념을 통해 이를 촘촘하게 관리해줍니다.

다시 문자열로 돌아가서, to_str()은 참조형 데이터타입인 &str을 만들며, to_string은 본인 데이터의 소유권을 본인이 지니고 있는 String 데이터 타입입니다.

&str은 참조형이기에 불변성을 띕니다. 읽기 전용 데이터 타입으로 스코프를 넘나들며 소유권의 이동이 불가능합니다.

string은 heap 메모리에 저장되는 유동적으로 변할 수 있는 데이터입니다. 미리 어느정도의 메모리를 heap에 할당해놓기 때문에 중간에 동적으로 수정도 가능하며 소유권의 이동이 자유롭습니다.

<img width="442" alt="스크린샷 2023-12-27 오후 6 45 16" src="https://github.com/darren-kk/Jaenitel-client/assets/111283378/f35e5289-9426-4dc6-b77a-c2b887e465ca">

다시 아까의 로직으로 돌아와서,
`let absolute_path = relative_path.absolutize().unwrap().to_str().unwrap().to_string;`

저는 abosolute_path를 다른 함수에게 인자로도 전달하고 싶었습니다. 그리고 전달받는 파일의 크기에 따라 동적으로 바뀔 예정이라고 생각했습니다.

실제로 해당 로직을 &str로만 선언해주면, 러스트의 친절한 컴파일러는 저희에게 다음과 같은 에러 메시지를 전달합니다.

<img width="777" alt="스크린샷 2024-01-12 오후 4 34 29" src="https://github.com/darren-kk/Jaenitel-client/assets/111283378/697c6cf7-d028-494b-8edf-60d985978d74">

러스트의 문자열 타입에 대한 보다 자세한 학습기록은 아래 링크에서 확인 하실 수 있습니다.

[String in Rust 학습기록](https://poised-moon-73b.notion.site/String-in-Rust-e81bddacbb4b46a096bd26ffdc2c0d20?pvs=4)

# 개선방향

개선방향 및 고민을 말씀 드리기에 앞서, 먼저 제가 만든 번들러와 웹팩, 비트(esbuild)의 절대적 시간 비교를 살펴보시겠습니다.

### my bundler (평균 7ms)

<img width="631" alt="스크린샷 2024-01-03 오후 6 44 02" src="https://github.com/darren-kk/Jaenitel-client/assets/111283378/279ccd3f-4fde-499b-99fe-dab334984526">

### vite (평균 80ms)

<img width="417" alt="스크린샷 2024-01-03 오후 7 08 11" src="https://github.com/darren-kk/Jaenitel-client/assets/111283378/e32e7851-d0b7-45a5-81e8-8eb7de2cc302">

### webpack - production mode (평균 120ms)

<img width="522" alt="스크린샷 2024-01-03 오후 6 56 22" src="https://github.com/darren-kk/Jaenitel-client/assets/111283378/ef85a68a-60d0-4cd5-9824-3bb624ac85e5">

3가지 모두 프로덕션 빌드로 진행 하였으며, 제 번들러의 시간이 가장 빠르게 측정되었습니다.

물론 이 절대적 시간의 비교는 사실 의미가 없습니다. 다른 두 번들러가 제공해주는 기능에 비하여 제 번들러의 기능이 너무 빈약하기 때문입니다.

이에 따라 저는 제 번들러가 어떠한 기능을 어떠한 방향성으로 앞으로 만들어가면 좋을지 고민했습니다.

어떤 측면에서 차이가 있는지 상세히 알아보고 학습한 기록은 아래 링크에서 확인 하실 수 있습니다.

[Webpack bundling process 학습기록](https://poised-moon-73b.notion.site/Webpack-bundling-process-c7f8e79a8e51444caa84d2df4c7beafc?pvs=4)

[Vite bundling process 학습기록](https://poised-moon-73b.notion.site/Vite-bundling-process-13cfdaafcf1f429489014a709b900854?pvs=4)

## Minify / Uglify

번들러가 제공하는 강력한 기능중에 하나는 바로 최적화입니다. 다양하게 흩어져 있는 모듈과 코드들이 하나의 파일로 합쳐지면서, 하나의 파일에서 공백이 제거되고 불필요한 변수명이 정리되며 사용되지 않는 코드는 Tree Shaking이라고 하는 방식으로 제거됩니다.

저의 번들러는 안타깝게도 해당 최적화의 기능이 구현되어 있지 않습니다.

따라서 로직의 방향성을 적립하고, 사용 가능한 툴 들을 알아보고자 합니다.

제 번들러가 EMS 모듈을 CJS로 바꿔주는것 처럼

1. 번들링이 완료된 파일을 AST로 파싱
2. AST를 순회하며 노드를 분석 및 식별차 찾기( 변수명, 함수명)
3. 유니크한 변수명으로 변경 (ex. a, b, c)
4. 공백 제거

위와 같은 로직의 흐름을 생각해 보았습니다.

이를 위해 terser 혹은 terser를 활용하여 minify 해주는 SWC를 활용해보고자 합니다.

## HMR

오늘날 인기 있는 번들러 툴은 대부분 Hot Module Replacement 라고 불리우는 실시간 변화 감지 및 즉각 적용 기술을 도입하여 제공하고 있습니다.

이는 개발단계에서 개발자의 생산성을 크게 높여주며, 불필요한 모듈 교체를 최적화 시켜줍니다.

HMR은 기본적으로 번들러에서 제공하는 개발서버 환경에서 실행이 됩니다.

저의 번들러가 해당 기능을 제공하기 위해선 다음과 같은 흐름이 필요합니다.

- 개발 전용 dev-server 열림
- 각 번들러별 기술을 통해 모듈에서의 변화 감지
- 웹소켓을 활용해 변화가 일어나면 실시간으로 서버에게 메시지 발송
- 서버는 메시지를 받고 해당 모듈과 그 의존성을 교체(webpack)
- 혹은 ESM을 활용해 해당 모듈만 교체(vite)

여기서 생성되는 개발 서버의 경우 node 서버로 할지, rust 서버로 할지에 대한 고민이 수반되는듯 합니다.

## 빌드시 유저 편의성 개선

현재 저의 번들러는 사용자가 직접 엔트리 포인트를 작성해야 하며, 추후에 번들될 파일이 담길 아웃풋 경로 역시 직접 인자로 전달해야 합니다.

이는 유저의 편의성 측면에서 큰 불편을 초래합니다.

다른 번들러들의 방식

웹팩과 비트 두 번들러 모두 config 파일을 이용합니다. 비트의 경우 합리적인 default 옵션을 마련해놓음으로써, 사용자가 config 파일을 직접 설정하지 않아도 괜찮도록 편의성에 중점을 두고 있습니다.

이러한 부분을 참고하여 저 역시 config 파일을 마련하여 해당 옵션을 읽을 수 있으며, 아니라면 합리적인 기본 옵션을 제공하고자 합니다.

추가적으로 저의 번들러는 각 모듈을 찾아 의존성 그래프를 생성할때, 파일의 경로에 .js 등의 확장자가 붙어있지 않다면 해당 파일을 찾아내지 못하고 있습니다.

문자열을 그대로 경로화 하기 떄문인데, 이 부분에서 기본적인 옵션을 주는 방향으로 개선하고자 합니다.

# Memoir

프론트의 세계는 넓고 깊습니다. 제가 이번 번들러를 어설프게 구현해보면서 가장 크게 느낀 부분입니다.

단순히 화면을 보여주고 데이터를 뿌려주는곳에 그치지 않고 그 뒷단의 과정들을 더욱 면밀히 살필 수 있었던 계기인듯 합니다.

특히 최근 각광받고 있는 rust의 학습을 계기로 번들링과 그 과정에 대한 깊은 이해를 가져갈 수 있어서 다행이라고 생각합니다.

물론 여전히 너무나 부족한게 많은 수준이지만, 꾸준히 시간을 들여 하나 하나 매꿔보고 싶습니다.
