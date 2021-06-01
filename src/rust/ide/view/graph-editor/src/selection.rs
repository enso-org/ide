//! Module that contains the logic sor selecting nodes. This includes selecting single nodes
//! by clicking on them separately, as well as click+drag for selecting with a selection area.
use ensogl::prelude::*;

use crate::NodeId;
use crate::Nodes;
use crate::TouchState;

use ensogl::frp;
use ensogl::gui::cursor;
use ensogl::gui::cursor::Cursor;



// ============
// === Mode ===
// ============

/// Possible selection modes.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum Mode {
    /// Select a single node when clicking on it. Deselects all other nodes.
    Normal,
    /// Toggle the selection state of the select node without changing the selection state of
    ///other node. This allows to add and remove nodes from the current selection.
    Multi,
    /// Add selected nodes to the set of currently selected nodes.
    Merge,
    /// Remove selected nodes from the set of currently selected nodes.
    Subtract,
    /// Invert the selection state of the selected nodes.
    Inverse
}

impl Mode {
    fn single_should_select(self, was_selected:bool) -> bool {
        match self {
            Self::Normal  => true,
            Self::Merge   => true,
            Self::Multi   => !was_selected,
            Self::Inverse => !was_selected,
            _             => false
        }
    }

    fn single_should_deselect(self, was_selected:bool) -> bool {
        match self {
            Self::Subtract => true,
            Self::Multi    => was_selected,
            Self::Inverse  => was_selected,
            _              => false
        }
    }

    fn multi_should_select(self, was_selected:bool) -> bool {
        match self {
            Self::Normal  => true,
            Self::Merge   => true,
            Self::Multi   => true,
            Self::Inverse => !was_selected,
            _             => false
        }
    }

    fn multi_should_deselect(self, was_selected:bool) -> bool {
        match self {
            Self::Subtract => true,
            Self::Multi    => was_selected,
            Self::Inverse  => was_selected,
            _              => false
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}



// ===================
// === BoundingBox ===
// ===================

/// Describes a 2D bounding box of an UI component.
#[derive(Clone,Copy,Default,Debug)]
pub struct BoundingBox {
    top    : f32,
    bottom : f32,
    left   : f32,
    right  : f32,
}

impl BoundingBox {
    pub fn from_corners(p1:Vector2, p2:Vector2) -> Self {
        let top    = p1.y.max(p2.y);
        let bottom = p1.y.min(p2.y);
        let left   = p1.x.min(p2.x);
        let right  = p1.x.max(p2.x);
        BoundingBox{top,bottom,left,right}
    }

    pub fn from_position_size(position:Vector2, size:Vector2) -> Self {
        Self::from_corners(position,position+size)
    }

    pub fn contains(&self, pos:Vector2) -> bool {
        self.contains_x(pos.x) && self.contains_y(pos.y)
    }

    fn contains_x(&self, x:f32) -> bool {
        x > self.left && x < self.right
    }

    fn contains_y(&self, y:f32) -> bool {
        y > self.bottom && y < self.top
    }

    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.top - self.bottom
    }

    pub fn intersects(&self, other:&BoundingBox) -> bool {
        // https://stackoverflow.com/a/13390495
        // self.right<other.left or other.right<self.left or self.bottom<other.top or other.bottom<self.top)
        let not_contained = (self.right < other.left)
                         || (other.right < self.left)
                         || (self.bottom > other.top)
                         || (other.bottom>self.top);
        !not_contained
    }
}


// === Bounding Box Tests ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection() {
        let bb1 = BoundingBox::from_corners(Vector2::new(0.5,0.5),Vector2::new(1.0,1.0));
        let bb2 = BoundingBox::from_corners(Vector2::new(0.0,0.0),Vector2::new(2.0,2.0));
        assert!(bb1.intersects(&bb2));
        assert!(bb2.intersects(&bb1));

        let bb1 = BoundingBox::from_corners(Vector2::new(3.0,3.0),Vector2::new(4.0,4.0));
        let bb2 = BoundingBox::from_corners(Vector2::new(0.0,0.0),Vector2::new(2.0,2.0));
        assert!(!bb1.intersects(&bb2));
        assert!(!bb2.intersects(&bb1));

        let bb1 = BoundingBox::from_corners(Vector2::new(0.0,0.0),Vector2::new(4.0,4.0));
        let bb2 = BoundingBox::from_corners(Vector2::new(0.0,0.0),Vector2::new(-2.0,-2.0));
        assert!(bb1.intersects(&bb2));
        assert!(bb2.intersects(&bb1));

        let bb1 = BoundingBox::from_corners(Vector2::new(0.0,0.0),Vector2::new(4.0,4.0));
        let bb2 = BoundingBox::from_corners(Vector2::new(2.0,2.0),Vector2::new(200.0,200.0));
        assert!(bb1.intersects(&bb2));
        assert!(bb2.intersects(&bb1));

        let bb1 = BoundingBox::from_corners(Vector2::new(-50.0,-50.0),Vector2::new(25.0,25.0));
        let bb2 = BoundingBox::from_corners(Vector2::new(5.00,50.0),Vector2::new(100.0,100.0));
        assert!(!bb1.intersects(&bb2));
        assert!(!bb2.intersects(&bb1));
    }
}


// ==========================
// === TemporarySelection ===
// ==========================

/// Struct that stores the initial selection state of a node. Used to restore the selection state
/// after a temporary selection has been undone, for example when an area selection is shrunk to no
/// longer encompass the node.
///
/// This implements `Hash` and `Eq` based only on the ndoe identity to avoid overwriting the initial
/// selection state in a set of `TemporarySelection`s. This allows the first added element for a
/// node to be preserved.
#[derive(Clone,Copy,Default,Debug,Eq)]
pub struct TemporarySelection {
    node         : NodeId,
    was_selected : bool,
}

impl TemporarySelection {
    fn new(node:NodeId, was_selected:bool)  -> Self {
        Self{node,was_selected}
    }
}

impl PartialEq for TemporarySelection {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl Hash for TemporarySelection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state)
    }
}

// ===============
// === NodeSet ===
// ===============

/// Set of `TemporarySelection` items. Used to keep track of the nodes currently selected with an
/// ongoing area selection.
mod node_set {
    use ensogl::prelude::*;
    use ensogl::frp;
    use crate::selection::TemporarySelection;

    type SetItem  = TemporarySelection;

    ensogl::define_endpoints! {
        Input {
            insert(SetItem),
            remove(SetItem),
            /// Remove the nodes that are not contained in the given Vec.
            remove_difference_with_vec(Vec<SetItem>),
            /// Empties the set without emitting `removed` events.
            reset(),
        }

        Output {
            // Emitted when an element that has not been in the set, is added to the set.
            // It is NOT emitted if an element that was added was already part of the set.
            added(SetItem),
            // Emitted when an element that has been in the set, is removed from the set.
            // It is NOT emitted if an element that was removed was not part of the set.
            removed(SetItem),
       }
    }

    #[derive(Clone,CloneRef,Debug,Default)]
    struct Model {
        set: Rc<RefCell<HashSet<SetItem>>>
    }

    impl Model {
        fn insert(&self, value: SetItem) -> bool {
            self.set.borrow_mut().insert(value)
        }

        fn remove(&self, value: SetItem) -> bool {
            self.set.borrow_mut().remove(&value)
        }

        fn difference(&self, other: &HashSet<SetItem>) -> Vec<SetItem> {
            self.set.borrow_mut().difference(other).cloned().collect()
        }

        fn reset(&self) {
            self.set.borrow_mut().clear()
        }
    }

    #[derive(Clone,CloneRef,Debug)]
    pub struct Set {
        frp: Rc<Frp>,
        model: Rc<Model>,
    }

    impl Set {
        pub fn new() -> Self {
            let frp   = Rc::new(Frp::new());
            let model = Rc::new(Model::default());
            Self { frp, model }.init()
        }

        fn init(self) -> Self {
            let network = &self.frp.network;
            let frp     = &self.frp;
            let set     = &self.model;
            frp::extend! { network
                eval_ frp.reset (set.reset());
                to_remove <- frp.remove_difference_with_vec.map(f!((values)
                    set.difference(&values.clone().into_iter().collect())));
                node_to_remvoe <= to_remove;
                frp.remove <+ node_to_remvoe;

                was_inserted     <- frp.insert.map(f!((value) set.insert(*value)));
                frp.source.added <+ frp.insert.gate(&was_inserted);

                was_removed        <- frp.remove.map(f!((value) set.remove(*value)));
                frp.source.removed <+ frp.remove.gate(&was_removed);

            }
            self
        }
    }

    impl Deref for Set {
        type Target = Frp;
        fn deref(&self) -> &Self::Target { &self.frp }
    }
}

fn get_nodes_in_bounding_box(bounding_box:&BoundingBox, nodes:&Nodes)  -> Vec<NodeId>{
    let nodes_raw = nodes.all.raw.as_ref().borrow();
    nodes_raw.iter().filter_map(|(id,node)|
        bounding_box.intersects(&node.view.frp.bounding_box.value()).as_some(*id)
    ).collect()

}

/// Return an FRP endpoint that indicates the current selection mode. This method sets up the logic
/// for deriving the selection mode from the global FRP inputs.
pub fn get_mode(network:&frp::Network,inputs:&crate::FrpEndpoints) -> frp::stream::Stream<Mode> {
    frp::extend! { network

    let multi_select_flag = crate::enable_disable_toggle
        ( network
        , &inputs.enable_node_multi_select
        , &inputs.disable_node_multi_select
        , &inputs.toggle_node_multi_select
        );

    let merge_select_flag = crate::enable_disable_toggle
        ( network
        , &inputs.enable_node_merge_select
        , &inputs.disable_node_merge_select
        , &inputs.toggle_node_merge_select
        );

    let subtract_select_flag = crate::enable_disable_toggle
        ( network
        , &inputs.enable_node_subtract_select
        , &inputs.disable_node_subtract_select
        , &inputs.toggle_node_subtract_select
        );

    let inverse_select_flag = crate::enable_disable_toggle
        ( network
        , &inputs.enable_node_inverse_select
        , &inputs.disable_node_inverse_select
        , &inputs.toggle_node_inverse_select
        );

    selection_mode <- all_with4
        (&multi_select_flag,&merge_select_flag,&subtract_select_flag,&inverse_select_flag,
        |multi,merge,subtract,inverse| {
            if      *multi    { Mode::Multi }
            else if *merge    { Mode::Merge }
            else if *subtract { Mode::Subtract }
            else if *inverse  { Mode::Inverse }
            else              { Mode::Normal }
        }
    );
    }
    selection_mode
}



// ==================
// === Controller ===
// ==================

/// Selection Controller that handles the logic for selecting and deselecting nodes in the graph
/// editor.
#[derive(Debug,Clone,CloneRef)]
pub struct Controller {
    network                : frp::Network,
    cursor_selection_nodes : node_set::Set,
    pub cursor_style       : frp::stream::Stream<cursor::Style>
}

impl Controller {
    pub fn new(inputs:&crate::FrpEndpoints,out:&crate::FrpEndpoints,cursor:&Cursor,
               mouse:&frp::io::Mouse,touch:&TouchState,nodes:&Nodes)
    -> Self {

        let network                = frp::Network::new("selection::Controller");
        let selection_mode         = get_mode(&network,inputs);
        let cursor_selection_nodes = node_set::Set::new();

        frp::extend! { network


            // ===  Selection Box & Mouse IO ===
            on_press_style   <- mouse.down_primary . constant(cursor::Style::new_press());
            on_release_style <- mouse.up_primary . constant(cursor::Style::default());

            edit_mode   <- bool(&inputs.edit_mode_off,&inputs.edit_mode_on);
            drag_start  <- mouse.down_primary.gate_not(&edit_mode);
            is_dragging <- bool(&mouse.up_primary,&drag_start);
            drag_end    <- is_dragging.on_false() ;
            drag_start  <- is_dragging.on_true() ;


            mouse_on_down_position <- mouse.position.sample(&mouse.down_primary);
            selection_size_down    <- mouse.position.map2(&mouse_on_down_position,|m,n|{m-n});
            selection_size         <- selection_size_down.gate(&touch.background.is_down);
            cursor_selection_start <- selection_size.map(|p|
                    cursor::Style::new_with_all_fields_default().press().box_selection(Vector2::new(p.x,p.y)));
            cursor_selection_end   <- mouse.up_primary . constant(cursor::Style::default());
            cursor_selection       <- any (cursor_selection_start, cursor_selection_end);

            cursor_on_down_position <- cursor.frp.scene_position.sample(&mouse.down_primary);
            should_update_drag      <- is_dragging && touch.background.is_down;
            cursor_drag_position    <- cursor.frp.scene_position.gate(&should_update_drag).on_change();

            scene_bounding_box      <- cursor_drag_position.map2(&cursor_on_down_position,|&m,&n|{
                // The dragged position is the center of the bounding box. Thus we need to offset the
                // corner point by the distance to the origin.
                let half      = m - n;
                let m_correct = n + 2.0 * half;
                BoundingBox::from_corners(n.xy(),m_correct.xy())}
            );

            nodes_in_bb <- scene_bounding_box.map(f!([nodes](bb) get_nodes_in_bounding_box(bb,&nodes)));
            nodes_in_bb <- nodes_in_bb.map(f!([nodes](nodes_selected) {
                nodes_selected.clone().into_iter().map(|node|{
                     let is_selected = nodes.is_selected(node);
                    TemporarySelection::new(node,is_selected)
                }).collect()
            }));
            node_info <= nodes_in_bb;


            // === Selection Box Handling ===

            keep_selection     <- selection_mode.map(|t| *t != Mode::Normal);
            deselect_on_select <- drag_start.gate_not(&keep_selection);
            eval_ deselect_on_select ( nodes.deselect_all() );

            cursor_selection_nodes.insert <+ node_info;
            cursor_selection_nodes.remove_difference_with_vec <+ nodes_in_bb;

            cursor_selection_nodes.reset <+ drag_end;

            // Node enters selection area, select depending on selection mode.
            node_added    <- cursor_selection_nodes.added.map(|node_info| node_info.node);
            should_select <- cursor_selection_nodes.added.map2(&selection_mode,
                |info,mode| mode.multi_should_select(info.was_selected)
            );
            should_deselect <- cursor_selection_nodes.added.map2(&selection_mode,
                |info,mode| mode.multi_should_deselect(info.was_selected)
            );

            out.source.node_selected   <+ node_added.gate(&should_select);
            out.source.node_deselected <+ node_added.gate(&should_deselect);

            // Node leaves selection area, revert to previous selection state.
            node_removed <- cursor_selection_nodes.removed.map(f!([](node_info) {
                if !node_info.was_selected { Some(node_info.node) } else {None}
            })).unwrap();

            out.source.node_deselected <+ node_removed;


            // ===  Single Node Selection Box & Mouse IO ===

            should_not_select       <- edit_mode || out.some_edge_endpoints_unset;
            node_to_select_non_edit <- touch.nodes.selected.gate_not(&should_not_select);
            node_to_select_edit     <- touch.nodes.down.gate(&edit_mode);
            node_to_select          <- any(node_to_select_non_edit,
                                           node_to_select_edit,
                                           inputs.select_node);
            node_was_selected       <- node_to_select.map(f!((id) nodes.selected.contains(id)));

            should_select <- node_to_select.map3(&selection_mode,&node_was_selected,
                |_,mode,was_selected| mode.single_should_select(*was_selected)
            );

            should_deselect <- node_to_select.map3(&selection_mode,&node_was_selected,
                |_,mode,was_selected| mode.single_should_deselect(*was_selected)
            );

            deselect_all_nodes      <- any_(...);
            deselect_on_select      <- node_to_select.gate_not(&keep_selection);
            deselect_all_nodes      <+ deselect_on_select;
            deselect_all_nodes      <+ inputs.deselect_all_nodes;

            deselect_on_bg_press    <- touch.background.selected.gate_not(&keep_selection);
            deselect_all_nodes      <+ deselect_on_bg_press;
            all_nodes_to_deselect   <= deselect_all_nodes.map(f_!(nodes.selected.mem_take()));
            out.source.node_deselected <+ all_nodes_to_deselect;

            node_selected           <- node_to_select.gate(&should_select);
            node_deselected         <- node_to_select.gate(&should_deselect);
            out.source.node_selected   <+ node_selected;
            out.source.node_deselected <+ node_deselected;

            // ===  Output bindings ===
            cursor_style <- any(on_press_style,on_release_style,cursor_selection);

        }

        Controller { network, cursor_selection_nodes, cursor_style }
    }
}
