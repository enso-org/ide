//! Application theme setup.
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::style::theme;

/// `define_theme` helper.
macro_rules! _define_theme_literals {
    ([$theme_name:ident $($path:ident)*] $name:ident = $e:expr) => {
        $theme_name.insert(format!("{}{}",stringify!($($path.)*).replace(" ", ""),stringify!($name)).as_str(), $e);
    };

    ([$($path:ident)*] $name:ident = $e:expr; $($rest:tt)*) => {
        _define_theme_literals!([$($path)*] $name = $e);
        _define_theme_literals!([$($path)*] $($rest)*);
    };

    ([$($path:ident)*] $name:ident {$($t:tt)*}) => {
        _define_theme_literals!([$($path)* $name] $($t)*);
    };

    ([$($path:ident)*] $name:ident {$($t:tt)*} $($rest:tt)*) => {
        _define_theme_literals!([$($path)*] $name {$($t)*});
        _define_theme_literals!([$($path)*] $($rest)*);
    };
}

macro_rules! _define_theme_modules {
    ([$theme_name:ident $($path:ident)*] $name:ident = $e:expr) => {
        const $name : &str = format!("{}{}",stringify!($($path.)*).replace(" ", ""),stringify!($name)).as_str();
    };

    ([$($path:ident)*] $name:ident = $e:expr; $($rest:tt)*) => {
        _define_theme_modules!([$($path)*] $name = $e);
        _define_theme_modules!([$($path)*] $($rest)*);
    };

    ([$($path:ident)*] $name:ident {$($t:tt)*}) => {
        pub mod $name {
            _define_theme_modules!([$($path)* $name] $($t)*);
        }
    };

    ([$($path:ident)*] $name:ident {$($t:tt)*} $($rest:tt)*) => {
        _define_theme_modules!([$($path)*] $name {$($t)*});
        _define_theme_modules!([$($path)*] $($rest)*);
    };
}

/// Used to define theme.
#[macro_export]
macro_rules! define_theme {
    (($app:ident , $name:ident) $($t:tt)*) => {
        let mut $name = theme::Theme::new();
        _define_theme_literals!([$name] $($t)*);
        $app.themes.register(stringify!($name),$name);

        if cfg!(not(_Theme_Vars_)) {
            #[cfg(_Theme_Vars_)]
            pub mod Vars {
                _define_theme_modules!([$name] $($t)*);
            }
        }
    };
}

/// Used to set up themes for the application.
pub fn setup(app:&Application) {

    define_theme! { (app,dark)
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
        _type {
            missing {
                color = color::Lcha::new(0.5,0.0,0.0,1.0)
            }
            color {
                luminance = 0.5;
                chroma = 0.8
            }
        }
    }

    define_theme! { (app,light)
        application {
            background {
                color = color::Lcha::new(0.96,0.013,0.18,1.0)
            }
            text {
                color = color::Lcha::new(0.0,0.0,0.0,0.7);
                selection {
                    color = color::Lcha::new(0.7,0.0,0.125,0.7)
                }
            }
        }
        graph_editor {
            node {
                background {
                    color = color::Lcha::new(0.98,0.013,0.18,1.0)
                }
                selection {
                    color = color::Lcha::new(0.83,0.58,0.436,1.0);
                    size = 7.0
                }
            }
            visualization {
                background {
                    color = color::Lcha::new(0.98,0.013,0.18,1.0)
                }
            }
        }
        breadcrumbs {
            full {
                color = color::Lcha::new(0.0,0.0,0.0,0.7)
            }
            transparent {
                color = color::Lcha::new(0.0,0.0,0.0,0.4)
            }
            selected {
                color = color::Lcha::new(0.0,0.0,0.0,0.6)
            }
            deselected{
                left {
                    color = color::Lcha::new(0.0,0.0,0.0,0.6)
                }
                right {
                    color = color::Lcha::new(0.0,0.0,0.0,0.2)
                }
            }
            hover {
                color = color::Lcha::new(0.0,0.0,0.0,0.6)
            }
        }
        list_view {
            background {
                color = color::Lcha::new(0.98,0.013,0.18,1.0)
            }
            highlight {
                color = color::Lcha::new(0.55,0.65,0.79,1.0)
            }
        }
        edge {
            split_color {
                lightness_factor = 1.2;
                chroma_factor = 0.8
            }
        }
        _type {
            missing {
                color = color::Lcha::new(0.8,0.0,0.0,1.0)
            }
            color {
                luminance = 0.8;
                chroma = 0.6
            }
        }
    }

    app.themes.set_enabled(&["light"]);
}