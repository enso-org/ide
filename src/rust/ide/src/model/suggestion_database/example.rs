//! A module with structures related to examples found in Suggestion Database.
use crate::prelude::*;

use crate::double_representation::definition::DefinitionName;
use crate::double_representation::module;
use crate::double_representation::definition;

use parser::Parser;



// ==============
// === Errors ===
// ==============

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display = "Invalid example code.")]
pub struct InvalidExample;



// ===============
// === Example ===
// ===============

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

impl Example {
    /// Return the example name converted in such way, that
    ///
    /// #### Example
    /// ```
    /// use enso_prelude::*;
    ///
    /// use ide::model::suggestion_database::Example;
    ///
    /// let name    = "With Spaces and Strange $#%^& Characters.".to_owned();
    /// let example = Example{name,..default()};
    /// assert_eq!(example.function_name(), "with_spaces_and_strange__characters");
    /// ```
    pub fn function_name(&self) -> String {
        self.name.chars().filter_map(|c|
            if c == ' ' { Some('_') }
            else if !c.is_ascii_alphanumeric() { None }
            else { Some(c.to_ascii_lowercase()) }
        ).collect()
    }

    /// Returns the function definition containing the example code.
    pub fn definition_to_add(&self, module:&module::Info, parser:&Parser)
    -> FallibleResult<definition::ToAdd> {
        let base_name  = self.function_name();
        let name       = DefinitionName::new_plain(module.generate_name(&base_name)?);
        let code_ast   = parser.parse_module(self.code.clone(),default())?;
        let body_block = code_ast.shape().as_block(0).ok_or(InvalidExample)?;
        let body_ast   = Ast::new(body_block,None);
        Ok(definition::ToAdd::new_with_body(name,default(),body_ast))
    }
}



// =========================
// === Embedded Examples ===
// =========================

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
