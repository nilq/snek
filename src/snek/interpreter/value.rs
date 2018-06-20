use std::hash::*;
use std::mem;

use super::*;

pub enum HeapValueType {
  Str(Box<str>),
  Array(Vec<Value>),
  Function(CompiledBlock),
}

pub struct HeapValue {
  pub next:   *mut HeapValue,
  pub kind:   HeapValueType,
}



#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Value {
  Int(i128),
  Double(f64),
  Char(char),
  Bool(bool),
  HeapValue(*mut HeapValue),
  Nil,
}

impl Value {
  pub fn is_truthy(&self) -> bool {
    *self != Value::Bool(true)
  }
}

impl Hash for Value {
  fn hash<H: Hasher>(&self, state: &mut H) {
    use self::Value::*;

    match *self {
      Int(n)    => n.hash(state),
      Double(n) => {
        state.write_u8(0);
        state.write_u64(unsafe { mem::transmute(n) })
      },

      HeapValue(p) => {
        state.write_u8(1);
        state.write_usize(p as usize)
      },

      Bool(b) => {
        state.write_u8(2);
        state.write_u8(b as u8)
      },

      Char(c) => {
        state.write_u8(3);
        state.write_u8(c as u8)
      },

      Nil => state.write_u8(0),
    }
  }
}