use crate::automata::pattern::Pattern;
use crate::automata::nfa::NFA;
use crate::state::rule::Rule;

use itertools::Itertools;

pub mod rule;

struct State {
    pub label:String,
    pub ix:i64,
    pub finish:Box<dyn FnMut()>,
    pub parent:Option<Box<State>>,
    pub rev_rules:Vec<Rule>,
}

impl Default for State {
    fn default() -> Self {
        State {
            label     : Default::default(),
            ix        : Default::default(),
            parent    : Default::default(),
            rev_rules : Default::default(),
            finish    : Box::new(||{}),
        }
    }
}

impl State {

    fn add_rule(&mut self, rule:Rule) {
        self.rev_rules.push(rule)
    }

    fn rule(&mut self, pattern:Pattern) -> rule::Builder<impl FnMut(Rule) + '_> {
        rule::Builder { pattern, finalizer: move |rule| self.add_rule(rule) }
    }

    fn rules(&self) -> Vec<&Rule> {
        let mut parent = &self.parent;
        let mut rules  = (&self.rev_rules).iter().rev().collect_vec();
        while let Some(state) = parent {
            rules.extend((&state.rev_rules).iter().rev());
            parent = &state.parent;
        }
        rules
    }

    fn rule_name(&self, rule_ix:usize) -> String {
        format!("group{}_rule{}", self.ix, rule_ix)
    }

    fn build_automata(&self) -> NFA {
        let mut nfa       = NFA::default();
        let     rules     = self.rules();
        let     start     = nfa.add_state();
        let mut endpoints = vec![0;rules.len()];
        for (ix,rule) in rules.iter().enumerate() {
            endpoints[ix] = self.build_rule_automata(&mut nfa,start,ix,rule);
        }
        let end = nfa.add_state();
        nfa.states[end].rule = Some(String::from(""));
        for endpoint in endpoints {
            nfa.set_link(endpoint, end)
        }
        nfa
    }

    pub fn build_rule_automata(&self, nfa:&mut NFA, last:usize, rule_ix:usize, rule:&Rule) -> usize {
        let end = Self::build_expr_automata(nfa,last,&rule.pattern);
        nfa.states[end].rule = Some(self.rule_name(rule_ix));
        end
    }

    pub fn build_expr_automata(nfa:&mut NFA, last:usize, expr:&Pattern) -> usize {
        let current = nfa.add_state();
        nfa.set_link(last, current);
        match expr {
            Pattern::Range(range) => {
                let state = nfa.add_state();
                nfa.set_link_range(current, state, range);
                state
            },
            Pattern::Many(body) => {
                let s1 = nfa.add_state();
                let s2 = Self::build_expr_automata(nfa,s1,body);
                let s3 = nfa.add_state();
                nfa.set_link(current, s1);
                nfa.set_link(current, s3);
                nfa.set_link(s2, s3);
                nfa.set_link(s3, s1);
                s3
            },
            Pattern::And(patterns) => {
                let build = |s,pat| Self::build_expr_automata(nfa,s,pat);
                patterns.iter().fold(current,build)
            },
            Pattern::Or(patterns) => {
                let build  = |pat| Self::build_expr_automata(nfa,current,pat);
                let states = patterns.iter().map(build).collect_vec();
                let end    = nfa.add_state();
                for state in states {
                    nfa.set_link(state, end);
                }
                end
            }
        }
    }
}

// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automata::pattern::Pattern;
    use crate::automata::nfa::NFA;
    use crate::state::rule::Rule;
    use crate::automata::state;
    use crate::automata::dict::Dict;
    use crate::automata::dfa::DFA;
    use crate::automata::state::Desc;
    use crate::automata::dfa::EndStatePriorityMap;

    use std::default::Default;



    const M:usize = state::MISSING;

    #[test]
    fn test_newline() {
        use crate::automata::state::State;

        let     pattern = Pattern::char('\n');
        let mut state   = crate::state::State::default();

        state.add_rule(Rule{pattern,tree:Default::default()});

        let nfa = state.build_automata();
        let dfa = nfa.to_dfa();

        let expected_nfa = NFA {
            states: vec![
                State::link_epsilon(&[1]),
                State::link_target(&[(10..10,2)]),
                State::link_epsilon(&[3]).named("group0_rule0"),
                State::default(),
            ],
            vocabulary: Dict::new(&[0,10,11,2147483647]),
        };
        let expected_dfa = DFA {
            vocabulary: Dict::new(&[0,10,11,2147483647]),
            links: vec![vec![M,1,M], vec![M,M,M]],
            priorities: EndStatePriorityMap::new(
                &[(1, Desc{priority:2,rule:"group0_rule0".into()})]
            )
        };

        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }

    #[test]
    fn test_letter() {
        use crate::automata::state::State;

        let     pattern = Pattern::range('a','z');
        let mut state   = crate::state::State::default();

        state.add_rule(Rule{pattern,tree:Default::default()});

        let nfa = state.build_automata();
        let dfa = nfa.to_dfa();

        let expected_nfa = NFA {
            states: vec![
                State::link_epsilon(&[1]),
                State::link_target(&[(97..122,2)]),
                State::link_epsilon(&[3]).named("group0_rule0"),
                State::default(),
            ],
            vocabulary: Dict::new(&[0,97,123,2147483647]),
        };
        let expected_dfa = DFA {
            vocabulary: Dict::new(&[0,97,123,2147483647]),
            links: vec![vec![M,1,M], vec![M,M,M]],
            priorities: EndStatePriorityMap::new(
                &[(1, Desc{priority:2,rule:"group0_rule0".into()})]
            )
        };

        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }

    #[test]
    fn test_spaces() {
        use crate::automata::state::State;

        let     pattern = Pattern::char(' ').many1();
        let mut state   = crate::state::State::default();

        state.add_rule(Rule{pattern,tree:Default::default()});

        let nfa = state.build_automata();
        let dfa = nfa.to_dfa();

        let expected_nfa = NFA {
            states: vec![
                State::link_epsilon(&[1,3]),
                State::link_target(&[(10..10,2)]),
                State::link_epsilon(&[11]).named("group0_rule0"),
                State::link_epsilon(&[4]),
                State::link_target(&[(32..32,5)]),
                State::link_epsilon(&[6]),
                State::link_epsilon(&[7,10]),
                State::link_epsilon(&[8]),
                State::link_target(&[(32..32,9)]),
                State::link_epsilon(&[10]),
                State::link_epsilon(&[7,11]).named("group0_rule1"),
                State::default(),
            ],
            vocabulary: Dict::new(&[0,10,11,32,33,2147483647]),
        };
        let expected_dfa = DFA {
            vocabulary: Dict::new(&[0,10,11,32,33,2147483647]),
            links: vec![
                vec![M,1,M,2,M],
                vec![M,M,M,M,M],
                vec![M,M,M,3,M],
                vec![M,M,M,3,M],
            ],
            priorities: EndStatePriorityMap::new(&[
                (1,Desc{priority:10,rule:"group0_rule0".into()}),
                (2,Desc{priority: 2,rule:"group0_rule1".into()}),
                (3,Desc{priority: 3,rule:"group0_rule1".into()}),
            ])
        };
        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }

    #[test]
    fn test_letter_and_spaces() {
        use crate::automata::state::State;

        let     letter = Pattern::range('a','z');
        let     spaces = Pattern::char(' ').many1();
        let mut state  = crate::state::State::default();

        state.add_rule(Rule{pattern:letter,tree:Default::default()});
        state.add_rule(Rule{pattern:spaces,tree:Default::default()});

        let nfa = state.build_automata();
        let dfa = nfa.to_dfa();

        let expected_nfa = NFA {
            states: vec![
                State::link_epsilon(&[1,3]),
                State::link_target(&[(97..122,2)]),
                State::link_epsilon(&[11]).named("group0_rule0"),
                State::link_epsilon(&[4]),
                State::link_target(&[(32..32,5)]),
                State::link_epsilon(&[6]),
                State::link_epsilon(&[7,10]),
                State::link_epsilon(&[8]),
                State::link_target(&[(32..32,9)]),
                State::link_epsilon(&[10]),
                State::link_epsilon(&[7,11]).named("group0_rule1"),
                State::default(),
            ],
            vocabulary: Dict::new(&[0,32,33,97,123,2147483647]),
        };
        let expected_dfa = DFA {
            vocabulary: Dict::new(&[0,32,33,97,123,2147483647]),
            links: vec![
                vec![M,1,M,2,M],
                vec![M,3,M,M,M],
                vec![M,M,M,M,M],
                vec![M,3,M,M,M],
            ],
            priorities: EndStatePriorityMap::new(&[
                (1,Desc{priority: 2,rule:"group0_rule0".into()}),
                (2,Desc{priority:10,rule:"group0_rule1".into()}),
                (3,Desc{priority: 2,rule:"group0_rule1".into()}),
            ])
        };

        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }
}
