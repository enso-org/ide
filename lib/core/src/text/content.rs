use crate::prelude::*;

use crate::text::font::{FontId, FontRenderInfo, Fonts};

use std::ops::{Range,RangeFrom};
use failure::_core::ops::RangeInclusive;


// ==================
// === DirtyLines ===
// ==================

/// Set of dirty lines' indices
#[derive(Debug)]
pub struct DirtyLines {
    pub single_lines : HashSet<usize>,
    pub range        : Option<RangeFrom<usize>>
}

impl Default for DirtyLines {
    /// Default `DirtyLines` where no line is dirty.
    fn default() -> Self {
        Self {
            single_lines : HashSet::new(),
            range        : None,
        }
    }
}

impl DirtyLines {
    /// Mark single line as dirty.
    pub fn add_single_line(&mut self, index:usize) {
        self.single_lines.insert(index);
    }

    /// Mark an open range of lines as dirty.
    pub fn add_lines_range_from(&mut self, range:RangeFrom<usize>) {
        let current_is_wider = self.range.as_ref().map_or(false, |cr| cr.start <= range.start);
        if !current_is_wider {
            self.range = Some(range);
        }
    }

    /// Mark an open range of lines as dirty.
    pub fn add_lines_range(&mut self, range:RangeInclusive<usize>) {
        for i in range {
            self.add_single_line(i);
        }
    }

    /// Check if line is marked as dirty.
    pub fn is_dirty(&self, index:usize) -> bool {
        let range_contains = self.range.as_ref().map_or(false, |r| r.contains(&index));
        range_contains || self.single_lines.contains(&index)
    }

    /// Check if there is any dirty line
    pub fn any_dirty(&self) -> bool {
        self.range.is_some() || !self.single_lines.is_empty()
    }
}


// ============================
// === TextComponentContent ===
// ============================

#[derive(Debug)]
pub struct TextComponentContent {
    pub lines        : Vec<String>,
    pub dirty_lines : DirtyLines,
    pub font        : FontId,
}

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct CharPosition {
    pub line        : usize,
    pub byte_offset : usize,
}

/// References to all needed stuff for generating buffer's data.
pub struct RefreshInfo<'a, 'b> {
    pub lines       : &'a [String],
    pub dirty_lines : DirtyLines,
    pub font        : &'b mut FontRenderInfo,
}

impl TextComponentContent {
    pub fn new(font_id:FontId, text:&str) -> Self {
        TextComponentContent {
            lines       : text.split('\n').map(|s| s.to_string()).collect(),
            dirty_lines : DirtyLines::default(),
            font        : font_id,
        }
    }

    pub fn refresh_info<'a,'b>(&'a mut self, fonts:&'b mut Fonts) -> RefreshInfo<'a,'b> {
        RefreshInfo {
            lines       : &mut self.lines,
            dirty_lines : std::mem::take(&mut self.dirty_lines),
            font        : fonts.get_render_info(self.font)
        }
    }
}

// =============
// ===
// =============

pub struct TextReplacement {
    pub range : Range<CharPosition>,
    pub lines : Vec<String>,
}

pub struct TextEdit<'a> {
    pub content      : &'a mut TextComponentContent,
    pub replacements : Vec<TextReplacement>,
}

impl<'a> TextEdit<'a> {
    pub fn new(content:&'a mut TextComponentContent, mut replacements:Vec<TextReplacement>) -> Self {
        replacements.sort_by_key(|r| (r.range.start.clone(), r.range.end.clone()));
        TextEdit {content,replacements}
    }

    pub fn do_edit(mut self) {
        for replacement in self.replacements.iter_mut().rev() {
            let start       = &replacement.range.start;
            let end         = &replacement.range.end;
            let mut lines   = std::mem::take(&mut replacement.lines);
            let lines_count = lines.len();
            if start.line == end.line && lines.len() == 1 {
                let mut edited_line = std::mem::take(&mut self.content.lines[start.line]);
                edited_line.replace_range(start.byte_offset..end.byte_offset, lines.first().unwrap().as_str());
                *lines.first_mut().unwrap() = edited_line;
            } else if start.line == end.line {
                let mut edited_line = std::mem::take(&mut self.content.lines[start.line]);
                let suffix = edited_line[end.byte_offset..].to_string();
                edited_line.replace_range(start.byte_offset.., &lines[0]);
                *lines.first_mut().unwrap() = edited_line;
                *lines.last_mut().unwrap()  = suffix;
            } else if lines.len() > 1 {
                let mut first_edited_line = std::mem::take(&mut self.content.lines[start.line]);
                let mut last_edited_line  = std::mem::take(&mut self.content.lines[end.line]);
                first_edited_line.replace_range(start.byte_offset.., &lines.first().unwrap());
                last_edited_line.replace_range(..end.byte_offset, &lines.last().unwrap());
                *lines.first_mut().unwrap() = first_edited_line;
                *lines.last_mut().unwrap()  = last_edited_line;
            } else {
                let mut first_edited_line = std::mem::take(&mut self.content.lines[start.line]);
                let last_edited_line      = std::mem::take(&mut self.content.lines[end.line]);
                first_edited_line.replace_range(start.byte_offset.., &lines.first().unwrap());
                first_edited_line.push_str(last_edited_line.as_str());
                *lines.first_mut().unwrap() = first_edited_line;
            }
            self.content.lines.splice(start.line..=end.line, lines);
            if (end.line - start.line + 1) != lines_count {
                self.content.dirty_lines.add_lines_range_from(start.line..);
            } else {
                self.content.dirty_lines.add_lines_range(start.line..=end.line);
            }
        }
    }
}