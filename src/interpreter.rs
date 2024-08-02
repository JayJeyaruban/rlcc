use std::io::Write;

use crate::parser::{ExprContext, LolCodeProgram};

pub fn execute<W: Write>(prog: LolCodeProgram, out: &mut W) {
    for instr in prog.instrs {
        match instr {
            ExprContext::Visible { args } => {
                for arg in args {
                    write!(out, "{arg}").expect("write to output");
                }

                writeln!(out, "").expect("newline to output");
            }
            ExprContext::Join(_) => todo!(),
            ExprContext::String(_) => todo!(),
            ExprContext::IncludeInProgress(_) => todo!(),
            ExprContext::Include(_) => {}
        }
    }
}
