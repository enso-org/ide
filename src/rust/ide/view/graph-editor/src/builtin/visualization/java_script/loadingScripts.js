/**
 * Helper function to load scripts.
 *
 * It runs only once because of the visualization implementation : file is loaded once, then
 * the onDataReceived() is called multiple times, so it won't load the same script on this or any
 * other visualization show/hide.
 */
function loadScript(url) {
    let script = document.createElement("script");
    script.src = url;

    document.head.appendChild(script);
}

/**
 * Helper function to load styles.
 *
 * It runs only once because of the visualization implementation : file is loaded once, then
 * the onDataReceived() is called multiple times, so it won't load the same style on this or any
 * other visualization show/hide.
 */
function loadStyle(url) {
    let style   = document.createElement("link");
    style.href  = url;
    style.rel   = "stylesheet";
    style.media = "screen";
    style.type  = "text/css";

    document.head.appendChild(style);
}
