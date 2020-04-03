use crate::automata::pattern::Pattern;
use crate::automata::nfa::NFA;
use crate::state::rule::Rule;

pub mod rule;

struct State {
    pub label:String,
    pub ix:i64,
    pub finish:Box<dyn FnMut()>,
    parent:Option<Box<State>>,
    rev_rules:Vec<Rule>,
}

impl State {

    fn set_parent(&mut self, parent:Box<State>) {
        self.parent = Some(parent)
    }

    fn add_rule(&mut self, rule:Rule) {
        self.rev_rules.push(rule)
    }

    fn rule(&mut self, pattern:Pattern) -> rule::Builder<impl FnMut()> {
        rule::Builder { pattern, finalizer: |rule| self.add_rule(rule) }
    }

    fn rules(&self) -> Vec<Rule> {
        let mut parent = self.parent;
        let mut rules  = self.rev_rules.iter().rev().collect();
        while let Some(state) = parent {
            rules.extend(state.rev_rules.iter().rev());
            parent = state.parent;
        }
        rules
    }

    fn rule_name(&self, rule_ix:usize) -> String {
        format!("group{}_rule{}", self.ix, rule_ix)
    }

    fn build_automata(&self) -> NFA {
        let mut nfa   = NFA::default();
        let start     = nfa.add_state();
        let endpoints = vec![0;self.rules.len()];
        for (ix,rule) in self.rules.enumerate() {
            endpoints[ix] = self.build_rule_automata(&mut nfa,start,ix,rule);
        }
        let end = nfa.add_state();
        nfa.state(end).rule = Some("");
        for endpoint in endpoints {
            nfa.link(endpoint, end)
        }
        nfa
    }

    pub fn build_rule_automata(&self, nfa:&mut NFA, last:usize, rule_ix:usize, rule:Rule) -> usize {
        let end = Self::build_expr_automata(nfa,last,rule.pattern);
        nfa.state(end).rule = Some(self.rule_name(rule_ix));
        end
    }

    pub fn build_expr_automata(nfa:&mut NFA, last:usize, expr:Pattern) -> usize {
        let current = nfa.add_state();
        nfa.link(last, current);
        match expr {
            Pattern::Always() => current,
            Pattern::Range(range) => {
                let state = nfa.addState();
                nfa.link_range(current,state,range);
                state
            },
            Pattern::Many(body) => {
                let s1 = nfa.addState();
                let s2 = Self::build_expr_automata(nfa,s1,body);
                let s3 = nfa.addState();
                nfa.link(current,s1);
                nfa.link(current,s3);
                nfa.link(s2,s3);
                nfa.link(s3,s1);
                s3
            },
            Pattern::And(patterns) => {
                let build = |s,pat| Self::build_expr_automata(nfa,s,pat);
                patterns.iter().fold(current,build)
            },
            Pattern::Or(patterns) => {
                let build  = |pat| Self::build_expr_automata(nfa,current,pat);
                let states = patterns.iter().map(build).collect();
                let end    = nfa.add_state();
                for state in states {
                    nfa.link(state,end);
                }
                end
            }
        }
    }
}
