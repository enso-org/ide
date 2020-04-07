use crate::automata::pattern::Pattern;

#[derive(Clone,Debug)]
pub struct Rule {
    pub pattern : Pattern,
    pub tree    : String,
}

#[derive(Clone,Debug)]
pub struct Builder<Finalizer> {
    pub pattern   : Pattern,
    pub finalizer : Finalizer,
}

impl<F:FnMut(Rule)> Builder<F> {
    fn run(&mut self, program:String){
        let rule = Rule {pattern:self.pattern.clone(), tree:program};
        (self.finalizer)(rule);
    }
}
