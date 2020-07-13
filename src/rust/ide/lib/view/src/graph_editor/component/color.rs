

use crate::prelude::*;

use crate::graph_editor::SharedHashMap;
use crate::graph_editor::Type;

use ensogl::data::color;

#[derive(Clone,Copy,Debug)]
pub struct ShapeInformation {
    pub color: color::Lcha,
}

struct ShapeInformationMap {
    data: SharedHashMap<ImString,ShapeInformation>
}

impl ShapeInformationMap {
    pub fn shape_information_for_type(&self, r#type:Type) -> ShapeInformation {
                let type_key = r#type.0.clone();
                let default_fn = move || Self::generate_shape_information(&r#type.0);
                *self.data.raw.borrow_mut().entry(type_key).or_insert_with(default_fn)
    }

    fn generate_shape_information(_type_str:&ImString) -> ShapeInformation {
        let color = color::Lcha::blue_green(0.5, 0.5);
        ShapeInformation{color}
    }


}

pub fn shape_information_for_type(_type:Type) -> ShapeInformation {
    let color = color::Lch::blue_green(0.5, 0.5);
    let color = color.into();
    ShapeInformation{color}
}

fn default_shape_information() -> ShapeInformation {
    let color = color::Lcha::red(0.5,0.5);
    let color = color.into();
    ShapeInformation{color}
}

#[derive(Clone,CloneRef,Debug,Default,Shrinkwrap)]
pub struct TypeMap {
    data: SharedHashMap<ast::Id,Type>,
}

impl TypeMap {

    fn try_get_type(&self, ast_id:Option<ast::Id>) -> Option<Type> {
        let ast_id = ast_id?;
        self.data.get_cloned(&ast_id)
    }
    pub fn type_color(&self, ast_id:Option<ast::Id>) -> color::Lcha {
        match self.try_get_type(ast_id) {
            Some(type_information) => shape_information_for_type(type_information).color,
            None                   => default_shape_information().color
        }
    }
}

