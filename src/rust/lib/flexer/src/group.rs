use crate::automata::pattern::Pattern;
use crate::automata::nfa::NFA;
use crate::group::rule::Rule;

use itertools::Itertools;

pub mod rule;



// ===========
// == Group ==
// ===========

/// Struct that group rules together.
/// It also inherits rules from parent group (if it has one).
/// Groups are the basic building block of flexer:
/// Flexer internally keeps a stack of groups, only one of them active at a time.
/// Each group contains set of regex patterns and callbacks (together called `Rule`).
/// Whenever a rule.pattern from active group is matched with part of input
/// the associated rule.callback is executed
/// which in turn may exit the current groupor enter a new one.
/// This allows us to nicely model a situation, where certain part of program
/// (like a string literal) should have very different parsing rules than other
/// (for example body of function).
/// Note that the input is first matched with first added rule, then with the second etc.
/// Therefore, if two rules overlap,
/// only the callback of the first added rule will be executed.
#[derive(Clone,Debug,Default)]
pub struct Group {
    /// Unique ID.
    pub id: usize,
    /// Custom name which is used for debugging.
    pub name: String,
    /// Parent which we inherit rules from.
    pub parent: Option<Box<Group>>,
    /// Set of regex patterns with associated callbacks.
    pub rules: Vec<Rule>,
}

impl Group {
    /// Adds new rule (regex pattern with associated callback) to group.
    pub fn add_rule(&mut self, rule:Rule) {
        self.rules.push(rule)
    }

    /// Returns rule builder for given pattern.
    /// TODO[jv] better describe it's purpose once we agree on correct API.
    pub fn rule(&mut self, pattern:Pattern) -> rule::Builder<impl FnMut(Rule) + '_> {
        rule::Builder{pattern,callback:move |rule| self.add_rule(rule)}
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
    fn callback_name(&self, rule_ix:usize) -> String {
        format!("group{}_rule{}",self.id,rule_ix)
    }
}

impl From<&Group> for NFA {
    /// Transforms Group to NFA.
    /// Algorithm is based on: https://www.youtube.com/watch?v=RYNN-tb9WxI
    fn from(group:&Group) -> Self {
        let mut nfa   = NFA::default();
        let start     = nfa.new_state();
        let build     = |rule:&Rule| rule.pattern.to_nfa(&mut nfa, start);
        let states    = group.rules().into_iter().map(build).collect_vec();
        let end       = nfa.new_state();
        for (ix, state) in states.into_iter().enumerate() {
            nfa.states[state.id].name = Some(group.callback_name(ix));
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
    use crate::automata::dfa::Callback;
    use crate::automata::nfa::NFA;
    use crate::automata::pattern::Pattern;
    use crate::automata::state;
    use crate::automata::state::State;
    use crate::group::Group;
    use crate::group::rule::Rule;

    use std::default::Default;



    const I:usize = state::INVALID.id;

    #[test]
    fn test_newline() {
        let     pattern = Pattern::char('\n');
        let mut state   = Group::default();

        state.add_rule(Rule{pattern, callback:"".into()});

        let nfa = NFA::from(&state);
        let dfa = DFA::from(&nfa);

        let expected_nfa = NFA {
            states: vec![
                State::from(vec![1]),
                State::from(vec![(10..=10, 2)]),
                State::from(vec![3]).named("group0_rule0"),
                State::default(),
            ],
            alphabet: Alphabet::from(vec![10,11]),
        };
        let expected_dfa = DFA {
            alphabet: Alphabet::from(vec![10,11]),
            links: DFA::links(vec![vec![I,1,I], vec![I,I,I]]),
            callbacks: vec![
                None,
                Some(Callback {priority:2, name:"group0_rule0".into()}),
            ]
        };

        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }

    #[test]
    fn test_letter() {
        let     pattern = Pattern::range('a'..='z');
        let mut state   = Group::default();

        state.add_rule(Rule{pattern, callback:"".into()});

        let nfa = NFA::from(&state);
        let dfa = DFA::from(&nfa);

        let expected_nfa = NFA {
            states: vec![
                State::from(vec![1]),
                State::from(vec![(97..=122,2)]),
                State::from(vec![3]).named("group0_rule0"),
                State::default(),
            ],
            alphabet: Alphabet::from(vec![97,123]),
        };
        let expected_dfa = DFA {
            alphabet: Alphabet::from(vec![97,123]),
            links: DFA::links(vec![vec![I,1,I], vec![I,I,I]]),
            callbacks: vec![
                None,
                Some(Callback {priority:2,name:"group0_rule0".into()}),
            ]
        };

        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }

    #[test]
    fn test_spaces() {
        let     pattern = Pattern::char(' ').many1();
        let mut state   = Group::default();

        state.add_rule(Rule{pattern, callback:"".into()});

        let nfa = NFA::from(&state);
        let dfa = DFA::from(&nfa);

        let expected_nfa = NFA {
            states: vec![
                State::from(vec![1]),
                State::from(vec![2]),
                State::from(vec![(32..=32,3)]),
                State::from(vec![4]),
                State::from(vec![5,8]),
                State::from(vec![6]),
                State::from(vec![(32..=32,7)]),
                State::from(vec![8]),
                State::from(vec![5,9]).named("group0_rule0"),
                State::default(),
            ],
            alphabet: Alphabet::from(vec![0, 32, 33]),
        };
        let expected_dfa = DFA {
            alphabet: Alphabet::from(vec![0, 32, 33]),
            links: DFA::links(vec![
                vec![I,1,I],
                vec![I,2,I],
                vec![I,2,I],
            ]),
            callbacks: vec![
                None,
                Some(Callback {priority:3,name:"group0_rule0".into()}),
                Some(Callback {priority:3,name:"group0_rule0".into()}),
            ]
        };
        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }

    #[test]
    fn test_letter_and_spaces() {
        let     letter = Pattern::range('a'..='z');
        let     spaces = Pattern::char(' ').many1();
        let mut state  = Group::default();

        state.add_rule(Rule{pattern:letter, callback:"".into()});
        state.add_rule(Rule{pattern:spaces, callback:"".into()});

        let nfa = NFA::from(&state);
        let dfa = DFA::from(&nfa);

        let expected_nfa = NFA {
            states: vec![
                State::from(vec![1,3]),
                State::from(vec![(97..=122,2)]),
                State::from(vec![11]).named("group0_rule0"),
                State::from(vec![4]),
                State::from(vec![(32..=32,5)]),
                State::from(vec![6]),
                State::from(vec![7,10]),
                State::from(vec![8]),
                State::from(vec![(32..=32,9)]),
                State::from(vec![10]),
                State::from(vec![7,11]).named("group0_rule1"),
                State::default(),
            ],
            alphabet: Alphabet::from(vec![32,33,97,123]),
        };
        let expected_dfa = DFA {
            alphabet: Alphabet::from(vec![32,33,97,123]),
            links: DFA::links(vec![
                vec![I,1,I,2,I],
                vec![I,3,I,I,I],
                vec![I,I,I,I,I],
                vec![I,3,I,I,I],
            ]),
            callbacks: vec![
                None,
                Some(Callback {priority:4,name:"group0_rule1".into()}),
                Some(Callback {priority:4,name:"group0_rule0".into()}),
                Some(Callback {priority:4,name:"group0_rule1".into()}),
            ]
        };

        assert_eq!(nfa,expected_nfa);
        assert_eq!(dfa,expected_dfa);
    }
}
