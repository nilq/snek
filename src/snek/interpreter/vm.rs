use super::*;

use std::rc::Rc;
use std::ptr;

pub struct Call {
  locals: Rc<[Value]>,
  ip:     usize,
  func:   *const CompiledBlock,
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



pub struct VirtualMachine {
  stack: Vec<Value>,
  calls: Vec<Call>,

  pub next: *mut HeapValue,
}

impl VirtualMachine {
  pub fn new() -> Self {
    VirtualMachine {
      stack: Vec::new(),
      calls: Vec::new(),
      next:  ptr::null_mut(),
    }
  }



  pub fn execute(&mut self, initial: *const CompiledBlock) {
    use self::Instruction::*;
    use self::Value::*;

    let mut ip: usize           = 0;
    let mut fun: &CompiledBlock = unsafe { &*initial };
    let mut locals              = vec![Nil; fun.locals.len()].into_boxed_slice();

    macro_rules! match_binop {
      ($($pat:pat => $block:block)+) => {{
        let _a = self.stack.pop().unwrap();
        let _b = self.stack.pop().unwrap();
        
        let _result = match (_b, _a) {
            $($pat => $block)+,
            _ => panic!("Invalid operands"),
        };
        
        self.stack.push(_result);
      }}
    }

    loop {
      let op = fun.code[ip];

      match op {
        LoadConst(index)  => self.stack.push(fun.consts[index as usize]),
        LoadLocal(index)  => self.stack.push(locals[index as usize]),
        StoreLocal(index) => {
          locals[index as usize] = self.stack.pop().unwrap();
        },

        BranchTrue(delta) => {
          if self.stack.pop().unwrap().is_truthy() {
            ip = ip.wrapping_add(delta as isize as usize)
          } else {
            ip = ip.wrapping_add(1)
          }

          continue
        },

        BranchFalse(delta) => {
          if !self.stack.pop().unwrap().is_truthy() {
            ip = ip.wrapping_add(delta as isize as usize)
          } else {
            ip = ip.wrapping_add(1)
          }

          continue
        },

        Jump(delta) => {
          ip = ip.wrapping_add(delta as isize as usize);

          continue
        },

        Pop => { self.stack.pop().unwrap(); },

        _ => (),
      },

      ip = ip.wrapping_add(1)
    }
  }
}