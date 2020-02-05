// Text input event handlers.
//
// The "text input event" is the normal keyup/keydown event, or cut/copy/paste event. The handling of these events is
// done by creating an invisible focused textarea html element and intercepting events emitted to this element.
// We do it this way, because this is so far the only way a website may affect the clipboard which works on each
// browser.
export class TextInputHandlers {
    // Constructor creates the textarea element and put it into document body.
    constructor() {
        this.event_handlers = {}
        this.text_area = create_invisible_text_area()
        document.body.appendChild(this.text_area)
        this.text_area.focus()
        this.bind_text_area_events()
    }

    // Set event handler. The name can be 'keyup' or 'keydown'.
    set_event_handler(name, callback) { this.event_handlers[name] = callback }
    // Set copy handler. The copy handler takes bool as first argument denoting if it is cut operation, and returns
    // string which should be actually copied to clipboard.
    set_copy_handler(handler)         { this.copy_handler          = handler }
    // Set paste handler. The paste handler takes the text from clipboard as the only argument.
    set_paste_handler(handler)        { this.paste_handler         = handler }

    // Remove the textarea element and stop handling any events.
    stop_handling() {
        this.text_area.remove()
    }

    // This is private function being a construction stage.
    bind_text_area_events() {
        var ta = this.text_area

        ta.addEventListener('cut', e => {
            setTimeout(_ => {ta.value = "";}, 0)
        })
        ta.addEventListener('copy', e => {
            setTimeout(_ => {ta.value = "";}, 0)
        })
        ta.addEventListener('paste', e => {
            if (typeof this.paste_handler !== 'undefined') {
                let paste = (event.clipboardData || window.clipboardData).getData('text')
                this.paste_handler(paste)
            } else {
                e.preventDefault()
            }
        })
        ta.addEventListener('contextmenu', e => {
            e.preventDefault()
        });
        ta.addEventListener('blur', e => {
            ta.focus()
        })
        ta.addEventListener('keydown', e => {
            let code = e.keyCode;

            if ((code === 88 || code == 67) && (e.metaKey || e.ctrlKey)) { // copy or cut
                if (typeof this.copy_handler !== 'undefined') {
                    ta.value = this.copy_handler(code === 88)
                    ta.selectionStart = 0;
                    ta.selectionEnd = ta.value.length;
                } else {
                    e.preventDefault()
                }
            } else if (!(code === 86 && (e.metaKey || e.ctrlKey))) { // not paste
                e.preventDefault()
            }

            if (typeof this.event_handlers['keydown'] !== 'undefined') {
                this.event_handlers['keydown'](e)
            }
        })
        ta.addEventListener('keyup', e => {
            e.preventDefault()
            if (typeof this.event_handlers['keyup'] !== 'undefined') {
                this.event_handlers['keyup'](e)
            }
        })
    }
}

// Creates invisible textarea.
function create_invisible_text_area() {
    const css_class_name = "enso";

    let ta = document.createElement('textarea')
    ta.className = css_class_name
    ta.setAttribute('autocomplete', 'off')
    ta.setAttribute('autocorrect', 'off')
    ta.setAttribute('autocapitalize', 'off')
    ta.setAttribute('spellcheck', 'false')
    let style = document.createElement('style')
    style.innerHTML = "\n"
        + "textarea." + css_class_name + " {\n"
        + "z-index: 100000;\n"
        + "position: absolute;\n"
        + "opacity: 0;\n"
        + "border-radius: 4px;\n"
        + "color:white;\n"
        + "font-size: 6;\n"
        + "background: gray;\n"
        + "-moz-appearance: none;\n"
        + "appearance:none;\n"
        + "border:none;\n"
        + "resize: none;\n"
        + "outline: none;\n"
        + "overflow: hidden;\n"
        + "text-indent: 0px;\n"
        + "padding: 0 0px;\n"
        + "margin: 0 -1px;\n"
        + "text-indent: 0px;\n"
        + "-ms-user-select: text;\n"
        + "-moz-user-select: text;\n"
        + "-webkit-user-select: text;\n"
        + "user-select: text;\n"
        + "white-space: pre!important;\n"
        + "}\n"
        + "textarea: focus." + css_class_name + " {\n"
        + "outline: 0px !important;\n"
        + "-webkit-appearance: none;\n"
        + "}"
    document.body.appendChild(style)
    ta.style.left = -100 + 'px'
    ta.style.top = -100 + 'px'
    ta.style.height = 1
    ta.style.width = 1
    return ta
}