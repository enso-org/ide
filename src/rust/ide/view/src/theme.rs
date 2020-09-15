//! Application theme setup.
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::style::theme;


/// `define_theme` helper.
macro_rules! _define_theme {
    ($name:ident = $e:expr) => {
        println!("const {:?} : &str = __path__",stringify!($name));
        println!("__theme_name__.insert(\"__path__\", {});",stringify!($e))
        //$theme_name.insert("$path.$name", $e);
    };

    ($name:ident = $e:expr; $($rest:tt)*) => {
        _define_theme!($name = $e);
        _define_theme!($($rest)*);
    };

    ($name:ident {$($t:tt)*}) => {
        println!("pub mod {:?} {{",stringify!($name));
        _define_theme!($($t)*);
        println!("}}")
    };

    ($name:ident {$($t:tt)*} $($rest:tt)*) => {
        _define_theme!($name {$($t)*});
        _define_theme!($($rest)*);
    };
}

/// Used to define theme.
#[macro_export]
macro_rules! define_theme {
    ($name:ident $($t:tt)*) => {
        println!("pub mod Vars {{ //{:?} theme.",stringify!($name));
        // let mut $name = theme::Theme::new();
        _define_theme!($($t)*);
        println!("}}")
    };
}

/// Used to set up themes for the application.
pub fn setup(app:&Application) {
    define_theme! { dark
        application {
            background {
                color = color::Lcha::new(0.13,0.013,0.18,1.0)
            }
            text {
                color = color::Lcha::new(1.0,0.0,0.0,0.7);
                selection {
                    color = color::Lcha::new(0.7,0.0,0.125,0.7)
                }
            }
        }
        graph_editor {
            node {
                background {
                    color = color::Lcha::new(0.2,0.013,0.18,1.0)
                }
                selection {
                    color = color::Lcha::new(0.72,0.5,0.22,1.0);
                    size = 7.0
                }
            }
            visualization {
                background {
                    color = color::Lcha::new(0.2,0.013,0.18,1.0)
                }
            }
        }
        breadcrumbs {
            full {
                color = color::Lcha::new(1.0,0.0,0.0,0.7)
            }
            transparent {
                color = color::Lcha::new(1.0,0.0,0.0,0.4)
            }
            selected {
                color = color::Lcha::new(1.0,0.0,0.0,0.6)
            }
            deselected{
                left {
                    color = color::Lcha::new(1.0,0.0,0.0,0.6)
                }
                right {
                    color = color::Lcha::new(1.0,0.0,0.0,0.2)
                }
            }
            hover {
                color = color::Lcha::new(1.0,0.0,0.0,0.6)
            }
        }
        list_view {
            background {
                color = color::Lcha::new(0.2,0.013,0.18,1.0)
            }
            highlight {
                color = color::Lcha::new(0.72,0.5,0.22,1.0)
            }
        }
        edge {
            split_color {
                lightness_factor = 0.2;
                chroma_factor = 1.0
            }
        }
        type {
            missing {
                color = color::Lcha::new(0.5,0.0,0.0,1.0)
            }
            color {
                luminance = 0.5;
                chroma = 0.8
            }
        }
    }

    let mut dark = theme::Theme::new();
    dark.insert("application.background.color", color::Lcha::new(0.13,0.013,0.18,1.0));
    dark.insert("application.text.color", color::Lcha::new(1.0,0.0,0.0,0.7));
    dark.insert("text.selection.color", color::Lcha::new(0.7,0.0,0.125,0.7));

    dark.insert("graph_editor.node.background.color", color::Lcha::new(0.2,0.013,0.18,1.0));
    dark.insert("graph_editor.node.selection.color", color::Lcha::new(0.72,0.5,0.22,1.0));
    dark.insert("graph_editor.node.selection.size", 7.0);
    dark.insert("graph_editor.visualization.background.color", color::Lcha::new(0.2,0.013,0.18,1.0));

    dark.insert("breadcrumbs.full.color", color::Lcha::new(1.0,0.0,0.0,0.7));
    dark.insert("breadcrumbs.transparent.color", color::Lcha::new(1.0,0.0,0.0,0.4));
    dark.insert("breadcrumbs.selected.color", color::Lcha::new(1.0,0.0,0.0,0.6));
    dark.insert("breadcrumbs.left.deselected.color", color::Lcha::new(1.0,0.0,0.0,0.6));
    dark.insert("breadcrumbs.right.deselected.color", color::Lcha::new(1.0,0.0,0.0,0.2));
    dark.insert("breadcrumbs.hover.color", color::Lcha::new(1.0,0.0,0.0,0.6));

    dark.insert("list_view.background.color", color::Lcha::new(0.2,0.013,0.18,1.0));
    dark.insert("list_view.highlight.color", color::Lcha::new(0.72,0.5,0.22,1.0));

    dark.insert("edge.split_color_lightness_factor", 0.2);
    dark.insert("edge.split_color_chroma_factor", 1.0);

    dark.insert("type.missing.color", color::Lcha::new(0.5,0.0,0.0,1.0));
    dark.insert("type.color_luminance", 0.5);
    dark.insert("type.color_chroma", 0.8);

    app.themes.register("dark",dark);


    let mut light = theme::Theme::new();
    light.insert("application.background.color", color::Lcha::new(0.96,0.013,0.18,1.0));
    light.insert("application.text.color", color::Lcha::new(0.0,0.0,0.0,0.7));
    light.insert("text.selection.color", color::Lcha::new(0.7,0.0,0.125,0.7));

    light.insert("graph_editor.node.background.color", color::Lcha::new(0.98,0.013,0.18,1.0));
    light.insert("graph_editor.node.selection.color", color::Lcha::new(0.83,0.58,0.436,1.0));
    light.insert("graph_editor.node.selection.size", 7.0);
    light.insert("graph_editor.visualization.background.color", color::Lcha::new(0.98,0.013,0.18,1.0));

    light.insert("breadcrumbs.full.color", color::Lcha::new(0.0,0.0,0.0,0.7));
    light.insert("breadcrumbs.transparent.color", color::Lcha::new(0.0,0.0,0.0,0.4));
    light.insert("breadcrumbs.selected.color", color::Lcha::new(0.0,0.0,0.0,0.6));
    light.insert("breadcrumbs.left.deselected.color", color::Lcha::new(0.0,0.0,0.0,0.6));
    light.insert("breadcrumbs.right.deselected.color", color::Lcha::new(0.0,0.0,0.0,0.2));
    light.insert("breadcrumbs.hover.color", color::Lcha::new(0.0,0.0,0.0,0.6));

    light.insert("list_view.background.color", color::Lcha::new(0.98,0.013,0.18,1.0));
    light.insert("list_view.highlight.color", color::Lcha::new(0.55,0.65,0.79,1.0));

    light.insert("edge.split_color_lightness_factor", 1.2);
    light.insert("edge.split_color_chroma_factor", 0.8);

    light.insert("type.missing.color", color::Lcha::new(0.8,0.0,0.0,1.0));
    light.insert("type.color_luminance", 0.8);
    light.insert("type.color_chroma", 0.6);

    app.themes.register("light",light);
    app.themes.set_enabled(&["light"]);
}