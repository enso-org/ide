//! A module with structures related to examples found in Suggestion Database.
use crate::prelude::*;

/// Example is a titled piece of code user can put into their graph to see and learn how to use
/// our language.
///
/// If user picks the example, the `code` should became a body of a new method defined in current
/// module. On scene the call for this method should appear.
#[allow(missing_docs)]
#[derive(Clone,Debug,Default,Eq,PartialEq)]
pub struct Example {
    pub name          : String,
    pub code          : String,
    pub documentation : String,
}

lazy_static! {
    /// The hard-coded examples to be used until the proper solution
    /// (described in https://github.com/enso-org/ide/issues/1011) will be implemented.
    pub static ref EXAMPLES:Vec<Example> = vec!
    [ Example {name: "Split an Example".to_owned(), code: r#"
        example = File.read "/home/adam-praca/Documents/example"
        example.split " "
        "#.to_owned(),
        documentation: "Lorem ipsum".to_owned()}
    , Example {name: "Table".to_owned(), code: r#"
        [2,6,35,678,9038,7390]
        "#.to_owned(),
        documentation: "Lorem ipsum".to_owned()}
    ];
}
