use std::io::Write;

use anyhow::Context;

use crate::{
    framework::StdOut,
    parser::{ExprContext, Instruction, LolCodeProgram},
};

pub trait Interpret {
    fn execute(&mut self, prog: LolCodeProgram) -> anyhow::Result<()>;
}

impl<T> Interpret for T
where
    T: StdOut,
{
    fn execute(&mut self, prog: LolCodeProgram) -> anyhow::Result<()> {
        for instr in prog.instrs {
            match instr {
                Instruction::Visible { args } => {
                    for arg in args {
                        write!(self.out(), "{arg}").context("write to output")?;
                    }

                    writeln!(self.out(), "").context("newline to output")?;
                }
                Instruction::LoadModule { .. } => {}
            }
        }

        Ok(())
    }
}
