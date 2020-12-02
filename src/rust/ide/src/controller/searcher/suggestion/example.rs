
#[derive(Clone,Debug,Default,Eq,PartialEq)]
pub struct Example {
    pub name          : String,
    pub code          : String,
    pub documentation : String,
}

const EXAMPLES:&[Example] = &
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
