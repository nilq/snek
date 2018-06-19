use super::*;

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
}

impl<'c> Compiler<'c> {
  pub fn new(vm: &'c mut VirtualMachine) -> Self {
    Compiler {
      locals: HashMap::new(),
      code:   Vec::new(),
      consts: Vec::new(),
      vm,
    }
  }
}



pub struct CompiledBlock {
  pub name:   String,
  pub code:   Rc<[Instruction]>,
  pub consts: Vec<Value>,
  pub locals: Rc<[String]>,
}