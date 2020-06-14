
/// Possible navigation patterns.
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Navigation {
    /// Move to the left by one grapheme cluster.
    Left,
    /// Move to the right by one grapheme cluster.
    Right,
    //    /// Move to the left by one word.
//    LeftWord,
//    /// Move to the right by one word.
//    RightWord,
    /// Move to left end of visible line.
    LeftOfLine,
    /// Move to right end of visible line.
    RightOfLine,
    /// Move up one visible line.
    Up,
    /// Move down one visible line.
    Down,
    /// Move up one viewport height.
    UpPage,
    /// Move down one viewport height.
    DownPage,
    /// Move up to the next line that can preserve the cursor position.
    UpExactPosition,
    /// Move down to the next line that can preserve the cursor position.
    DownExactPosition,
    /// Move to the start of the text line.
    StartOfParagraph,
    /// Move to the end of the text line.
    EndOfParagraph,
    /// Move to the end of the text line, or next line if already at end.
    EndOfParagraphKill,
    /// Move to the start of the document.
    StartOfDocument,
    /// Move to the end of the document
    EndOfDocument,
}
