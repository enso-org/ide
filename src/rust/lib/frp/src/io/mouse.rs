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
    /// Construct a button from the provided code point. In case the code is unrecognized, `None`
    /// will be returned.
    pub fn try_from_code(code:i32) -> Option<Self> {
        match code {
            0 => Some(Self::Button0),
            1 => Some(Self::Button1),
            2 => Some(Self::Button2),
            3 => Some(Self::Button3),
            4 => Some(Self::Button4),
            _ => None,
        }
    }

    /// Construct a button from the provided code point. In case the code is unrecognized, the
    /// default button will be returned.
    pub fn from_code(code:i32) -> Self {
        Self::try_from_code(code).unwrap_or_default()
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

impl Default for Button {
    fn default() -> Self {
        Self::Button0
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
    pub network              : frp::Network,
    pub up                   : frp::Source<Button>,
    pub down                 : frp::Source<Button>,
    pub wheel                : frp::Source,
    pub up_button_0          : frp::Stream,
    pub up_button_1          : frp::Stream,
    pub up_button_2          : frp::Stream,
    pub up_button_3          : frp::Stream,
    pub up_button_4          : frp::Stream,
    pub down_button_0        : frp::Stream,
    pub down_button_1        : frp::Stream,
    pub down_button_2        : frp::Stream,
    pub down_button_3        : frp::Stream,
    pub down_button_4        : frp::Stream,
    pub is_down_button_0     : frp::Stream<bool>,
    pub is_down_button_1     : frp::Stream<bool>,
    pub is_down_button_2     : frp::Stream<bool>,
    pub is_down_button_3     : frp::Stream<bool>,
    pub is_down_button_4     : frp::Stream<bool>,
    pub is_up_button_0       : frp::Stream<bool>,
    pub is_up_button_1       : frp::Stream<bool>,
    pub is_up_button_2       : frp::Stream<bool>,
    pub is_up_button_3       : frp::Stream<bool>,
    pub is_up_button_4       : frp::Stream<bool>,
    pub position             : frp::Source<Vector2<f32>>,
    pub prev_position        : frp::Stream<Vector2<f32>>,
    pub translation          : frp::Stream<Vector2<f32>>,
    pub distance             : frp::Stream<f32>,
    pub ever_moved           : frp::Stream<bool>,
    // pub button_mask          : frp::Stream<ButtonMask>,
    // pub previous_button_mask : frp::Stream<ButtonMask>,
}

impl Default for Mouse {
    fn default() -> Self {
        frp::new_network! { network
            up            <- source();
            down          <- source();
            wheel         <- source();
            position      <- source();
            is_down       <- bool(&up,&down);
            is_up         <- is_down.map(|t|!t);
            prev_position <- position.previous();
            translation   <- position.map2(&prev_position,|t,s|t-s);
            distance      <- translation.map(|t:&Vector2<f32>|t.norm());
            ever_moved    <- position.constant(true);

            up_button_0_check <- up.map(|t|*t==Button0);
            up_button_1_check <- up.map(|t|*t==Button1);
            up_button_2_check <- up.map(|t|*t==Button2);
            up_button_3_check <- up.map(|t|*t==Button3);
            up_button_4_check <- up.map(|t|*t==Button4);

            down_button_0_check <- down.map(|t|*t==Button0);
            down_button_1_check <- down.map(|t|*t==Button1);
            down_button_2_check <- down.map(|t|*t==Button2);
            down_button_3_check <- down.map(|t|*t==Button3);
            down_button_4_check <- down.map(|t|*t==Button4);

            up_button_0 <- up.gate(&up_button_0_check).constant(());
            up_button_1 <- up.gate(&up_button_1_check).constant(());
            up_button_2 <- up.gate(&up_button_2_check).constant(());
            up_button_3 <- up.gate(&up_button_3_check).constant(());
            up_button_4 <- up.gate(&up_button_4_check).constant(());

            down_button_0 <- down.gate(&down_button_0_check).constant(());
            down_button_1 <- down.gate(&down_button_1_check).constant(());
            down_button_2 <- down.gate(&down_button_2_check).constant(());
            down_button_3 <- down.gate(&down_button_3_check).constant(());
            down_button_4 <- down.gate(&down_button_4_check).constant(());

            is_down_button_0 <- bool(&up_button_0,&down_button_0);
            is_down_button_1 <- bool(&up_button_1,&down_button_1);
            is_down_button_2 <- bool(&up_button_2,&down_button_2);
            is_down_button_3 <- bool(&up_button_3,&down_button_3);
            is_down_button_4 <- bool(&up_button_4,&down_button_4);

            is_up_button_0 <- is_down_button_0.map(|t|!t);
            is_up_button_1 <- is_down_button_1.map(|t|!t);
            is_up_button_2 <- is_down_button_2.map(|t|!t);
            is_up_button_3 <- is_down_button_3.map(|t|!t);
            is_up_button_4 <- is_down_button_4.map(|t|!t);

        };
        Self {network,up,down,wheel,up_button_0,up_button_1,up_button_2,up_button_3,up_button_4
             ,down_button_0,down_button_1,down_button_2,down_button_3,down_button_4
             ,is_down_button_0,is_down_button_1,is_down_button_2,is_down_button_3,is_down_button_4
             ,is_up_button_0,is_up_button_1,is_up_button_2,is_up_button_3,is_up_button_4
             ,position,prev_position,translation,distance,ever_moved}
    }
}

impl Mouse {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }
}
