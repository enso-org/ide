use parser::Parser;
use uuid::Uuid;
use wasm_bindgen_test::{wasm_bindgen_test_configure, wasm_bindgen_test};
use parser::api::Error::ParsingError;

wasm_bindgen_test_configure!(run_in_browser);


#[wasm_bindgen_test]
fn web_test() {
   let mut parser = Parser::new_or_panic();
   
   let mut parse = |input| {
      let uuid = Uuid::parse_str("02723954-fbb0-4641-af53-cec0883f260a").unwrap();
      match parser.parse(String::from(input), vec![((0,0),uuid)]) {
         Err(ParsingError(str)) => str,
         _ => panic!("Not implemented.")
      }
   };
   
   assert_eq!(parse(""), "\
      {\"shape\":{\"Module\":{\"lines\":[{\"elem\":null,\"off\":0}]}}\
      ,\"span\":0}"
   );
}
