#[macro_use]
use super::*;

use super::super::error::Response::Wrong;
use std::mem;



pub struct Compiler<'c> {
  pub bytecode: Vec<u8>,
  pub visitor:  &'c mut Visitor<'c>,
}

impl<'c> Compiler<'c> {
  pub fn new(visitor: &'c mut Visitor<'c>) -> Self {
    Compiler {
      bytecode: Vec::new(),
      visitor,
    }
  }



  pub fn compile(&mut self, ast: &'c Vec<Statement>) -> Result<(), ()> {
    println!();

    for statement in ast {
      self.compile_statement(statement)?
    }

    self.emit(Instruction::Halt);

    println!();

    Ok(())
  }

  fn compile_statement(&mut self, statement: &'c Statement) -> Result<(), ()> {
    use self::StatementNode::*;

    match statement.node {
      Expression(ref expression) => self.compile_expression(expression)?,
      Constant(ref t, ref left, ref right) => {
        if let ExpressionNode::Identifier(_) = left.node {
          self.assign(t, left, right)?
        }
      }

      Variable(ref t, ref left, ref right) => {
        if let ExpressionNode::Identifier(_) = left.node {
          if let &Some(ref right) = right {
            self.assign(t, left, right)?
          }
        } else if let ExpressionNode::Set(_) = left.node {
          return Err(
            response!(
              Wrong("set declaration compilation is unimplemented"),
              statement.pos
            )
          )
        }
      }

      _ => (),
    }

    Ok(())
  }

  fn compile_expression(&mut self, expression: &'c Expression) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Int(ref n) => {
        self.emit(Instruction::Push);
        self.emit_byte(mem::size_of::<u128>() as u8);
        self.emit_bytes(
          unsafe {
            &mem::transmute::<u128, [u8; mem::size_of::<u128>()]>(*n)
          }
        );
      },

      Float(ref n) => {
        self.emit(Instruction::Push);
        self.emit_byte(mem::size_of::<f32>() as u8);
        self.emit_bytes(
          unsafe {
            &mem::transmute::<f32, [u8; mem::size_of::<f32>()]>(*n)
          }
        );
      },

      Char(ref n) => {
        self.emit(Instruction::Push);
        self.emit_byte(mem::size_of::<char>() as u8);
        self.emit_byte(*n as u8)
      },

      String(ref n) => {
        self.emit(Instruction::Push);
        self.emit_byte(n.len() as u8);
        self.emit_bytes(n.as_bytes());
      },

      Bool(ref n) => {
        self.emit(Instruction::Push);
        self.emit_byte(mem::size_of::<u8>() as u8);
        self.emit_byte(*n as u8)
      },

      Identifier(ref name) => {
        let (index, env_index) = self.visitor.symtab.get_name(name).unwrap();
        let offset             = self.visitor.typetab.get_offset(index, env_index).unwrap();
        let size               = self.visitor.typetab.get_type(index, env_index).unwrap().node.byte_size();

        self.emit(Instruction::PushDeref);
        self.emit_byte(size as u8);
        self.emit_bytes(&to_bytes!(offset => u32));
      },

      Cast(ref expression, ref t) => {
        use self::TypeNode::*;

        let size = self.visitor.type_expression(&expression)?.node.byte_size();

        self.compile_expression(expression)?;

        match (self.visitor.type_expression(expression)?.node, &t.node) {
          (ref a, ref b) if *b == &F32 || *b == &F64 => match a {
            &I08 | &I32 | &I64 | &I128 | &U08 | &U32 | &U64 | &U128 => self.emit(Instruction::ConvIF),
            &F32 | &F64                                             => self.emit(Instruction::ConvFF),
            c => return Err(
              response!(
                Wrong(format!("can't cast from `{}`", c)),
                expression.pos
              )
            )
          },

          (ref a, ref b) if *b == &I08 || *b == &I32 || *b == &I64 || *b == &I128 || *b == &U08 || *b == &U32 || *b == &U64 || *b == &U128 => match a {
            &F32 | &F64                                             => self.emit(Instruction::ConvFI),
            &I08 | &I32 | &I64 | &I128 | &U08 | &U32 | &U64 | &U128 => self.emit(Instruction::ConvII),

            c => return Err(
              response!(
                Wrong(format!("can't cast from `{}`", c)),
                expression.pos
              )
            )
          },

          (_, ref node) => return Err(response!(Wrong(format!("can't cast to `{}`", node))))
        }

        self.emit_byte(size as u8);
        self.emit_byte(t.node.byte_size() as u8)
      },

      _ => (),
    }

    Ok(())
  }



  fn assign(&mut self, t: &Type, left: &'c Expression<'c>, right: &'c Expression<'c>) -> Result<(), ()> {
    use self::TypeNode::*;

    if let ExpressionNode::Identifier(ref name) = left.node {
      self.compile_expression(right)?;

      let right_type = self.visitor.type_expression(right)?;

      if right_type.node != t.node {
        match &t.node {
          &I08 | &I32 | &I64 | &I128 | &U08 | &U32 | &U64 | &U128 => match &right_type.node {
            &I08 | &I32 | &I64 | &I128 => {
              self.emit(Instruction::ConvII);
              self.emit_byte(right_type.node.byte_size() as u8);
              self.emit_byte(t.node.byte_size() as u8)
            },

            _ => (),
          },

          &F32 | &F64 => match &right_type.node {
            &I08 | &I32 | &I64 => {
              self.emit(Instruction::ConvIF);
              self.emit_byte(right_type.node.byte_size() as u8);
              self.emit_byte(t.node.byte_size() as u8)
            },

            _ => (),
          }

          _ => (),
        }
      }

      self.emit(Instruction::Pop);

      if t.node != Nil {
        self.emit_byte(t.node.byte_size() as u8);      
      } else {
        self.emit_byte(right_type.node.byte_size() as u8);
      }

      let (index, env_index) = self.visitor.symtab.get_name(name).unwrap();
      let offset             = self.visitor.typetab.get_offset(index, env_index).unwrap();

      let address = &to_bytes!(offset => u32);

      self.emit_bytes(address);
    }

    Ok(())
  }



  fn emit(&mut self, code: Instruction) {
    println!("{}", code);
    self.bytecode.push(code as u8)
  }

  fn emit_byte(&mut self, byte: u8) {
    println!("\t{}", byte);
    self.bytecode.push(byte)
  }

  fn emit_bytes(&mut self, bytes: &[u8]) {
    println!("\t{:?}", bytes);
    self.bytecode.extend(bytes)
  }
}