//! This module defines a structure gathering statistics of the running engine. The statistics are
//! an amazing tool for debugging what is really happening under the hood and understanding the
//! performance characteristics.

use crate::prelude::*;



// =============
// === Stats ===
// =============

/// Structure containing all the gathered stats.
#[derive(Debug,Clone)]
pub struct Stats {
    rc: Rc<RefCell<StatsData>>
}

impl Default for Stats {
    fn default() -> Self {
        let rc = Rc::new(RefCell::new(default()));
        Self {rc}
    }
}

impl Stats {

    // === SpriteSystem ===

    /// Gets `SpriteSystem` count.
    pub fn sprite_system_count(&self) -> usize {
        self.rc.borrow().sprite_system_count
    }

    /// Sets `SpriteSystem` count.
    pub fn set_sprite_system_count(&self, value:usize) {
        self.rc.borrow_mut().sprite_system_count = value;
    }

    /// Modifies `SpriteSystem` count.
    pub fn mod_sprite_system_count<F:FnOnce(usize)->usize>(&self, f:F) {
        let value = self.sprite_system_count();
        let value = f(value);
        self.set_sprite_system_count(value);
    }

    /// Increase `SpriteSystem` count.
    pub fn inc_sprite_system_count(&self) {
        self.mod_sprite_system_count(|t| t+1);
    }

    /// Increase `SpriteSystem` count.
    pub fn dec_sprite_system_count(&self) {
        self.mod_sprite_system_count(|t| t+1);
    }


    // === Sprite ===

    /// Gets `Sprite` count.
    pub fn sprite_count(&self) -> usize {
        self.rc.borrow().sprite_count
    }

    /// Sets `Sprite` count.
    pub fn set_sprite_count(&self, value:usize) {
        self.rc.borrow_mut().sprite_count = value;
    }

    /// Modifies `Sprite` count.
    pub fn mod_sprite_count<F:FnOnce(usize)->usize>(&self, f:F) {
        let value = self.sprite_count();
        let value = f(value);
        self.set_sprite_count(value);
    }

    /// Increase `Sprite` count.
    pub fn inc_sprite_count(&self) {
        self.mod_sprite_count(|t| t+1);
    }

    /// Increase `Sprite` count.
    pub fn dec_sprite_count(&self) {
        self.mod_sprite_count(|t| t+1);
    }

}



// =================
// === StatsData ===
// =================

#[derive(Debug,Default,Clone,Copy)]
struct StatsData {
    sprite_system_count : usize,
    sprite_count        : usize,
}
