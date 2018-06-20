#![feature(i128)]
#![feature(i128_type)]
#![feature(use_nested_groups)]

extern crate colored;

mod snek;
use snek::lexer::*;
use snek::parser::*;
use snek::visitor::*;
use snek::interpreter::*;

fn main() {
  let content = r#"
foo := 1000
bar := (foo / 3)
  "#;

  let source = Source::from("<static.wu>", content.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(content.chars().collect(), &source);

  let mut tokens = Vec::new();

  for token_result in lexer {
    if let Ok(token) = token_result {
      tokens.push(token)
    } else {
      return
    }
  }

  let tokens_ref = tokens.iter().map(|x| &*x).collect::<Vec<&Token>>();

  let mut parser = Parser::new(tokens_ref, &source);
  
  match parser.parse() {
    Ok(ast) => {
      println!("{:#?}", ast);
  
      let mut visitor = Visitor::new(&source, &ast);

      match visitor.visit() {
        Ok(_) => {
          let mut vm = VirtualMachine::new();
          let block  = Compiler::new(&mut vm, &source).compile_main(&ast, "entry").unwrap();

          vm.execute(&block);

          println!("{:#?}", vm.stack)
        },
        _     => ()
      }
    },

    _ => (),
  }
}
