loadScript('https://cdnjs.cloudflare.com/ajax/libs/sql-formatter/4.0.2/sql-formatter.min.js')

class SqlVisualization extends Visualization {
    static inputType = 'Any' // 'Standard.Database.Data.Table.Table | Standard.Database.Data.Column.Column'
    static label = 'SQL Query'

    constructor(api) {
        super(api)
        this.setPreprocessorModule('Standard.Visualization.Sql.Visualization')
        this.setPreprocessorCode(`x -> here.prepare_visualization x`)

        // mock theme
        function hash(s) {
            let sum = 0
            for (let i = 0; i < s.length; ++i) {
                sum = (sum + 31 * s.charCodeAt(i)) % 255
            }
            return sum
        }
        function getColor(name) {
            return [hash('r' + name), hash('g' + name), hash('b' + name), 1]
        }
        this.theme = {
            getColorForType: getColor,
            getForegroundColorForType: function (x) {
                return [255, 255, 255, 1]
            },
        }
    }

    onDataReceived(data) {
        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild)
        }
        let parsedData = data
        if (typeof data === 'string') {
            parsedData = JSON.parse(data)
        }

        const style = `
        <style>
        .sql {
            font-family: DejaVuSansMonoBook, sans-serif;
            font-size: 12px;
            margin-left: 7px;
        }
        .interpolation {
            border-radius: 6px;
            padding:1px 2px 1px 2px;
            display: inline;
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
            background-color: rgba(245, 245, 245, 1);
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

        function renderColor(color) {
            return 'rgba(' + color[0] + ',' + color[1] + ',' + color[2] + ',' + color[3] + ')'
        }

        const bgAlpha = 0.25
        const theme = this.theme

        function splitTypeName(name) {
            var ix = name.lastIndexOf('.')
            if (ix < 0) {
                return ['', name]
            }

            return [name.substr(0, ix + 1), name.substr(ix + 1)]
        }

        function renderParameter(param) {
            const actualType = param.actual_type
            let value = param.value

            if (actualType == 'Builtins.Main.Text') {
                value = "'" + value.replaceAll("'", "''") + "'"
            }

            const actualTypeColor = theme.getColorForType(actualType)
            const fgColor = actualTypeColor
            let bgColor = [...fgColor.slice(0, 3), bgAlpha]
            const expectedEnsoType = param.expected_enso_type

            let html = ''
            if (actualType == expectedEnsoType) {
                const elem = document.createElement('div')
                html +=
                    '<div class="interpolation" style="color:' +
                    renderColor(fgColor) +
                    ';background-color:' +
                    renderColor(bgColor) +
                    ';">'
                html += value
                html += '</div>'
            } else {
                let expectedType = expectedEnsoType
                if (expectedType === null) {
                    expectedType = 'Standard.Database.Data.Sql.Sql_Type.' + param.expected_sql_type
                }

                const expectedTypeColor = theme.getColorForType(expectedType)
                const hoverBgColor = expectedTypeColor
                bgColor = [...hoverBgColor.slice(0, 3), bgAlpha]
                const hoverFgColor = theme.getForegroundColorForType(expectedType)

                const received = splitTypeName(actualType)
                const expected = splitTypeName(expectedType)
                let message =
                    'Received <span class="modulepath">' +
                    received[0] +
                    '</span><span style="color: ' +
                    renderColor(actualTypeColor) +
                    '">' +
                    received[1] +
                    '</span><br>'
                message +=
                    'Expected <span class="modulepath">' +
                    expected[0] +
                    '</span><span style="color: ' +
                    renderColor(expectedTypeColor) +
                    '">' +
                    expected[1] +
                    '</span><br>'
                message += 'The database may perform an auto conversion.'

                html += '<div class="interpolation mismatch"'
                html +=
                    ' style="color:' +
                    renderColor(fgColor) +
                    ';background-color:' +
                    renderColor(bgColor) +
                    ';"'
                html += ' data-fgColor="' + renderColor(fgColor) + '"'
                html += ' data-bgColor="' + renderColor(bgColor) + '"'
                html += ' data-fgColorHover="' + renderColor(hoverFgColor) + '"'
                html += ' data-bgColorHover="' + renderColor(hoverBgColor) + '"'
                html += ' data-message="' + encodeURIComponent(message) + '"'
                html += '>'
                html += value
                html += '</div>'
            }
            return html
        }

        console.log(parsedData)
        let visHtml = style
        if (parsedData.error !== undefined) {
            visHtml += parsedData.error
        } else {
            const params = parsedData.interpolations.map(renderParameter)
            let language = 'sql'
            if (parsedData.dialect == 'postgresql') {
                language = 'postgresql'
            }
            const formatted = sqlFormatter.format(parsedData.code, {
                params: params,
                language: language,
            })
            visHtml += '<pre class="sql">' + formatted + '</pre>'
        }
        const container = document.createElement('div')
        container.setAttributeNS(null, 'style', 'position: relative;')
        const width = this.dom.getAttributeNS(null, 'width')
        const height = this.dom.getAttributeNS(null, 'height')
        const elem = document.createElement('div')
        elem.setAttributeNS(null, 'id', 'vis-sql-view')
        elem.setAttributeNS(null, 'class', 'scrollable')
        elem.setAttributeNS(null, 'viewBox', '0 0 ' + width + ' ' + height)
        elem.setAttributeNS(null, 'width', '100%')
        elem.setAttributeNS(null, 'height', '100%')
        const viewStyle = `width: ${width - 10}px;
             height: ${height - 10}px;
             overflow: scroll;
             padding:2.5px;`
        elem.setAttributeNS(null, 'style', viewStyle)
        elem.innerHTML = visHtml
        const tooltip = document.createElement('div')
        tooltip.setAttributeNS(null, 'class', 'tooltip')
        container.appendChild(elem)
        container.appendChild(tooltip)
        this.dom.appendChild(container)

        const dom = this.dom
        let tooltipOwner = null
        function interpolationMouseEnter(event) {
            const fg = this.getAttribute('data-fgColorHover')
            const bg = this.getAttribute('data-bgColorHover')
            console.log(fg, bg)
            const message = decodeURIComponent(this.getAttribute('data-message'))
            console.log(message)
            this.style.color = fg
            this.style.backgroundColor = bg
            const tooltip = dom.getElementsByClassName('tooltip')[0]
            tooltipOwner = this
            tooltip.innerHTML = message
            tooltip.style.opacity = 1
            const pre = this.parentElement
            const scrollElement = this.parentElement.parentElement
            const scrollX = scrollElement.scrollLeft
            const scrollY = scrollElement.scrollTop
            const x = this.offsetLeft - tooltip.offsetWidth / 2 + this.offsetWidth / 2 - scrollX
            const y = this.offsetTop - elem.offsetTop - pre.offsetTop - scrollY - 160
            console.log(x, y)
            tooltip.style.transform = 'translate(' + x + 'px, ' + y + 'px)'
        }
        function interpolationMouseLeave(event) {
            const fg = this.getAttribute('data-fgColor')
            const bg = this.getAttribute('data-bgColor')
            console.log(fg, bg)
            this.style.color = fg
            this.style.backgroundColor = bg
            dom.getElementsByClassName('tooltip')[0].style.opacity = 0
            if (tooltipOwner === null || tooltipOwner == this) {
                const tooltip = dom.getElementsByClassName('tooltip')[0]
                tooltipOwner = null
                tooltip.style.opacity = 0
            }
        }
        const mismatches = this.dom.getElementsByClassName('mismatch')
        for (let i = 0; i < mismatches.length; ++i) {
            mismatches[i].addEventListener('mouseenter', interpolationMouseEnter)
            mismatches[i].addEventListener('mouseleave', interpolationMouseLeave)
        }
    }

    setSize(size) {
        this.dom.setAttributeNS(null, 'width', size[0])
        this.dom.setAttributeNS(null, 'height', size[1])
    }
}

return SqlVisualization
