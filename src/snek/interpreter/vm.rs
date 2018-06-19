use super::*;

use std::rc::Rc;

pub struct Call {
  locals: Rc<[Value]>,
  ip:     usize,
  func:   *const CompiledBlock,
}

pub struct VirtualMachine {
  stack: Vec<Value>,
  calls: Vec<Call>,

  pub next: *mut HeapValue,
}



#[derive(Debug, Clone, Copy)]
pub enum Instruction {
  Add,
  Sub,
  Rem,
  Div,

  Neg,

  Lt,
  LtEq,
  Gt,
  GtEq,
  Eq,
  NEq,

  LoadConst(u32),
  LoadLocal(u32),
  StoreLocal(u32),

  BranchTrue(i32),
  BranchFalse(i32),
  Jump(i32),

  Pop,
  Return,
  Put,
  Call(u8),
}