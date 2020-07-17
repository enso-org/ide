use enso_prelude::*;



/// Simple interactive tester - calls parser with its argument (or a
/// hardcoded default) and prints the result
fn main() {
    let default_input = String::from("import Foo.Bar\nfoo = a + 2");
    let program = std::env::args().nth(1).unwrap_or(default_input);
    println!("Will parse: {}", program);

    let parser = parser::Parser::new_or_panic();
    let output = parser.parse(program, default());
    match output {
        Ok(result) => println!("Parser responded with: {:?}", result),
        Err(e)     => println!("Failed to obtain a response: {:?}", e),
    }
    

    let default_input = String::from("##\n  DEPRECATED\n  Foo bar baz\ntype Foo\n  type Bar");
    let program = std::env::args().nth(1).unwrap_or(default_input);
    println!("Will parse: {}", program);

    let parser = parser::DocParser::new_or_panic();
    let output = parser.doc_parser_generate_html_source(program);
    match output {
        Ok(result) => println!("Parser responded with: {:?}", result),
        Err(e)     => println!("Failed to obtain a response: {:?}", e),
    }
}
