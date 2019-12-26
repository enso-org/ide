use crate::prelude::*;

use wasm_bindgen::prelude::*;


#[wasm_bindgen(inline_js = "
import * as StatsPanel from 'stats.js';


export class Stats {
    constructor() {
        this.panels = []
        this.active = true
        this.dom    = document.createElement('div')
        this.dom.classList.add('statsjs')

        for (let id = 0; id < 3; id++) {
            let panel = new StatsPanel()
            panel.showPanel(id)
            this.dom.appendChild(panel.dom)
            panel.domElement.style.cssText = `position:absolute;top:0px;left:${id*80}px;`
            this.panels.push(panel)
        }
        this.hide()
    }

    show() {
        this.active = true
        this.dom.style.display = ''
    }

    hide() {
        this.active = false
        this.dom.style.display = 'none'
    }

    toggle() {
        if (this.active) { this.hide() } else { this.show() }
    }

    begin() {
        for (let panel of this.panels) {
            panel.begin()
        }
    }

    end() {
        for (let panel of this.panels) {
            panel.end()
        }
    }
}


export function new_stats() {
    let stats = new Stats()
    document.body.appendChild(stats.dom)
    stats.toggle()
    return stats
}"
)]
extern "C" {
    pub type Stats;

    pub fn new_stats() -> Stats;

    #[wasm_bindgen(structural, method)]
    pub fn show(this: &Stats);

    #[wasm_bindgen(structural, method)]
    pub fn hide(this: &Stats);

    #[wasm_bindgen(structural, method)]
    pub fn toggle(this: &Stats);

    #[wasm_bindgen(structural, method)]
    pub fn begin(this: &Stats);

    #[wasm_bindgen(structural, method)]
    pub fn end(this: &Stats);
}

impl Debug for Stats {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stats")
    }
}

impl Default for Stats {
    fn default() -> Self {
        new_stats()
    }
}