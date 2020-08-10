//! Definition of the TextList component.


use crate::prelude::*;

use enso_frp as frp;
use enso_frp;
use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::shape::text::glyph::system::GlyphSystem;
use ensogl::display;
use ensogl::gui::component::Animation;
use ensogl::gui::component::Tween;
use ensogl::gui::component;

use super::node;


/// Traits that need to be implemented for a struct that can be used in a `TextList`.
pub trait TextListItem = Debug + Clone + Display + PartialEq + 'static;



// =================
// === Constants ===
// =================

const LINE_HEIGHT  : f32 = 30.0;

const MAX_CHARACTERS_PER_LINE : usize = 25;

const TEXT_PADDING   : f32 = node::NODE_SHAPE_RADIUS;
const TEXT_FONT_SIZE : f32 = 8.0;

const MOUSE_OUT_DELAY_TIME_MS : f32 =  500.0;



// ========================
// === Background Shape ===
// ========================

/// Text list background shape definition. Covers the background of the whole list.
pub mod background {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let width         = Var::<Pixels>::from("input_size.x");
            let height        = Var::<Pixels>::from("input_size.y");
            let select_radius = node::NODE_SHAPE_RADIUS.px() ;

            let shape        = Rect((&width,&height)).corners_radius(&select_radius);
            let fill_color   = color::Rgba::from(color::Lcha::new(0.0,0.013,0.18,0.6));
            let shape_filled = shape.fill(fill_color);

            shape_filled.into()
        }
    }
}



// ================
// === TextItem ===
// ================

/// Text shape definition. Renders the actual text of an item.
/// TODO consider generalising with `node::label`, but we want a different font size here, which
/// seems non-trivial to parametrise.
pub mod text {
    use super::*;

    #[derive(Clone,CloneRef,Debug)]
    #[allow(missing_docs)]
    pub struct Shape {
        pub label : ensogl::display::shape::text::glyph::system::Line,
        pub obj   : display::object::Instance,

    }
    impl ensogl::display::shape::system::Shape for Shape {
        type System = ShapeSystem;
        fn sprites(&self) -> Vec<&Sprite> {
            vec![]
        }
    }
    impl display::Object for Shape {
        fn display_object(&self) -> &display::object::Instance {
            &self.obj
        }
    }
    #[derive(Clone,CloneRef,Debug)]
    #[allow(missing_docs)]
    pub struct ShapeSystem {
        pub glyph_system: GlyphSystem,
        style_manager: StyleWatch,

    }
    impl ShapeSystemInstance for ShapeSystem {
        type Shape = Shape;

        fn new(scene:&Scene) -> Self {
            let style_manager = StyleWatch::new(&scene.style_sheet);
            let font          = scene.fonts.get_or_load_embedded_font("DejaVuSansMono").unwrap();
            let glyph_system  = GlyphSystem::new(scene,font);
            let symbol        = &glyph_system.sprite_system().symbol;
            scene.views.main.remove(symbol);
            scene.views.label.add(symbol);
            Self {glyph_system,style_manager} // .init_refresh_on_style_change()
        }

        fn new_instance(&self) -> Self::Shape {
            let color = color::Rgba::new(1.0,1.0,1.0,0.7);
            let obj   = display::object::Instance::new(Logger::new("test"));
            let label = self.glyph_system.new_line();
            label.set_font_size(TEXT_FONT_SIZE);
            label.set_font_color(color);
            label.set_text("");
            obj.add_child(&label);
            Shape {label,obj}
        }
    }
}



// ============================
// === Text Item Background ===
// ============================

/// Text item background shape definition. Is invisible and covers the background of a single item.
/// Used to get mouse events for the item.
pub mod text_item_hover {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let width        = Var::<Pixels>::from("input_size.x");
            let height       = Var::<Pixels>::from("input_size.y");
            let shape        = Rect((&width,&height));
            let shape_filled = shape.fill(color::Rgba::new(1.0,0.0,0.0,0.000_001));

            shape_filled.into()
        }
    }
}



// ===========================
// === Text Item Highlight ===
// ===========================

/// Text item highlight shape definition. Used to indicate the currently active or hovered item.
pub mod text_item_highlight {
    use super::*;

    ensogl::define_shape_system! {
        (opacity:f32) {
            let width         = Var::<Pixels>::from("input_size.x");
            let height        = Var::<Pixels>::from("input_size.y");
            let corner_radius = node::NODE_SHAPE_RADIUS.px();
            let shape         = Rect((&width,&height)).corners_radius(&corner_radius);
            let color         = Var::<color::Rgba>::from("srgba(0.2,0.5,7.0,input_opacity)");
            let shape_filled = shape.fill(color);

            shape_filled.into()
        }
    }
}



// ===========
// === Frp ===
// ===========

/// FRP api of the `TextList`.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Frp<T:TextListItem> {
    pub set_content     : frp::Source<Vec<T>>,
    pub set_width       : frp::Source<f32>,
    pub set_preselected : frp::Source<Option<T>>,

    pub set_layout_collapsed : frp::Source,
    pub set_layout_expanded  : frp::Source,

    pub mouse_out : frp::Stream,
    pub selection : frp::Stream<Option<T>>,

    set_selection   : frp::Source<Option<T>>,
    on_item_hover   : frp::Source<Option<T>>,
    on_mouse_out    : frp::Source,
}

impl<T:TextListItem> Frp<T> {
    fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            set_content     <- source();
            set_selection   <- source();
            set_width       <- source();
            on_item_hover   <- source();
            set_preselected <- source();
            on_mouse_out    <- source();

            set_layout_collapsed <- source();
            set_layout_expanded  <- source();

            let selection = set_selection.clone_ref().into();
            let mouse_out = on_mouse_out.clone_ref().into();
        }
        Self{set_content,selection,set_selection,set_width,on_item_hover,set_preselected,
            set_layout_collapsed,set_layout_expanded,on_mouse_out,mouse_out}
    }
}



// =======================
// === Text List Model ===
// =======================

#[derive(Clone,Debug)]
struct TextListModel<T:TextListItem> {
    scene                    : Scene,
    logger                   : Logger,
    display_object           : display::object::Instance,
    content_views            : RefCell<Vec<component::ShapeView<text::Shape>>>,
    content_background_views : RefCell<Vec<component::ShapeView<text_item_hover::Shape>>>,
    background_shape         : component::ShapeView<background::Shape>,
    highlight_shape          : component::ShapeView<text_item_highlight::Shape>,
    content_items            : RefCell<Vec<T>>,
    item_network             : RefCell<frp::Network>,
    width                    : Cell<f32>,
}

impl<T:TextListItem> TextListModel<T> {
    fn new(scene:&Scene) -> Self {
        let logger                   = Logger::new("TextListModel");
        let display_object           = display::object::Instance::new(&logger);
        let background_shape         = component::ShapeView::new(&logger,scene);
        let highlight_shape          = component::ShapeView::new(&logger,scene);
        let scene                    = scene.clone();
        let content_items            = default();
        let content_background_views = default();
        let content_views            = default();
        let item_network             = default();
        let width                    = default();


        TextListModel{scene,display_object,logger,content_items,background_shape,content_views,
                      item_network,content_background_views,highlight_shape,width}.init()
    }

    fn init(self)-> Self {
        self.add_child(&self.background_shape);
        self.add_child(&self.highlight_shape);
        self
    }

    /// Create a textual representation for the given content item.
    fn format_item(item:&T) -> String {
        let formatted     = format!("{}",item);
        // We cut of strings that are to long and replace the end with "...".
        let max_output    = formatted.chars().take(MAX_CHARACTERS_PER_LINE);
        let dots_required =  max_output.clone().count() > (MAX_CHARACTERS_PER_LINE - 3);
        if dots_required {
            let shortened = formatted.chars().take( max_output.count() - 3);
            let with_dots = shortened.chain("...".chars());
            return with_dots.collect()
        }
        max_output.collect()
    }

    /// Update the content items and create the appropriate shapes.
    /// *Important*: to avoid memory leaks, this does not update the FRP bindings. To do that
    /// `init_item_frp` needs to be called.
    fn set_content(&self, content:&[T]) {
        let mut content_views            = Vec::with_capacity(content.len());
        let mut content_background_views = Vec::with_capacity(content.len());

        for (_index,item) in content.iter().enumerate() {

            let label            = component::ShapeView::<text::Shape>::new(&self.logger,&self.scene);
            label.shape.label.set_text(Self::format_item(item));

            let label_background = component::ShapeView::<text_item_hover::Shape>::new(&self.logger,&self.scene);
            label_background.shape.size.set(self.item_size());

            self.add_child(&label);
            self.add_child(&label_background);

            content_views.push(label);
            content_background_views.push(label_background);
        };

        *self.content_views.borrow_mut()            = content_views;
        *self.content_background_views.borrow_mut() = content_background_views;
        *self.content_items.borrow_mut()            = content.to_owned();

        let item_count      = self.content_items.borrow().len();
        self.set_background_to_cover_items(item_count);
        debug_assert_eq!(self.content_views.borrow().len()           , self.content_items.borrow().len());
        debug_assert_eq!(self.content_background_views.borrow().len(), self.content_items.borrow().len());
    }

    fn init_item_frp(&self, frp:&Frp<T>) {
        let item_network    = frp::Network::new();
        let content_views   = self.content_background_views.borrow();
        let items           = self.content_items.borrow();
        let views_and_items = content_views.iter().zip(items.iter());

        for (view,item) in views_and_items {
            frp::extend! { item_network
                let item_shared  = item.clone();
                eval_ view.events.mouse_down  (frp.set_selection.emit(Some(item_shared.clone())));
                let item_shared  = item.clone();
                eval_  view.events.mouse_over (frp.on_item_hover.emit(Some(item_shared.clone())));
                eval_  view.events.mouse_out  (frp.on_item_hover.emit(None));
            }
        }

        *self.item_network.borrow_mut() = item_network;
    }

    fn item_count(&self) -> usize {
        self.content_items.borrow().len()
    }

    fn set_width(&self, width:f32) {
        self.width.set(width);
        self.highlight_shape.shape.sprite.size.set(self.item_size());
    }

    /// Hide the highlight shape.
    fn deactivate_highlight(&self) {
        self.highlight_shape.set_position_xy(self.item_base_position(0.0));
        self.highlight_shape.shape.opacity.set(0.0);
    }

    /// Set the given item as selected. That means it moves to the top of the list.
    fn set_preselected_item(&self, item:Option<T>) {
        if let Some(item) = item {
            self.set_item_to_first_position(item);
        }
    }

    fn set_item_to_first_position(&self, item:T) {
        let index = self.content_items.borrow().iter().position(|other| *other == item);
        if let Some(index) = index {
            self.content_items.borrow_mut().swap(0,index);
            self.content_views.borrow_mut().swap(0,index);
            self.content_background_views.borrow_mut().swap(0,index);
        }
    }
}

impl<T:TextListItem> display::Object for TextListModel<T> {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}


// === Layout logic ===

impl<T:TextListItem> TextListModel<T> {

    /// Return the position of an item at the given index. Fractional indices are allowed for smooth
    /// transitions.
    fn item_base_position(&self, index:f32) -> Vector2<f32> {
        let item_height = LINE_HEIGHT;
        Vector2::new(self.width.get()/2.0,-item_height*(index + 0.5))
    }

    /// Return the position of the text that belongs to the item at the given index. Fractional
    /// indices are allowed for smooth transitions.
    fn text_base_position(&self, index:f32) -> Vector2<f32> {
        // Text baseline is different, so we correct this here.
        let text_offset_y = TEXT_FONT_SIZE / 2.0;
        Vector2::new(TEXT_PADDING,self.item_base_position(index).y - text_offset_y)
    }

    /// Return the extent of a single content item.
    fn item_size(&self) -> Vector2 {
        let line_height = LINE_HEIGHT;
        Vector2::new(self.width.get(),line_height)
    }

    /// Hide all items except for the given `item`.
    fn show_single_item(&self, item:Option<T>) {
        if let Some(item) = item {
            let content_views         = self.content_views.borrow();
            let content_hover_views   = self.content_background_views.borrow();
            let items                 = self.content_items.borrow();
            let views_with_items_iter = izip!(content_views.iter(),
                                              content_hover_views.iter(),
                                              items.iter());

            for (label,hover_view,content_item) in views_with_items_iter {
                if *content_item == item {
                    label.set_position_xy(self.text_base_position(0.0));
                    hover_view.set_position_xy(self.item_base_position(0.0));
                    self.add_child(&label);
                    self.add_child(&hover_view);
                } else {
                    label.unset_parent();
                    hover_view.unset_parent();
                }
            }
        }
    }

    /// Show the text list as an single collapsed item. The item shown is the currently
    /// selected item.
    fn set_layout_collapsed(&self) {
        // We always have the selected item at the first position.
        let first_item = self.content_items.borrow().get(0).cloned();
        self.show_single_item(first_item);
        self.set_background_to_cover_items(1);
        self.deactivate_highlight();
    }

    /// Show the text list as an expanded list of all available items.
    fn set_layout_expanded(&self) {
        let content_views         = self.content_views.borrow();
        let content_hover_views   = self.content_background_views.borrow();
        let views_with_items_iter = izip!(content_views.iter(),content_hover_views.iter());

        for (index, (label,hover_view)) in views_with_items_iter.enumerate() {
            self.display_object.add_child(&label);
            self.display_object.add_child(&hover_view);
            label.set_position_xy(self.text_base_position(index as f32));
            hover_view.set_position_xy(self.item_base_position(index as f32));
        };

        self.set_background_to_cover_items(self.item_count());
    }

    /// Set the background size to cover the given number of items.
    fn set_background_to_cover_items(&self, item_count:usize) {
        let base_size           = self.item_size();
        let background_size     = Vector2::new(base_size.x,base_size.y * item_count as f32);
        let background_position = Vector2::new(background_size.x/2.0,-background_size.y/2.0);
        self.background_shape.shape.size.set(background_size);
        self.background_shape.set_position_xy(background_position);
    }

}



// ================
// === TextList ===
// ================

/// A list of items that allows selection. Behaves like a dropdown menu.
///
/// The `TextList` can present a list of `TextListItem`s from which one can be selected.
/// It has two layout modes: (1) collapsed, where only the currently selected list item is visible
/// and (2) expanded, where all available items are visible and the currently hovered item is
/// highlighted.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct TextList<T:TextListItem> {
        model   : Rc<TextListModel<T>>,
        network : frp::Network,
    pub frp     : Frp<T>
}

impl<T:TextListItem> display::Object for TextList<T> {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object()
    }
}

impl<T:TextListItem> TextList<T> {
    /// Constructor for the `TextList`. The list is empty by default and needs to be populated
    /// via the frp api.
    pub fn new(scene:&Scene) -> Self {
        let model   = TextListModel::new(scene);
        let model   = Rc::new(model);
        let network = frp::Network::new();
        let frp     = Frp::new(&network);

        Self{model,network,frp}.init_frp()
    }

    fn init_frp(self) -> Self {
        let network = &self.network;
        let frp     = &self.frp;
        let model   = &self.model;

        let highlight_opacity  = Animation::<f32>::new(&network);
        let highlight_position = Animation::<f32>::new(&network);
        let highlight_shape    = &model.highlight_shape;

        // We use a delay tween to avoid emitting a mouse out event when we only switch from one
        // item to another.
        let mouse_out_timer = Tween::new(&network);
        mouse_out_timer.set_duration(MOUSE_OUT_DELAY_TIME_MS);

        // Arbitrary sentinel value to check for the end of the tween.
        const TWEEN_END_VALUE:f32 = 1.0;

        frp::extend! { network
            // External API
            eval frp.set_preselected ((item) model.set_preselected_item(item.clone()));
            eval frp.set_content ([frp,model](content) {
                model.set_content(content);
                model.init_item_frp(&frp);
                model.set_preselected_item(content.get(0).cloned());
                model.set_layout_collapsed();
            });
            eval_ frp.set_layout_collapsed ( model.set_layout_collapsed() );
            eval_ frp.set_layout_expanded ([highlight_opacity,highlight_position,model] {
                // We want to ensure highlight appearance is reset
                highlight_position.set_target_value(0.0);
                highlight_opacity.set_target_value(0.0);
                highlight_position.set_value(0.0);
                highlight_opacity.set_value(0.0);
                model.set_layout_expanded();
            });
            eval frp.set_preselected ((item) model.set_preselected_item(item.clone()));
            eval frp.set_width        ((size) model.set_width(*size));
             // Internal shape API
            eval frp.on_item_hover ([mouse_out_timer,highlight_opacity,highlight_position,model](item) {
                match item {
                    Some(item) => {
                        model.set_layout_expanded();
                        mouse_out_timer.stop();
                        let item_index = model.content_items.borrow().iter().position(|other| other == item);
                        if let Some(index) = item_index {
                            highlight_opacity.set_target_value(1.0);
                            highlight_position.set_target_value(index as f32);
                        }
                    },
                    None => {
                        mouse_out_timer.reset();
                        mouse_out_timer.set_target_value(TWEEN_END_VALUE);
                         highlight_opacity.set_target_value(0.0);
                    },
                }
            });
            eval_ frp.selection ( model.set_layout_collapsed() );
            // Animations
            eval highlight_opacity.value    ((value) {
                highlight_shape.shape.opacity.set(*value);
            });
            eval highlight_position.value ([highlight_shape,model](value) {
               highlight_shape.set_position_xy(model.item_base_position(*value))
            });
            // Mouse out tween
            mouse_out_timer_finished    <- mouse_out_timer.value.map(|t| *t>=TWEEN_END_VALUE );
            on_mouse_out_timer_finished <- mouse_out_timer_finished.gate(&mouse_out_timer_finished).constant(());
            eval_ on_mouse_out_timer_finished ( frp.on_mouse_out.emit(()));

        }
        self
    }

    // TODO: Implement proper sorting and remove.
    /// Hack function used to register the elements for the sorting purposes. To be removed.
    pub fn order_hack(scene:&Scene) {
        let logger = Logger::new("hack_sort");
        component::ShapeView::<background::Shape>::new(&logger,scene);
        component::ShapeView::<text_item_highlight::Shape>::new(&logger,scene);
        component::ShapeView::<text_item_hover::Shape>::new(&logger,scene);
    }
 }
