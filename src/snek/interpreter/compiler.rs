use super::*;
use super::error::Response::Wrong;
use super::source::Source;


use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy)]
struct JumpPatch(usize);

#[derive(Clone, Copy)]
struct BranchTarget(usize);



pub struct Compiler<'c> {
  locals: HashMap<String, u32>,
  code:   Vec<Instruction>,
  consts: Vec<Value>,
  vm:     &'c mut VirtualMachine,

  source: &'c Source,
}

impl<'c> Compiler<'c> {
  pub fn new(vm: &'c mut VirtualMachine, source: &'c Source) -> Self {
    Compiler {
      locals: HashMap::new(),
      code:   Vec::new(),
      consts: Vec::new(),
      vm,

      source,
    }
  }



  fn declare_local(&mut self, name: &str) -> Result<u32, ()> {
    use std::collections::hash_map::Entry;

    let index = self.locals.len();

    if index > self.locals.len() {
      Err(
        response!(
          Wrong(format!("local overflow at `{}`", name)),
          self.source.file
        )
      )
    } else {
      let index = index as u32;

      let entry = self.locals.entry(name.to_string());

      match entry {
        Entry::Occupied(_) => Err(
          response!(
            Wrong(format!("redeclared local `{}`", name)),
            self.source.file
          )
        ),

        Entry::Vacant(vacant) => {
          vacant.insert(index);

          Ok(index)
        }
      }
    }
  }
}



pub struct CompiledBlock {
  pub name:   String,
  pub code:   Rc<[Instruction]>,
  pub consts: Vec<Value>,
  pub locals: Rc<[String]>,
}