use super::*;
use super::error::Response::Wrong;
use super::source::Source;


use std::mem;

use std::collections::HashMap;

#[derive(Clone, Copy)]
struct JumpPatch(usize);

#[derive(Clone, Copy)]
struct BranchTarget(usize);



pub struct CompiledBlock {
  pub name:   String,
  pub code:   Box<[Instruction]>,
  pub consts: Vec<Value>,
  pub locals: Box<[String]>,
}



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
    Ok(self.locals.get(name).map(|i| *i).unwrap())
  }

  fn emit(&mut self, instr: Instruction) {
    self.code.push(instr)
  }

  fn emit_load_constant(&mut self, value: Value) -> Result<(), ()> {
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



  fn compile_statement(&mut self, statement: &'c Statement<'c>) -> Result<(), ()> {
    use self::StatementNode::*;
    use self::ExpressionNode::*;
    
    match statement.node {
      Variable(_, ref left, ref right) => {
        if let Identifier(ref name) = left.node {
          if let Some(ref right) = *right {
            self.compile_expression(&*right)?;

            let index = self.declare_local(name)?;

            self.emit(Instruction::StoreLocal(index))
          } else {
            self.declare_local(name)?;
          }
        }
      },

      Expression(ref expression) => self.compile_expression(expression)?,

      _ => (),
    }

    Ok(())
  }

  fn compile_expression(&mut self, expression: &'c Expression<'c>) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Int(a)    => self.emit_load_constant(Value::Int(a as i128))?,
      Double(a) => self.emit_load_constant(Value::Double(a))?,

      Binary(ref left, ref op, ref right) => {
        self.compile_expression(&**left)?;
        self.compile_expression(&**right)?;

        use self::Operator::*;

        match *op {
          Add    => self.emit(Instruction::Add),
          Sub    => self.emit(Instruction::Sub),
          Mul    => self.emit(Instruction::Mul),
          Div    => self.emit(Instruction::Div),
          Mod    => self.emit(Instruction::Mod),
          _   => (),
        }
      },

      Identifier(ref name) => {
        println!("herer");

        let index = self.fetch_local(name)?;
        self.emit(Instruction::LoadLocal(index))
      },

      _ => (),
    }

    Ok(())
  }



  pub fn compile_main(&mut self, block: &'c Vec<Statement<'c>>, name: &str) -> Result<CompiledBlock, ()> {
    for element in block {
      self.compile_statement(element)?
    }

    self.emit_load_constant(Value::Nil)?;
    self.code.push(Instruction::Return);

    let mut local_names = vec![String::new(); self.locals.len()];

    for (name, i) in self.locals.drain() {
      local_names[i as usize] = name
    }

    Ok(
      CompiledBlock {
        name:   name.to_string(),
        code:   mem::replace(&mut self.code, Vec::new()).into_boxed_slice(),
        consts: mem::replace(&mut self.consts, Vec::new()),
        locals: local_names.into_boxed_slice()
      }
    )
  }
}
