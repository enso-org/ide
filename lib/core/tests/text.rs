//! Test suite for the Web and headless browsers.
#![cfg(target_arch = "wasm32")]

use basegl::display::world::{World,WorldRef,Workspace,WorkspaceID,Add};
use basegl_system_web::{get_element_by_id, create_element, dyn_into, NodeInserter, StyleSetter,
                        get_webgl_context, Result, Error};
use web_sys::{HtmlElement,HtmlCanvasElement};


// ==================
// === World test ===
// ==================

/// A little framework for doing web tests with worlds
///
/// This should be a temporary solution - until world and htmlscene frameworks will be merged.
pub struct WorldTest {
    pub world_ptr    : WorldRef,
    pub workspace_id : WorkspaceID,
}

impl WorldTest {
    /// Set up new test with World
    ///
    /// This creates `canvas` element and add workspace operating on this canvas. Returns `None`
    /// if webgl context is unavailable (likely because test are run in headless browser - whe
    /// should not fail in that case)
    pub fn new(test_name:&str) -> Option<WorldTest> {
        let workspace_set_up = Self::setup_workspace_canvas(test_name);
        match workspace_set_up {
            Ok(())                         => Some(Self::create_world_with_workspace(test_name)),
            Err(Error::NoWebGL{version:_}) => None,
            other_error                    => {other_error.unwrap(); None}
        }
    }

    fn setup_workspace_canvas(test_name:&str) -> Result<()> {
        let root                               = get_element_by_id(test_name)?;
        let root_html      : HtmlElement       = dyn_into(root.clone())?;
        let canvas_element                     = create_element("canvas")?;
        let canvas_html    : HtmlElement       = dyn_into(canvas_element.clone())?;
        let canvas         : HtmlCanvasElement = dyn_into(canvas_element.clone())?;

        get_webgl_context(&canvas, 1)?;
        canvas.set_width(640);
        canvas.set_height(640);
        canvas_html.set_id(Self::workspace_name(test_name).as_str());
        root_html.set_property_or_panic("overflow", "scroll");
        root.append_or_panic(&canvas_html);
        Ok(())
    }

    fn create_world_with_workspace(test_name:&str) -> WorldTest {
        let world_ptr    = World::new();
        let workspace    = Workspace::build(Self::workspace_name(test_name));
        let workspace_id = world_ptr.borrow_mut().add(workspace);
        WorldTest {world_ptr,workspace_id}
    }

    fn workspace_name(test_name:&str) -> String {
        "workspace_".to_owned() + test_name
    }
}


#[cfg(test)]
mod tests {
    use web_test::*;

    use super::WorldTest;
    use basegl::Color;
    use basegl::display::world::World;
    use basegl::dirty::traits::SharedSetter1;
    use basegl::text::TextComponentBuilder;

    use basegl_core_msdf_sys::run_once_initialized;
    use nalgebra::{Point2,Vector2};

    web_configure!(run_in_browser);

    const SCROLLING_BENCHMARK_ITERATIONS : usize = 10;
    const TEST_TEXT : &str = "To be, or not to be, that is the question:\n\
        Whether 'tis nobler in the mind to suffer\n\
        The slings and arrows of outrageous fortune,\n\
        Or to take arms against a sea of troubles\n\
        And by opposing end them.";
    const LONG_TEXT : &str    = include_str!(concat!(env!("OUT_DIR"), "/long.txt"));
    const WIDE_TEXT : &str    = include_str!(concat!(env!("OUT_DIR"), "/wide.txt"));
    const FONTS     : &[&str] = &
        [ "DejaVuSans"
        , "DejaVuSansMono"
        , "DejaVuSansMono-Bold"
        , "DejaVuSerif"
        ];

    #[web_test]
    fn small_font() {
        if let Some(world_test) = WorldTest::new("small_font") {
            run_once_initialized(move || {
                let text         = TEST_TEXT.to_string();
                let size         = 0.025;
                create_test_components_for_each_font(&world_test,text,size);
            });
        }
    }

    #[web_test]
    fn normal_font() {
        if let Some(world_test) = WorldTest::new("normal_font") {
            run_once_initialized(move || {
                let text         = TEST_TEXT.to_string();
                let size         = 0.0375;
                create_test_components_for_each_font(&world_test,text,size);
            });
        }
    }

    #[web_test]
    fn big_font() {
        if let Some(world_test) = WorldTest::new("big_font") {
            run_once_initialized(move || {
                let text         = TEST_TEXT.to_string();
                let size         = 0.125;
                create_test_components_for_each_font(&world_test,text,size);
            });
        }
    }

    #[web_bench]
    fn scrolling_vertical(bencher:&mut Bencher) {
        if let Some(world_test) = WorldTest::new("scrolling_vertical") {
            let mut bencher_clone = bencher.clone();
            run_once_initialized(move || {
                create_full_sized_text_component(&world_test,LONG_TEXT.to_string());
                bencher_clone.iter(move || {
                    let world : &mut World = &mut world_test.world_ptr.borrow_mut();
                    for _ in 0..SCROLLING_BENCHMARK_ITERATIONS {
                        let workspace          = &mut world.workspaces[world_test.workspace_id];
                        let text_component     = &mut workspace.text_components[0];
                        text_component.scroll(Vector2::new(0.0,-1.0));
                        world.workspace_dirty.set(world_test.workspace_id);
                        world.update();
                    }
                });
            });
        }
    }

    #[web_bench]
    fn scrolling_horizontal(bencher:&mut Bencher) {
        if let Some(world_test) = WorldTest::new("scrolling_horizontal") {
            let mut bencher_clone = bencher.clone();
            run_once_initialized(move || {
                create_full_sized_text_component(&world_test,WIDE_TEXT.to_string());
                bencher_clone.iter(move || {
                    let world : &mut World = &mut world_test.world_ptr.borrow_mut();
                    for _ in 0..SCROLLING_BENCHMARK_ITERATIONS {
                        let workspace          = &mut world.workspaces[world_test.workspace_id];
                        let text_component     = &mut workspace.text_components[0];
                        text_component.scroll(Vector2::new(1.0,0.0));
                        world.workspace_dirty.set(world_test.workspace_id);
                        world.update();
                    }
                });
            });
        }
    }

    fn create_full_sized_text_component(world_test:&WorldTest, text:String) {
        let workspace_id       = world_test.workspace_id;
        let world : &mut World = &mut world_test.world_ptr.borrow_mut();
        let workspace          = &mut world.workspaces[workspace_id];
        let fonts              = &mut world.fonts;
        let font_name          = FONTS[0];
        let font_id            = fonts.load_embedded_font(font_name).unwrap();

        let text_component = TextComponentBuilder {
            workspace,fonts,text,font_id,
            position  : Point2::new(-1.0, -1.0),
            size      : Vector2::new(2.0, 2.0),
            text_size : 0.03125,
            color     : Color {r: 1.0, g: 1.0, b: 1.0, a: 1.0},
        }.build();
        workspace.text_components.push(text_component);
        world.workspace_dirty.set(workspace_id); // TODO[AO] Make dirty flags for component
    }

    fn create_test_components_for_each_font(world_test:&WorldTest, text:String, text_size:f64) {
        let workspace_id       = world_test.workspace_id;
        let world : &mut World = &mut world_test.world_ptr.borrow_mut();
        let workspace          = &mut world.workspaces[workspace_id];
        let fonts              = &mut world.fonts;

        for (i, font_name) in FONTS.iter().enumerate() {
            let x         = -1.0 + (i / 2) as f64;
            let y         = -1.0 + (i % 2) as f64;
            let font_id   = fonts.load_embedded_font(font_name).unwrap();
            let text_component = TextComponentBuilder {
                workspace,fonts,font_id,text_size,
                position  : Point2::new(x,y),
                text      : text.clone(),
                size      : Vector2::new(1.0,1.0),
                color     : Color{r:1.0, g:1.0, b:1.0, a:1.0}
            }.build();
            workspace.text_components.push(text_component);
        }
        world.workspace_dirty.set(workspace_id);
    }
}