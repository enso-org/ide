// ==================
// === HTML Utils ===
// ==================

export function remove_node(node) {
    if (node) {
        node.parentNode.removeChild(node)
    }
}

export function new_top_level_div() {
    let node = document.createElement('div')
    node.style.width  = '100%'
    node.style.height = '100%'
    document.body.appendChild(node)
    return node
}

export async function log_group_collapsed(msg,f) {
    console.groupCollapsed(msg)
    let out
    try {
        out = await f()
    } catch (error) {
        console.groupEnd()
        throw error
    }
    console.groupEnd()
    return out
}
