//! Mouse FRP bindings.

use crate::prelude::*;

use crate as frp;
use crate::data::bitfield::BitField;
use crate::data::bitfield::BitField32;
use nalgebra::Vector2;



// ==============
// === Button ===
// ==============

/// An enumeration representing the mouse buttons. Please note that we do not name the buttons
/// left, right, and middle, as this assumes we use a mouse for right-hand people.
///
/// JS supports up to 5 mouse buttons currently:
/// https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button
/// https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/buttons
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[allow(missing_docs)]
pub enum Button {Button0,Button1,Button2,Button3,Button4}
pub use Button::*;

#[allow(non_upper_case_globals,missing_docs)]
mod button_aliases {
    use super::*;
    pub const PrimaryButton   : Button = Button0;
    pub const MiddleButton    : Button = Button1;
    pub const SecondaryButton : Button = Button2;
}
pub use button_aliases::*;

impl Button {
    /// Construct a button from a code point.
    pub fn from_code(code:i32) -> Option<Self> {
        match code {
            0 => Some(Self::Button0),
            1 => Some(Self::Button1),
            2 => Some(Self::Button2),
            3 => Some(Self::Button3),
            4 => Some(Self::Button4),
            _ => None,
        }
    }

    /// The code point of the button.
    pub fn code(self) -> usize {
        match self {
            Button0 => 0,
            Button1 => 1,
            Button2 => 2,
            Button3 => 3,
            Button4 => 4,
        }
    }
}



// ==================
// === ButtonMask ===
// ==================

/// The button bitmask (each bit represents one button). Used for matching button combinations.
#[derive(Clone,Copy,Debug,Default,Eq,Hash,PartialEq,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct ButtonMask(pub BitField32);

impl<'a> FromIterator<&'a Button> for ButtonMask {
    fn from_iter<T: IntoIterator<Item=&'a Button>>(buttons:T) -> Self {
        let mut mask = ButtonMask::default();
        for button in buttons {
            let bit = button.code();
            mask.set_bit(bit,true);
        }
        mask
    }
}

impl From<&[Button]>   for ButtonMask { fn from(t:&[Button])   -> Self { ButtonMask::from_iter(t) } }
impl From<&[Button;0]> for ButtonMask { fn from(t:&[Button;0]) -> Self { ButtonMask::from_iter(t) } }
impl From<&[Button;1]> for ButtonMask { fn from(t:&[Button;1]) -> Self { ButtonMask::from_iter(t) } }
impl From<&[Button;2]> for ButtonMask { fn from(t:&[Button;2]) -> Self { ButtonMask::from_iter(t) } }
impl From<&[Button;3]> for ButtonMask { fn from(t:&[Button;3]) -> Self { ButtonMask::from_iter(t) } }
impl From<&[Button;4]> for ButtonMask { fn from(t:&[Button;4]) -> Self { ButtonMask::from_iter(t) } }
impl From<&[Button;5]> for ButtonMask { fn from(t:&[Button;5]) -> Self { ButtonMask::from_iter(t) } }



// =============
// === Mouse ===
// =============

/// Mouse FRP bindings.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Mouse {
    pub network       : frp::Network,
    pub up            : frp::Source,
    pub down          : frp::Source,
    pub wheel         : frp::Source,
    pub is_down       : frp::Stream<bool>,
    pub is_up         : frp::Stream<bool>,
    pub position      : frp::Source<Vector2<f32>>,
    pub prev_position : frp::Stream<Vector2<f32>>,
    pub translation   : frp::Stream<Vector2<f32>>,
    pub distance      : frp::Stream<f32>,
    pub ever_moved    : frp::Stream<bool>,
//    pub button_mask   : frp::Stram<ButtonMask>,
}

impl Default for Mouse {
    fn default() -> Self {
        frp::new_network! { network
            up            <- source_();
            down          <- source_();
            wheel         <- source_();
            position      <- source();
            is_down       <- bool(&up,&down);
            is_up         <- is_down.map(|t|!t);
            prev_position <- position.previous();
            translation   <- position.map2(&prev_position,|t,s|t-s);
            distance      <- translation.map(|t:&Vector2<f32>|t.norm());
            ever_moved    <- position.constant(true);
        };
        Self {network,up,down,wheel,is_down,is_up,position,prev_position,translation,distance
             ,ever_moved}
    }
}

impl Mouse {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }
}
