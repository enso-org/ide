use crate::automata::pattern::Pattern;
use crate::automata::nfa::NFA;
use crate::group::rule::Rule;

use itertools::Itertools;

pub mod rule;

/// Group of a set of rules.
struct Group {
    /// ID of group.
    pub id        : usize,
    /// Name of group.
    pub name      : String,
    /// Parent of group.
    pub parent    : Option<Box<Group>>,
    /// Set of rules. See Rule for more information.
    pub rules     : Vec<Rule>,
    /// Function that is called on exiting group.
    pub finish    : Box<dyn FnMut()>,
}

impl Default for Group {
    fn default() -> Self {
        Group {
            name      : Default::default(),
            id        : Default::default(),
            parent    : Default::default(),
            rules     : Default::default(),
            finish    : Box::new(||{}),
        }
    }
}

impl Group {

    /// Adds new rule to group.
    pub fn add_rule(&mut self, rule:Rule) {
        self.rules.push(rule)
    }

    /// Returns rule builder for given pattern.
    pub fn rule(&mut self, pattern:Pattern) -> rule::Builder<impl FnMut(Rule) + '_> {
        rule::Builder { pattern, callback: move |rule| self.add_rule(rule) }
    }

    /// All rules including parent rules.
    pub fn rules(&self) -> Vec<&Rule> {
        let mut parent = &self.parent;
        let mut rules  = (&self.rules).iter().collect_vec();
        while let Some(state) = parent {
            rules.extend((&state.rules).iter());
            parent = &state.parent;
        }
        rules
    }

    /// Canonical name of given rule.
    pub fn rule_name(&self, rule_ix:usize) -> String {
        format!("group{}_rule{}", self.id, rule_ix)
    }

    /// Transforms Group to NFA.
    /// Algorithm is based on: https://www.youtube.com/watch?v=RYNN-tb9WxI
    pub fn to_nfa(&self) -> NFA {
        let mut nfa   = NFA::default();

        let start  = nfa.new_state();
        let build  = |rule:&Rule| rule.pattern.to_nfa(&mut nfa, start);
        let states = self.rules().into_iter().map(build).collect_vec();
        let end    = nfa.new_state();
        for (ix, state) in states.into_iter().enumerate() {
            nfa.states[state].name = Some(self.rule_name(ix));
            nfa.connect(state, end);
        }
        nfa
    }
}

// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use crate::automata::alphabet::Alphabet;
    use crate::automata::dfa::DFA;
    use crate::automata::dfa::EndState;
    use crate::automata::nfa::NFA;
    use crate::automata::pattern::Pattern;
    use crate::automata::state;
    use crate::automata::state::State;
    use crate::group::Group;
    use crate::group::rule::Rule;

    use std::default::Default;



    const M:usize = state::MISSING;

    #[test]
    fn test_newline() {
        let     pattern = Pattern::char('\n');
        let mut state   = Group::default();

        state.add_rule(Rule{pattern, callback:"".into()});

        let nfa = state.to_nfa();
        let dfa = nfa.to_dfa();

        let expected_nfa = NFA {
            states: vec![
                State::epsilon_links(&[1]),
                State::links(&[(10..10, 2)]),
                State::epsilon_links(&[3]).named("group0_rule0"),
                State::default(),
            ],
            alphabet: Alphabet::new(&[10,11]),
        };
        let expected_dfa = DFA {
            alphabet: Alphabet::new(&[10,11]),
            links: vec![vec![M,1,M], vec![M,M,M]],
            end_states: vec![
                None,
                Some(EndState{priority:2, name:"group0_rule0".into()}),
            ]
        };

        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }

    #[test]
    fn test_letter() {
        let     pattern = Pattern::range('a'..'z');
        let mut state   = Group::default();

        state.add_rule(Rule{pattern, callback:"".into()});

        let nfa = state.to_nfa();
        let dfa = nfa.to_dfa();

        let expected_nfa = NFA {
            states: vec![
                State::epsilon_links(&[1]),
                State::links(&[(97..122,2)]),
                State::epsilon_links(&[3]).named("group0_rule0"),
                State::default(),
            ],
            alphabet: Alphabet::new(&[97,123]),
        };
        let expected_dfa = DFA {
            alphabet: Alphabet::new(&[97,123]),
            links: vec![vec![M,1,M], vec![M,M,M]],
            end_states: vec![
                None,
                Some(EndState{priority:2,name:"group0_rule0".into()}),
            ]
        };

        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }

    #[test]
    fn test_spaces() {
        println!("[[{}]]", 10 as u8 as char);
        let     pattern = Pattern::char(' ').many1();
        let mut state   = Group::default();

        state.add_rule(Rule{pattern, callback:"".into()});

        let nfa = state.to_nfa();
        let dfa = nfa.to_dfa();

        let expected_nfa = NFA {
            states: vec![
                State::epsilon_links(&[1]),
                State::epsilon_links(&[2]),
                State::links(&[(32..32,3)]),
                State::epsilon_links(&[4]),
                State::epsilon_links(&[5,8]),
                State::epsilon_links(&[6]),
                State::links(&[(32..32,7)]),
                State::epsilon_links(&[8]),
                State::epsilon_links(&[5,9]).named("group0_rule0"),
                State::default(),
            ],
            alphabet: Alphabet::new(&[0, 32, 33]),
        };
        let expected_dfa = DFA {
            alphabet: Alphabet::new(&[0, 32, 33]),
            links: vec![
                vec![M,1,M],
                vec![M,2,M],
                vec![M,2,M],
            ],
            end_states: vec![
                None,
                Some(EndState{priority:3,name:"group0_rule0".into()}),
                Some(EndState{priority:3,name:"group0_rule0".into()}),
            ]
        };
        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }

    #[test]
    fn test_letter_and_spaces() {
        let     letter = Pattern::range('a'..'z');
        let     spaces = Pattern::char(' ').many1();
        let mut state  = Group::default();

        state.add_rule(Rule{pattern:letter, callback:"".into()});
        state.add_rule(Rule{pattern:spaces, callback:"".into()});

        let nfa = state.to_nfa();
        let dfa = nfa.to_dfa();

        let expected_nfa = NFA {
            states: vec![
                State::epsilon_links(&[1,3]),
                State::links(&[(97..122,2)]),
                State::epsilon_links(&[11]).named("group0_rule0"),
                State::epsilon_links(&[4]),
                State::links(&[(32..32,5)]),
                State::epsilon_links(&[6]),
                State::epsilon_links(&[7,10]),
                State::epsilon_links(&[8]),
                State::links(&[(32..32,9)]),
                State::epsilon_links(&[10]),
                State::epsilon_links(&[7,11]).named("group0_rule1"),
                State::default(),
            ],
            alphabet: Alphabet::new(&[32,33,97,123]),
        };
        let expected_dfa = DFA {
            alphabet: Alphabet::new(&[32,33,97,123]),
            links: vec![
                vec![M,1,M,2,M],
                vec![M,3,M,M,M],
                vec![M,M,M,M,M],
                vec![M,3,M,M,M],
            ],
            end_states: vec![
                None,
                Some(EndState{priority:4,name:"group0_rule1".into()}),
                Some(EndState{priority:4,name:"group0_rule0".into()}),
                Some(EndState{priority:4,name:"group0_rule1".into()}),
            ]
        };

        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }
}
