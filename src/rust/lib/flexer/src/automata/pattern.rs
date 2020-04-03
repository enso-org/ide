use std::ops::BitOr;
use std::ops::BitAnd;
use std::ops::Range;
use core::iter;

const MAX:i64 = i64::max_value();
const MIN:i64 = i64::min_value();

#[derive(Clone,Debug)]
pub enum Pattern {
    Range(Range<i64>),
    Or(Vec<Pattern>),
    And(Vec<Pattern>),
    Many(Pattern)
}

use Pattern::*;
use crate::parser;

impl BitOr<Pattern> for Pattern {
    type Output = Pattern;

    fn bitor(self, rhs: Pattern) -> Self::Output {
        match (self,rhs) {
            (Or(&mut or), Or(    or2)) => {or.extend(or2) ; self},
            (Or(&mut or), _          ) => {or.push(rhs)   ; self},
            (_          , Or(&mut or)) => {or.push(self)  ; rhs },
            (_          , _          ) => Or(vec![self,rhs]),
        }
    }
}
impl BitAnd<Pattern> for Pattern {
    type Output = Pattern;

        fn bitand(self, rhs: Pattern) -> Self::Output {
        match (self,rhs) {
            (And(&mut or), And(    or2)) => {or.extend(or2) ; self},
            (And(&mut or), _           ) => {or.push(rhs)   ; self},
            (_           , And(&mut or)) => {or.push(self)  ; rhs },
            (_           , _           ) => And(vec![self,rhs]),
        }
    }
}

impl Pattern {

    pub fn never()         -> Self { Pattern::Range(0..-1)      }
    pub fn always()        -> Self { Pattern::Range(MIN..MAX)   }
    pub fn any_char()      -> Self { Pattern::Range(0..MAX)     }
    pub fn many(self)      -> Self { Many(self)                 }
    pub fn many1(self)     -> Self { self & self.many()         }
    pub fn opt(self)       -> Self { self | Self::always()      }
    pub fn code(code: i64) -> Self { Pattern::Range(code..code) }

    pub fn char(char:char) -> Self {
        Self::code((char as u32).into())
    }
    pub fn range(start:char, end:char) -> Self {
        Pattern::Range((start as u32).into()..(end as u32).into())
    }

    pub fn eof() -> Self { Self::code(parser::EOF_CODE) }

    pub fn all(chars:String) -> Self {
        chars.chars().fold(Self::never(), |a,b| a & Self::char(b))
    }

    pub fn any(chars:String) -> Self {
        chars.chars().fold(Self::never(), |a,b| a | Self::char(b))
    }

    pub fn none(chars:String) -> Self {
        let char_iter  = chars.chars().map(|c| i64::from(c as u32));
        let char_iter2 = iter::once(0).chain(char_iter).chain(iter::once(MAX));
        let mut codes  = char_iter2.collect::<Vec<i64>>()[..];

        codes.sort();
        codes.windows(2).fold(Self::never(), |a,(s,e)| {
            if e < s {a} else {
                a | Pattern::Range(s..e)
            }
        })
    }

    pub fn not(char:char) -> Self {
        Self::none_of(char.toString)
    }

    pub fn repeat(pat:Pattern, num:usize) -> Self {
        (0..num).fold(Self::always(), |p,_| p & pat)
    }

    pub fn repeat_between(pat:Pattern, min:usize, max:usize) -> Self {
        (min..max).fold(Self::never(), |p,n| p | Self::repeat(pat,n))
    }

}
