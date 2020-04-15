use crate::parser;
use crate::automata::nfa::NFA;
use crate::automata::state::Symbol;
use crate::automata::state::Id;

use core::iter;
use itertools::Itertools;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::RangeInclusive;


// =============
// == Pattern ==
// =============

/// Simple regex pattern.
#[derive(Clone,Debug)]
pub enum Pattern {
    /// Pattern that triggers on any symbol from given range.
    Range(RangeInclusive<Symbol>),
    /// Pattern that triggers on any given pattern from sequence.
    Or(Vec<Pattern>),
    /// Pattern that triggers when a sequence of patterns is encountered.
    And(Vec<Pattern>),
    /// Pattern that triggers on 0..N repetitions of given pattern.
    Many(Box<Pattern>)
}

use Pattern::*;

impl BitOr<Pattern> for Pattern {
    type Output = Pattern;
    fn bitor(self, rhs: Pattern) -> Self::Output {
        match (self, rhs) {
            (Or(mut lhs), Or(    rhs)) => {lhs.extend(rhs) ; Or(lhs)},
            (Or(mut lhs), rhs        ) => {lhs.push(rhs)   ; Or(lhs)},
            (lhs        , Or(mut rhs)) => {rhs.push(lhs)   ; Or(rhs)},
            (lhs        , rhs        ) => Or(vec![lhs,rhs]),
        }
    }
}

impl BitAnd<Pattern> for Pattern {
    type Output = Pattern;
    fn bitand(self, rhs: Pattern) -> Self::Output {
        match (self, rhs) {
            (And(mut lhs), And(    rhs)) => {lhs.extend(rhs) ; And(lhs)},
            (And(mut lhs), rhs         ) => {lhs.push(rhs)   ; And(lhs)},
            (lhs         , And(mut rhs)) => {rhs.push(lhs)   ; And(rhs)},
            (lhs         , rhs         ) => And(vec![lhs,rhs]),
        }
    }
}

impl Pattern {

    /// Pattern that never triggers.
    pub fn never() -> Self {
        Pattern::symbols(0..=-1)
    }

    /// Pattern that always triggers.
    pub fn always() -> Self {
        Pattern::symbols(i64::min_value()..=i64::max_value())
    }

    /// Pattern that triggers on any char.
    pub fn any_char() -> Self {
        Pattern::symbols(0..=i64::from(u32::max_value()))
    }

    /// Pattern that triggers on 0..N repetitions of given pattern.
    pub fn many(self) -> Self {
        Many(Box::new(self))
    }

    /// Pattern that triggers on 1..N repetitions of given pattern.
    pub fn many1(self) -> Self {
        self.clone() & self.many()
    }

    /// Pattern that triggers on 0..1 repetitions of given pattern.
    pub fn opt(self) -> Self {
        self | Self::always()
    }

    /// Pattern that triggers on given symbol
    pub fn symbol(symbol:i64) -> Self {
        Pattern::symbols(symbol..=symbol)
    }

    /// Pattern that triggers on any of the given symbols.
    pub fn symbols(symbols:RangeInclusive<i64>) -> Self {
        let start = Symbol{val:*symbols.start()};
        let end   = Symbol{val:*symbols.end()};
        Pattern::Range(start..=end)
    }

    /// Pattern that triggers on end of file.
    pub fn eof() -> Self {
        Self::symbol(parser::EOF_CODE.val)
    }

    /// Pattern that triggers on given character.
    pub fn char(char:char) -> Self {
        Self::symbol((char as u32).into())
    }


    /// Pattern that triggers on any of the given characters.
    pub fn range(chars:RangeInclusive<char>) -> Self {
        let start = i64::from(*chars.start() as u32);
        let end   = i64::from(*chars.end()   as u32);
        Pattern::symbols(start..=end)
    }

    /// Pattern that triggers when sequence of characters is encountered.
    pub fn all(chars:String) -> Self {
        chars.chars().fold(Self::never(), |pat,char| pat & Self::char(char))
    }

    /// Pattern that triggers on any characters from given sequence.
    pub fn any(chars:String) -> Self {
        chars.chars().fold(Self::never(), |pat,char| pat | Self::char(char))
    }

    /// Pattern that doesn't trigger on any given character from given sequence.
    pub fn none(chars:String) -> Self {
        let max        = i64::max_value();
        let char_iter  = chars.chars().map(|char| i64::from(char as u32));
        let char_iter2 = iter::once(0).chain(char_iter).chain(iter::once(max));
        let mut codes  = char_iter2.collect_vec();

        codes.sort();
        codes.iter().tuple_windows().fold(Self::never(), |pat,(start,end)| {
            if end < start {pat} else {
                pat | Pattern::symbols(*start..=*end)
            }
        })
    }

    /// Pattern that triggers on any character but the one given.
    pub fn not(char:char) -> Self {
        Self::none(char.to_string())
    }

    /// Pattern that triggers on N repetitions of given pattern.
    pub fn repeat(pat:Pattern, num:usize) -> Self {
        (0..num).fold(Self::always(), |p,_| p & pat.clone())
    }

    /// Pattern that triggers on MIN..MAX repetitions of given pattern.
    pub fn repeat_between(pat:Pattern, min:usize, max:usize) -> Self {
        (min..max).fold(Self::never(), |p,n| p | Self::repeat(pat.clone(),n))
    }

    /// Transforms pattern to NFA.
    /// The algorithm is based on: https://www.youtube.com/watch?v=RYNN-tb9WxI
    pub fn to_nfa(&self, nfa:&mut NFA, last: Id) -> Id {
        let current = nfa.new_state();
        nfa.connect(last, current);
        match self {
            Pattern::Range(range) => {
                let state = nfa.new_state();
                nfa.connect_by(current, state, range);
                state
            },
            Pattern::Many(body) => {
                let s1 = nfa.new_state();
                let s2 = body.to_nfa(nfa, s1);
                let s3 = nfa.new_state();
                nfa.connect(current, s1);
                nfa.connect(current, s3);
                nfa.connect(s2, s3);
                nfa.connect(s3, s1);
                s3
            },
            Pattern::And(patterns) => {
                let build = |s,pat:&Self| pat.to_nfa(nfa, s);
                patterns.iter().fold(current,build)
            },
            Pattern::Or(patterns) => {
                let build  = |pat:&Self| pat.to_nfa(nfa, current);
                let states = patterns.iter().map(build).collect_vec();
                let end    = nfa.new_state();
                for state in states {
                    nfa.connect(state, end);
                }
                end
            }
        }
    }
}
