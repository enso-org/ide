#![allow(missing_docs)]

use ensogl::traits::*;

use ensogl::system::web;
use ensogl::system::web::NodeInserter;
use ensogl::display::symbol::DomSymbol;
use web::StyleSetter;
use ensogl::display::world::*;
use ensogl::display::navigation::navigator::Navigator;
use ensogl::prelude::*;

use nalgebra::Vector2;
use nalgebra::Vector3;
use wasm_bindgen::prelude::*;

fn generate_mock_doc() -> String {
    let inp_only_doc = r#"DEPRECATED
                          REMOVED - replaced by Foo Bar
                          ADDED
                          MODIFIED
                          UPCOMING
                          ALAMAKOTA a kot ma Ale
                          This is a test of Enso Documentation Parser. This is a short synopsis.

                          Here you can write the body of documentation. On top you can see tags
                          added to this piece of code. You can customise your text with _Italic_
                          ~Strikethrough~ or *Bold*. ~_*Combined*_~ is funny


                          There are 3 kinds of sections
                            - Important
                            - Info
                            - Example
                              * You can use example to add multiline code to your documentation

                          ! Important
                            Here is a small test of Important Section

                          ? Info
                            Here is a small test of Info Section

                          > Example
                            Here is a small test of Example Section
                                Import Foo
                                def Bar a"#;
    let default_input = String::from(inp_only_doc);
    let program = std::env::args().nth(1).unwrap_or(default_input);

    let parser = parser::DocParser::new_or_panic();
    let output = parser.generate_html_doc_pure(program);
    output.unwrap_or_else(|_| String::from("<h1>hello EnsoGL</h1>"))
}


#[wasm_bindgen]
#[allow(dead_code)]
#[allow(clippy::many_single_char_names)]
pub fn run_example_documentation_generator_view() {
    // FIXME : For now removing external CSS file, as I dont know yet how to load it properly
    //         And where to place it, so for now CSS is a multiline string just added to generated
    //         documentation string.
    let output_unwrapped = generate_mock_doc().replace(r#"<link rel="stylesheet" href="style.css" />"#, "");
    let css = r#"
    <style>
body {
  -webkit-font-smoothing: antialiased;
  font-style: normal;
  word-wrap: break-word;
  font-size: 17px;
  line-height: 1.52947;
  font-weight: 400;
  letter-spacing: -0.021em;
  font-family: "SF Pro Text", "SF Pro Icons", "Helvetica Neue", "Helvetica", "Arial", sans-serif;
  background-color: white;
  color: #333333;
  margin: 0;
  padding: 0;
}

p {
  display: block;
  margin-block-start: 1em;
  margin-block-end: 1em;
  margin-inline-start: 0;
  margin-inline-end: 0;
}

a:hover {
  color: #0070c9 !important;
  text-decoration: inherit;
}

a {
  color: #333333;
  background-color: transparent;
  text-decoration: inherit;
  display: inline-block;
  transition: all 0.3s ease;
}

img {
  display: block;
}

code {
  color: #0070c9;
  background-color: transparent;
  font-size: inherit;
  font-family: "SF Pro Text", "SF Pro Icons", "Helvetica Neue", "Helvetica", "Arial", sans-serif;
  line-height: inherit;
  display: inline-block;
  white-space: pre-wrap;
}

button {
  display: inline-block;
  padding: 8px 30px;
  margin: 10px 0;
  outline: none;
  background-color: transparent;
  border: 1px solid #333333;
  color: #333333;
  border-radius: 5px;
  font-size: 13px;
  vertical-align: top;
  transition: all 0.3s ease;
}

button:hover {
  background-color: #333333;
  color: #e5e5e5;
}

b {
  font-weight: 600;
}

h1 {
  font-size: 34px;
  line-height: 1.08824;
  font-weight: 500;
  letter-spacing: 0.01em;
}

h2 {
  font-size: 28px;
  line-height: 1.1073;
  font-weight: 500;
  letter-spacing: 0.012em;
}

.Body h2 {
  margin: 0.65rem 0 0;
}

li {
  padding-left: 10px;
}

/*/////////////////// */
.creator .Unclosed,
.creator .invalidIndent,
.creator .invalidLink {
  display: inline;
  color: orangered;
}
.creator .Tags .UNRECOGNIZED {
  border: 2px solid;
  color: orangered;
}

.Unclosed,
.invalidIndent,
.invalidLink {
  display: inline;
}

/*////////////// */
.Header {
  font-size: 19px;
  font-weight: 500;
}

.Important .Header,
.Info .Header,
.Example .Header {
  margin-bottom: 0.7em;
  font-weight: 600;
  letter-spacing: -0.021em;
  line-height: 17px;
  font-synthesis: none;
  font-family: "SF Pro Text", "SF Pro Icons", "Helvetica Neue", "Helvetica", "Arial", sans-serif;
}

/*//////////// */
.Tags {
  margin-left: auto;
  margin-right: auto;
  margin-bottom: 20px;
  padding-top: 15px;
}
.Tags .DEPRECATED,
.Tags .MODIFIED,
.Tags .ADDED,
.Tags .UPCOMING,
.Tags .REMOVED,
.Tags .UNRECOGNIZED {
  line-height: 1.5;
  font-weight: 400;
  border-radius: 4px;
  font-size: 12px;
  letter-spacing: -0.021em;
  display: inline-flex;
  padding: 5px 15px;
  margin: 2px;
  white-space: nowrap;
  background: transparent;
}
.Tags .DEPRECATED {
  border: 1px solid #d20606;
  color: #d20606;
}
.Tags .MODIFIED {
  border: 1px solid #003ec3;
  color: #003ec3;
}
.Tags .ADDED {
  border: 1px solid #79A129;
  color: #79A129;
}
.Tags .UPCOMING,
.Tags .REMOVED,
.Tags .UNRECOGNIZED {
  border: 1px solid #666666;
  color: #666666;
}

.ExtForTagDetails {
  margin: 0 3px;
  color: #999999;
}

/*//////////////// */
.Raw,
.Important,
.Info,
.CodeBlock,
.Example {
  margin-top: 0;
  margin-left: auto;
  margin-right: auto;
  position: relative;
  text-decoration: inherit;
}

.Body .Raw {
  margin-bottom: 0.6rem;
  font-size: 17px;
  line-height: 1.52947;
  font-weight: 400;
  letter-spacing: -0.021em;
  font-family: "SF Pro Text", "SF Pro Icons", "Helvetica Neue", "Helvetica", "Arial", sans-serif;
  color: #333333;
  font-style: normal;
}

.Important,
.Info,
.CodeBlock,
.Example {
  font-size: 17px;
  padding: 15px 10px 15px 20px;
  border: 0;
  border-radius: 6px;
  margin: 0.7em 0;
}

.Important {
  background-color: #FBECC2;
}

.Info {
  background-color: #D6E1CA;
}

.Example {
  background-color: #fafafa;
}

.CodeBlock {
  background-color: #fefefe;
  margin: 10px 20px;
  display: none;
}
.CodeBlock code {
  font-family: monospace;
}

/*/////////////////////////////////// */
.Def {
  margin: 40px auto auto;
  padding: 0 15px;
  text-decoration: inherit;
}
.Def .Synopsis,
.Def .Body,
.Def .Tags,
.Def .ASTData {
  padding-left: 0;
  text-decoration: inherit;
}
.Def .Synopsis {
  padding: 0;
  margin-bottom: 15px;
  font-size: 17px;
  font-weight: 400;
  color: #333333;
  font-style: normal;
}
.Def .constr {
  padding: 25px 0;
  margin: 0;
}
.Def .DefDoc .Body {
  display: none;
}
.Def .DefDoc .documentation {
  display: inline-flex;
  width: 100%;
  margin-bottom: 10px;
}
.Def .DefDoc .documentation .ASTHead {
  width: 30% !important;
  margin: 10px 0;
}
.Def .DefDoc .documentation .ASTHead .DefTitle,
.Def .DefDoc .documentation .ASTHead .Infix {
  padding: 0;
  font-size: 17px;
  font-weight: 400;
  font-style: normal;
  text-decoration: inherit;
}
.Def .DefDoc .documentation .ASTData {
  width: 70% !important;
}
.Def .DefDoc .documentation .Doc {
  text-decoration: inherit;
}
.Def .DefDoc .documentation .Doc .Synopsis {
  text-decoration: inherit;
  margin: 10px 0;
}
.Def .DefDoc .documentation .Tags {
  margin: 2px 0 0 auto;
  padding: 0;
}
.Def .DefNoDoc {
  padding-bottom: 10px;
}

.DefTitle {
  display: inline-flex;
  font-size: x-large;
  font-weight: 400;
  margin-bottom: 20px;
}

.DefArgs {
  margin-left: 5px;
  font-weight: 400;
  color: #0070c9;
}

/*///////////////////////// */
.Synopsis,
.Body {
  margin: 0 auto;
  padding: 5px;
  text-align: left;
}

.Synopsis {
  margin-top: 35px;
  font-size: 20px;
}

.Documentation .ASTData,
.Documentation .ASTHead {
  text-align: left;
  line-height: 1.05;
  border-radius: 6px;
}
.Documentation .ASTData {
  width: 100%;
  background-color: #fafafa;
}
.Documentation .ASTHead {
  margin: 20px auto 5px;
  background-color: #ffffff;
}
.Documentation .ASTHead .DefTitle {
  font-size: 42px;
  margin: 0;
}
.Documentation .ASTData .ASTHead {
  background-color: #fafafa;
}
.Documentation .ASTData .ASTHead .DefTitle {
  font-size: x-large;
}
.Documentation .Documented {
  margin: 0;
  width: 100%;
  background-color: #ffffff;
}
.Documentation .DefNoBody {
  text-decoration: inherit;
}

/*/////////////////////////// */
@media (max-width: 500px) {
  .Synopsis,
.Body,
.Tags,
.Documentation .ASTData .Def {
    max-width: 380px;
  }

  .Documentation .ASTHead,
.DefNoBody,
.DefBody {
    max-width: 400px;
  }

  .Def {
    padding: 5px;
  }
}
@media (min-width: 500px) {
  .Synopsis,
.Body,
.Tags,
.Documentation .ASTData .Def {
    max-width: 440px;
  }

  .Documentation .ASTHead,
.DefNoBody,
.DefBody {
    max-width: 470px;
  }
}
@media (min-width: 600px) {
  .Synopsis,
.Body,
.Tags,
.Documentation .ASTData .Def {
    max-width: 490px;
  }

  .Documentation .ASTHead,
.DefNoBody,
.DefBody {
    max-width: 520px;
  }
}
@media (min-width: 900px) {
  .Synopsis,
.Body,
.Tags,
.Documentation .ASTData .Def {
    max-width: 680px;
  }

  .Documentation .ASTHead,
.DefNoBody,
.DefBody {
    max-width: 710px;
  }
}
@media (min-width: 1300px) {
  .Synopsis,
.Body,
.Tags,
.Documentation .ASTData .Def {
    max-width: 790px;
  }

  .Documentation .ASTHead,
.DefNoBody,
.DefBody {
    max-width: 820px;
  }
}

</style>
"#;
    let full_file: String = format!("{}{}", css, output_unwrapped);

    web::forward_panic_hook_to_console();
    web::set_stdout();
    let world         = World::new(&web::get_html_element_by_id("root").unwrap());
    let scene         = world.scene();
    let camera        = scene.camera();
    let screen        = camera.screen();
    let navigator     = Navigator::new(scene,camera);
    let dom_front_layer = &scene.dom.layers.main;
    let dom_back_layer  = &scene.dom.layers.overlay;

    let div = web::create_div();
    div.set_style_or_panic("width"  , "100% !important");
    div.set_style_or_panic("height" , "100% !important");
    div.set_inner_html(&full_file);

    let width  = screen.width;
    let height = screen.height;

    let mut css3d_objects: Vec<DomSymbol> = default();
    let size       = Vector2::new(width, height);
    let position   = Vector3::new(0.0, 0.0, 0.0);
    let object     = DomSymbol::new(&div);
    dom_front_layer.manage(&object);
    world.add_child(&object);
    let r          = (255.0) as u8;
    let g          = (255.0) as u8;
    let b          = (255.0) as u8;
    let color      = iformat!("rgb({r},{g},{b})");
    div.set_style_or_panic("background-color",color);

    object.dom().append_or_panic(&div);
    object.set_size(size);
    object.mod_position(|t| *t = position);
    css3d_objects.push(object);

    world.display_object().update();

    let layers = vec![dom_front_layer.clone_ref(),dom_back_layer.clone_ref()];

    world.keep_alive_forever();
    world.on_frame(move |_| {
        let _keep_alive = &navigator;

        for (_, object) in css3d_objects.iter_mut().enumerate() {
            layers[0].manage(&object);
        }
    }).forget();
}
