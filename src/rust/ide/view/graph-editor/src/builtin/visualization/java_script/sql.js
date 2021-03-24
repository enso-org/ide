loadScript('https://cdnjs.cloudflare.com/ajax/libs/sql-formatter/4.0.2/sql-formatter.min.js')

/**
 * A visualization that pretty-prints generated SQL code and displays type hints related to
 * interpolated query parameters.
 */
class SqlVisualization extends Visualization {
    // TODO Change the type below once #837 is done:
    // 'Standard.Database.Data.Table.Table | Standard.Database.Data.Column.Column'
    static inputType = 'Any'
    static label = 'SQL Query'

    constructor(api) {
        super(api)
        this.setPreprocessorModule('Standard.Visualization.Sql.Visualization')
        this.setPreprocessorCode(`x -> here.prepare_visualization x`)
    }

    onDataReceived(data) {
        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild)
        }

        let parsedData = data
        if (typeof data === 'string') {
            parsedData = JSON.parse(data)
        }

        let visHtml = visualizationStyle
        if (parsedData.error !== undefined) {
            visHtml += parsedData.error
        } else {
            const params = parsedData.interpolations.map(param =>
                renderInterpolationParameter(this.theme, param)
            )

            let language = 'sql'
            if (parsedData.dialect == 'postgresql') {
                language = 'postgresql'
            }

            const formatted = sqlFormatter.format(parsedData.code, {
                params: params,
                language: language,
            })

            const codeRepresentation = '<pre class="sql">' + formatted + '</pre>'
            visHtml += codeRepresentation
        }

        const containers = this.createContainers()
        const parentContainer = containers[0]
        const scrollable = containers[1]
        scrollable.innerHTML = visHtml
        this.dom.appendChild(parentContainer)

        const tooltip = new Tooltip(parentContainer)
        const baseMismatches = this.dom.getElementsByClassName('mismatch')
        const extendedMismatchAreas = this.dom.getElementsByClassName('mismatch-mouse-area')
        setupMouseInteractionForMismatches(tooltip, baseMismatches)
        setupMouseInteractionForMismatches(tooltip, extendedMismatchAreas)
    }

    /**
     * Creates containers for the visualization.
     */
    createContainers() {
        const parentContainer = document.createElement('div')
        parentContainer.setAttributeNS(null, 'style', 'position: relative;')
        const width = this.dom.getAttributeNS(null, 'width')
        const height = this.dom.getAttributeNS(null, 'height')
        const scrollable = document.createElement('div')
        scrollable.setAttributeNS(null, 'id', 'vis-sql-view')
        scrollable.setAttributeNS(null, 'class', 'scrollable')
        scrollable.setAttributeNS(null, 'viewBox', '0 0 ' + width + ' ' + height)
        scrollable.setAttributeNS(null, 'width', '100%')
        scrollable.setAttributeNS(null, 'height', '100%')
        const viewStyle = `width: ${width - 5}px;
             height: ${height - 5}px;
             overflow: scroll;
             padding:2.5px;`
        scrollable.setAttributeNS(null, 'style', viewStyle)
        parentContainer.appendChild(scrollable)
        return [parentContainer, scrollable]
    }

    setSize(size) {
        this.dom.setAttributeNS(null, 'width', size[0])
        this.dom.setAttributeNS(null, 'height', size[1])
    }
}

/**
 * Splits a qualified type name into a module prefix and the typename itself.
 */
function splitQualifiedTypeName(name) {
    var ix = name.lastIndexOf('.')
    if (ix < 0) {
        return {
            prefix: '',
            name: name,
        }
    }

    return {
        prefix: name.substr(0, ix + 1),
        name: name.substr(ix + 1),
    }
}

/**
 * Renders a 4-element array representing a color into a CSS-compatible rgba string.
 */
function renderColor(color) {
    const r = 255 * color.red
    const g = 255 * color.green
    const b = 255 * color.blue
    const a = color.alpha
    return 'rgba(' + r + ',' + g + ',' + b + ',' + a + ')'
}

/** Changes the alpha component of a color (represented as a 4-element array),
 * returning a new color.
 */
function changeAlpha(color, newAlpha) {
    return {
        red: color.red,
        green: color.green,
        blue: color.blue,
        alpha: newAlpha,
    }
}

/**
 * Renders a HTML representation of a message to be displayed in a tooltip,
 * which explains a type mismatch.
 */
function renderTypeHintMessage(
    receivedTypeName,
    expectedTypeName,
    receivedTypeColor,
    expectedTypeColor
) {
    const received = splitQualifiedTypeName(receivedTypeName)
    const expected = splitQualifiedTypeName(expectedTypeName)

    const receivedPrefix = '<span class="modulepath">' + received.prefix + '</span>'
    const receivedStyledSpan = '<span style="color: ' + renderColor(receivedTypeColor) + '">'
    const receivedSuffix = receivedStyledSpan + received.name + '</span>'

    const expectedPrefix = '<span class="modulepath">' + expected.prefix + '</span>'
    const expectedStyledSpan = '<span style="color: ' + renderColor(expectedTypeColor) + '">'
    const expectedSuffix = expectedStyledSpan + expected.name + '</span>'

    let message = 'Received ' + receivedPrefix + receivedSuffix + '<br>'
    message += 'Expected ' + expectedPrefix + expectedSuffix + '<br>'
    message += 'The database may perform an auto conversion.'
    return message
}

const textType = 'Builtins.Main.Text'
const customSqlTypePrefix = 'Standard.Database.Data.Sql.Sql_Type.'

/** Specifies opacity of interpolation background color. */
const interpolationBacgroundOpacity = 0.3

/**
 * Renders HTML for displaying an Enso parameter that is interpolated into the SQL code.
 */
function renderInterpolationParameter(theme, param) {
    const actualType = param.actual_type
    let value = param.value

    if (actualType == textType) {
        value = "'" + value.replaceAll("'", "''") + "'"
    }

    const actualTypeColor = theme.getColorForType(actualType)
    const fgColor = actualTypeColor
    let bgColor = changeAlpha(fgColor, interpolationBacgroundOpacity)
    const expectedEnsoType = param.expected_enso_type

    if (actualType == expectedEnsoType) {
        return renderRegularInterpolation(value, fgColor, bgColor)
    } else {
        let expectedType = expectedEnsoType
        if (expectedType === null) {
            expectedType = customSqlTypePrefix + param.expected_sql_type
        }

        const expectedTypeColor = theme.getColorForType(expectedType)
        const hoverBgColor = expectedTypeColor
        bgColor = changeAlpha(hoverBgColor, interpolationBacgroundOpacity)
        const hoverFgColor = theme.getForegroundColorForType(expectedType)

        const message = renderTypeHintMessage(
            actualType,
            expectedType,
            actualTypeColor,
            expectedTypeColor
        )

        return renderMismatchedInterpolation(
            value,
            message,
            fgColor,
            bgColor,
            hoverFgColor,
            hoverBgColor
        )
    }
}

/**
 * A helper that renders the HTML representation of a regular SQL interpolation.
 */
function renderRegularInterpolation(value, fgColor, bgColor) {
    let html =
        '<div class="interpolation" style="color:' +
        renderColor(fgColor) +
        ';background-color:' +
        renderColor(bgColor) +
        ';">'
    html += value
    html += '</div>'
    return html
}

/**
 * A helper that renders the HTML representation of a type-mismatched SQL interpolation.
 *
 * This only prepares the HTML code, to setup the interactions, `setupMouseInteractionForMismatches`
 * must be called after these HTML elements are added to the DOM.
 */
function renderMismatchedInterpolation(
    value,
    message,
    fgColor,
    bgColor,
    hoverFgColor,
    hoverBgColor
) {
    let html = '<div class="mismatch-parent">'
    html += '<div class="mismatch-mouse-area"></div>'
    html += '<div class="interpolation mismatch"'
    html +=
        ' style="color:' + renderColor(fgColor) + ';background-color:' + renderColor(bgColor) + ';"'
    html += ' data-fgColor="' + renderColor(fgColor) + '"'
    html += ' data-bgColor="' + renderColor(bgColor) + '"'
    html += ' data-fgColorHover="' + renderColor(hoverFgColor) + '"'
    html += ' data-bgColorHover="' + renderColor(hoverBgColor) + '"'
    html += ' data-message="' + encodeURIComponent(message) + '"'
    html += '>'
    html += value
    html += '</div>'
    html += '</div>'
    return html
}

/**
 * A hint tooltip that can be displayed above elements.
 */
class Tooltip {
    constructor(container) {
        this.tooltip = document.createElement('div')
        this.tooltip.setAttributeNS(null, 'class', 'tooltip')
        container.appendChild(this.tooltip)
        this.tooltipOwner = null
    }

    /**
     * Hides the tooltip.
     *
     * The actor parameter specifies who is initiating the hiding.
     * If this method is called but the tooltip has got a new owner in the meantime, the call is
     * ignored.
     */
    hide(actor) {
        if (this.tooltipOwner === null || this.tooltipOwner == actor) {
            this.tooltipOwner = null
            this.tooltip.style.opacity = 0
        }
    }

    /**
     * Shows the tooltip above the element represented by `actor`.
     *
     * Tooltip content is specified by the `message` which can include arbitrary HTML.
     */
    show(actor, message) {
        this.tooltipOwner = actor
        this.tooltip.innerHTML = message
        this.tooltip.style.opacity = 1

        const interpolantContainer = actor.parentElement
        const codeContainer = interpolantContainer.parentElement
        const scrollElement = codeContainer.parentElement

        const scrollOffsetX = scrollElement.scrollLeft
        const scrollOffsetY = scrollElement.scrollTop + scrollElement.offsetHeight

        const interpolantOffsetX = interpolantContainer.offsetLeft
        const interpolantOffsetY = interpolantContainer.offsetTop

        const centeringOffset = (interpolantContainer.offsetWidth - this.tooltip.offsetWidth) / 2
        const belowPadding = 3
        const belowOffset = interpolantContainer.offsetHeight + belowPadding

        const x = interpolantOffsetX - scrollOffsetX + centeringOffset
        const y = interpolantOffsetY - scrollOffsetY + belowOffset

        this.tooltip.style.transform = 'translate(' + x + 'px, ' + y + 'px)'
    }
}

/**
 * Sets up mouse events for the interpolated parameters that have a type mismatch.
 */
function setupMouseInteractionForMismatches(tooltip, elements) {
    function interpolationMouseEnter(event) {
        const target = this.parentElement.getElementsByClassName('mismatch')[0]
        const fg = target.getAttribute('data-fgColorHover')
        const bg = target.getAttribute('data-bgColorHover')
        const message = decodeURIComponent(target.getAttribute('data-message'))
        tooltip.show(target, message)
        target.style.color = fg
        target.style.backgroundColor = bg
    }
    function interpolationMouseLeave(event) {
        const target = this.parentElement.getElementsByClassName('mismatch')[0]
        const fg = target.getAttribute('data-fgColor')
        const bg = target.getAttribute('data-bgColor')
        target.style.color = fg
        target.style.backgroundColor = bg
        tooltip.hide(target)
    }

    for (let i = 0; i < elements.length; ++i) {
        elements[i].addEventListener('mouseenter', interpolationMouseEnter)
        elements[i].addEventListener('mouseleave', interpolationMouseLeave)
    }
}

const visualizationStyle = `
    <style>
    .sql {
        font-family: DejaVuSansMonoBook, sans-serif;
        font-size: 12px;
        margin-left: 7px;
        margin-top: 5px;
    }
    .interpolation {
        border-radius: 6px;
        padding:1px 2px 1px 2px;
        display: inline;
    }
    .mismatch-parent {
        position: relative;
        display: inline-flex;
        justify-content: center;
    }
    .mismatch-mouse-area {
        display: inline;
        position: absolute;
        width: 150%;
        height: 150%;
        align-self: center;
        z-index: 0;
    }
    .mismatch {
        z-index: 1;
    }
    .modulepath {
        color: rgba(150, 150, 150, 0.9);
    }
    .tooltip {
        font-family: DejaVuSansMonoBook, sans-serif;
        font-size: 12px;
        opacity: 0;
        transition: opacity 0.2s;
        display: inline-block;
        white-space: nowrap;
        background-color: rgba(249, 249, 249, 1);
        box-shadow: 0 0 16px rgba(0, 0, 0, 0.16);
        text-align: left;
        border-radius: 6px;
        padding: 5px;
        position: absolute;
        z-index: 99999;
        pointer-events: none;
    }
    </style>
    `

return SqlVisualization
