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

  fn fetch_local(&mut self, name: &str) -> Result<u32, ()> {
    self.locals.get(name).map(|i| *i).ok_or(
      response!(
        Wrong(format!("can't get undeclared local `{}`", name))
      )
    )
  }

  fn emit(&mut self, instr: Instruction) {
    self.code.push(instr)
  }

  fn emit_local_constant(&mut self, value: Value) -> Result<(), ()> {
    let index = self.consts.len();

    if index > u32::max_value() as usize {
      Err(
        response!(
          Wrong("constant overflow"),
          self.source.file
        )
      )
    } else {
      let index = index as u32;

      self.consts.push(value);
      self.emit(Instruction::LoadConst(index));

      Ok(())
    }
  }

  fn emit_branch_false(&mut self) -> JumpPatch {
    let result = JumpPatch(self.code.len());

    self.emit(Instruction::BranchFalse(0));
    result
  }

  fn emit_branch_true(&mut self) -> JumpPatch {
    let result = JumpPatch(self.code.len());

    self.emit(Instruction::BranchTrue(0));
    result
  }

  fn save_branch_target(&self) -> BranchTarget {
    BranchTarget(self.code.len())
  }

  fn patch_jump(&mut self, patch: JumpPatch) -> Result<(), ()> {
    let current    = self.code.len();
    let branch_pos = patch.0;
    let delta      = (current as isize) - (branch_pos as isize);

    if delta > i32::max_value() as isize || delta < i32::min_value() as isize {
      Err(
        response!(
          Wrong("branching too far"),
          self.source.file
        )
      )
    } else {
      let delta = delta as i32;

      match self.code[branch_pos] {
        Instruction::Jump(_)        => self.code[branch_pos] = Instruction::Jump(delta),
        Instruction::BranchTrue(_)  => self.code[branch_pos] = Instruction::BranchTrue(delta),
        Instruction::BranchFalse(_) => self.code[branch_pos] = Instruction::BranchFalse(delta),
      
        _ => unreachable!(),
      }

      Ok(())
    }
  }

  fn emit_jump_to(&mut self, target: BranchTarget) -> Result<(), ()> {
    let current = self.code.len();
    let BranchTarget(target) = target;
    let delta = target as isize - current as isize;


    if delta > i32::max_value() as isize || delta < i32::min_value() as isize {
      Err(
        response!(
          Wrong("branching too far"),
          self.source.file
        )
      )
    } else {
      let delta = delta as i32;
      self.emit(Instruction::Jump(delta));

      Ok(())
    }
  }
}



pub struct CompiledBlock {
  pub name:   String,
  pub code:   Rc<[Instruction]>,
  pub consts: Vec<Value>,
  pub locals: Rc<[String]>,
}