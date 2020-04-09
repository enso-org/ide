use parser::Parser;
use enso_prelude::default;
use ast::HasIdMap;


// TODO [ao] this test isn't passing perhaps due to bug in parser.
#[ignore]
#[test]
fn parsing_main_with_id_map() {
    const PROGRAM : &str = r#"main =
    2 + 2"#;
    let parser = Parser::new().unwrap();
    let ast1 = parser.parse(PROGRAM.to_string(),default()).unwrap();
    let id_map = ast1.id_map();
    println!("{:?}", id_map);
    let ast2 = parser.parse(PROGRAM.to_string(),id_map).unwrap();
    assert_eq!(ast1,ast2)
}
