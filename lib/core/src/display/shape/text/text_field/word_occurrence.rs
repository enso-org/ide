//! This module contains definitions for word occurrences in TextField.

use crate::prelude::*;

use std::ops::Range;

use crate::display::shape::text::text_field::content::TextFieldContent;
use crate::display::shape::text::text_field::cursor::Cursor;
use data::text::TextLocation;



// =================
// === WordRange ===
// =================

/// A struct containing indices for word's start and end.
#[derive(Shrinkwrap,Debug,Clone)]
pub struct WordRange {
    /// A property containing the word start and end.
    #[shrinkwrap(main_field)]
    pub word_range : Range<TextLocation>,
    is_selected    : bool
}

impl WordRange {
    /// Creates a new `WordRange`.
    pub fn new(word_range : Range<TextLocation>) -> Self {
        let is_selected = false;
        Self {word_range,is_selected}
    }

    /// Returns a word under cursor, if any.
    pub fn word_at_cursor(content:&TextFieldContent, cursor:&Cursor) -> Option<Self> {
        let line  = cursor.position.line;
        let chars = content.lines()[line].chars();
        if let Some(range) = get_index_range_of_word_at(chars,cursor.position.column) {
            let column = range.start;
            let start  = TextLocation {line,column};
            let column = range.end;
            let end    = TextLocation {line,column};
            Some(Self::new(start..end))
        } else {
            None
        }
    }
}



// =======================
// === WordOccurrences ===
// =======================

/// A struct containing word occurrences in `TextFieldContent`.
#[derive(Shrinkwrap,Clone,Debug)]
pub struct WordOccurrences {
    /// Words occurrences.
    #[shrinkwrap(main_field)]
    pub words     : Vec<WordRange>,
    current_index : usize
}

impl WordOccurrences {
    /// Gets the range of each occurrence of word.
    pub fn new(content:&TextFieldContent, cursor:&mut Cursor) -> Option<Self> {
        let range = if cursor.has_selection() {
            Some(cursor.selection_range())
        } else {
            let word = WordRange::word_at_cursor(content, &cursor);
            word.map(|word| Some(word.word_range)).unwrap_or(None)
        };
        if let Some(range) = range {
            let word = content.copy_fragment(range);
            let cursor_location = cursor.position;
            let mut words = Vec::new();
            let word: Vec<char> = word.chars().collect();

            for (index, line) in content.lines().iter().enumerate() {
                let line_chars = line.chars();
                let words_in_line = get_word_occurrences(line_chars, &word);
                for word in words_in_line {
                    let line = index;
                    let column = word.start;
                    let start = TextLocation { line, column };
                    let column = word.end;
                    let end = TextLocation { line, column };
                    words.push(WordRange::new(start..end));
                }
            }

            if words.is_empty() {
                None
            } else {
                let current_index      = words.iter().find_position(|current_word| {
                    cursor_location.line == current_word.start.line &&
                        cursor_location.column >= current_word.start.column &&
                        cursor_location.column <= current_word.end.column
                }).map(|(index, _)| index).unwrap_or(0);
                let current_index = current_index - 1;
                Some(Self { words, current_index }.initialize(&cursor))
            }
        } else {
            None
        }
    }

    fn initialize(mut self, cursor:&Cursor) -> Self {
        if cursor.has_selection() {
            self.advance()
        }
        self
    }

    fn current_mut(&mut self) -> &mut WordRange {
        &mut self.words[self.current_index]
    }

    fn advance(&mut self) {
        self.current_index = (self.current_index + 1) % self.words.len();
    }

    /// Get next word occurrence if not already selected.
    pub fn select_next(&mut self) -> Option<WordRange> {
        self.advance();
        let mut word = self.current_mut();
        if word.is_selected {
            None
        } else {
            word.is_selected = true;
            Some(word.clone())
        }
    }
}



// =============
// === Utils ===
// =============

fn get_words(content:&[char]) -> Vec<Vec<(usize,char)>> {
    let indexed:Vec<(usize,char)> = content.iter().copied().enumerate().collect();
    let words = indexed.split(|(_,character)| !character.is_alphanumeric() && *character != '_');
    let words = words.filter(|word_in_context| !word_in_context.is_empty());
    words.map(|c| c.to_vec()).collect()
}

fn get_index_range_of_word_at(content:&[char], index:usize) -> Option<Range<usize>> {
    let words       = get_words(content);
    let mut ranges  = words.iter().map(|word| {
        let (start,_) = word.first().unwrap();
        let end       = start + word.len();
        *start..end
    });
    ranges.find(|word_range| index >= word_range.start && index <= word_range.end)
}

fn get_word_occurrences(content:&[char], word:&[char]) -> Vec<Range<usize>> {
    let mut occurrences = Vec::new();

    for word_in_content in get_words(content) {
        let count = word_in_content.iter().zip(word).filter(|&((_, a), b)| a == b).count();
        if count == word_in_content.len() {
            let (start,_) = word_in_content.first().unwrap();
            let end       = start + word_in_content.len();
            occurrences.push(*start..end)
        }
    }

    occurrences
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn index_range_of_word_at() {
        let content           = String::from("   abc    def    ghi  ");
        let content:Vec<char> = content.chars().collect();
        let range             = get_index_range_of_word_at(&content,12);
        assert_eq!(range, Some(10..13));
    }

    #[test]
    fn word_occurrences() {
        let content = String::from("   abc    def    ghi  abcabc abc");
        let content:Vec<char> = content.chars().collect();
        let word              = String::from("abc");
        let word:Vec<char>    = word.chars().collect();
        let occurrences       = get_word_occurrences(&content,&word);
        assert_eq!(occurrences[0],3..6);
        assert_eq!(occurrences[1],29..32);
    }
}
