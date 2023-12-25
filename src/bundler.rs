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
