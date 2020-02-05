export function bind_keyboard_events(copy_handler, paste_handler, key_down_handler, key_up_handler) {
    var ta = document.createElement('textarea')
    ta.className = "makepad"
    ta.setAttribute('autocomplete', 'off')
    ta.setAttribute('autocorrect', 'off')
    ta.setAttribute('autocapitalize', 'off')
    ta.setAttribute('spellcheck', 'false')
    var style = document.createElement('style')
    style.innerHTML = "\n"
        + "textarea.makepad {\n"
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
        + "textarea: focus.makepad {\n"
        + "outline: 0px !important;\n"
        + "-webkit-appearance: none;\n"
        + "}"
    document.body.appendChild(style)
    ta.style.left = -100 + 'px'
    ta.style.top = -100 + 'px'
    ta.style.height = 1
    ta.style.width = 1

    ta.addEventListener('cut', e => {
        setTimeout(_ => {ta.value = "";}, 0)
    })
    ta.addEventListener('copy', e => {
        setTimeout(_ => {ta.value = "";}, 0)
    })
    ta.addEventListener('paste', e => {
        let paste = (event.clipboardData || window.clipboardData).getData('text')
        paste_handler(paste)
    })
    ta.addEventListener('contextmenu', e => {
        e.preventDefault()
    });
    ta.addEventListener('blur', e => {
        ta.focus();
    })
    ta.addEventListener('keydown', e => {
        let code = e.keyCode;

        if ((code === 88 || code == 67) && (e.metaKey || e.ctrlKey)) { // copy or cut
            ta.value = copy_handler();
            ta.selectionStart = 0;
            ta.selectionEnd = ta.value.length;
        } else if (!(code === 86 && (e.metaKey || e.ctrlKey))) { // not paste
            e.preventDefault()
        }
        key_down_handler(e)
    })
    ta.addEventListener('keyup', e => {
        e.preventDefault()
        key_up_handler(e)
    })
    document.body.appendChild(ta);
    ta.focus();
}