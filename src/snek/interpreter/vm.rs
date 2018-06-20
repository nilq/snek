use super::*;

use std::ptr;
use std::mem;

pub struct CallInfo {
  locals: Box<[Value]>,
  ip:     usize,
  func:   *const CompiledBlock,
}



#[derive(Debug, Clone, Copy)]
pub enum Instruction {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Concat,

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
  pub stack: Vec<Value>,

  calls: Vec<CallInfo>,

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

        Add => match_binop! {
          (Int(a), Int(b))       => { Int(a + b) }
          (Double(a), Double(b)) => { Double(a + b) }
        },

        Sub => match_binop! {
          (Int(a), Int(b))       => { Int(a - b) }
          (Double(a), Double(b)) => { Double(a - b) }
        },

        Mul => match_binop! {
          (Int(a), Int(b))       => { Int(a * b) }
          (Double(a), Double(b)) => { Double(a * b) }
        },

        Div => match_binop! {
          (Int(a), Int(b))       => { Int(a / b) }
          (Double(a), Double(b)) => { Double(a / b) }
        },

        Mod => match_binop! {
          (Int(a), Int(b))       => { Int(a % b) }
          (Double(a), Double(b)) => { Double(a % b) }
        },

        Return => {
          if let Some(call_info) = self.calls.pop() {
            fun    = unsafe { &*call_info.func };
            locals = call_info.locals;
            ip     = call_info.ip
          } else {
            break
          }
        },

        Call(args) => {
          let args = args as usize;

          let func_index = self.stack.len() - args - 1;
          let func_val   = self.stack[func_index];

          let func_backup = fun;

          if let Value::HeapValue(pointer) = func_val {
            let object = unsafe { &*pointer };

            if let HeapValueType::Function(ref function) = object.kind {
              fun = function
            }
          }

          let mut new_locals = vec![Value::Nil; fun.locals.len()].into_boxed_slice();

          for i in 0 .. args {
            new_locals[i] = self.stack[func_index + i + 1]
          }

          for _ in 0 .. args + 1 {
            self.stack.pop();
          }

          let old_locals = mem::replace(&mut locals, new_locals);

          self.calls.push(
            CallInfo {
              ip,
              locals: old_locals,
              func: func_backup,
            }
          );

          ip = 0;

          continue
        }

        _ => (),
      }

      ip = ip.wrapping_add(1)
    }
  }



  pub fn allocate(&mut self, kind: HeapValueType) -> Value {
    let object = Box::into_raw(
      Box::new(
        HeapValue {
          next: self.next,
          kind,
        }
      )
    );

    self.next = object;

    Value::HeapValue(object)
  }
}