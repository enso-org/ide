use crate::prelude::*;

struct GeneralizedApplication {
    target : Ast,
    arguments : Vec<Ast>
}

//impl GeneralizedApplication {
//    pub fn try_new(ast:&Ast) -> Option<GeneralizedApplication> {
//        if let Some(chain) = crate::opr::Chain::try_new(ast) {
//            let target = chain.target?;
//            let arguments = chain.args.into_iter().map(|arg| arg.operand)
//
//        }
//    }
//}

