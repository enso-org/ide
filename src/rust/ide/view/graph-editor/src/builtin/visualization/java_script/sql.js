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
        .mismatch {
            /*border-style: solid;
            border-width: 1px;
            border-color: rgba(255, 255, 255, 1);*/
        }
        .tooltip {
            font-family: DejaVuSansMonoBook, sans-serif;
            font-size: 12px;
            opacity: 0;
            transition: opacity 0.2s;
            width: 220px;
            background-color: black;
            color: #fff;
            text-align: center;
            border-radius: 6px;
            padding: 5px;
            position: absolute;
            z-index: 99999 !important;
            pointer-events: none;
        }
        </style>
        `

        function renderColor(color) {
            return 'rgba(' + color[0] + ',' + color[1] + ',' + color[2] + ',' + color[3] + ')'
        }

        const bgAlpha = 0.25
        const theme = this.theme

        function simplifyTypeName(name) {
            const builtinPrefix = 'Builtins.Main.'
            if (name.startsWith(builtinPrefix)) {
                return name.slice(builtinPrefix.length)
            }
            return name
        }

        function renderParameter(param) {
            const actualType = param.actual_type
            let value = param.value

            if (actualType == 'Builtins.Main.Text') {
                value = "'" + value.replace("'", "''") + "'"
            }

            const fgColor = theme.getColorForType()
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

                const hoverBgColor = theme.getColorForType(expectedType)
                bgColor = [...hoverBgColor.slice(0, 3), bgAlpha]
                const hoverFgColor = theme.getForegroundColorForType(expectedType)

                const message =
                    'Got ' +
                    simplifyTypeName(actualType) +
                    ', but ' +
                    simplifyTypeName(expectedType) +
                    ' was expected. The database engine may perform a conversion.'

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
            const x = this.offsetLeft - tooltip.offsetWidth / 2 + this.offsetWidth / 2
            const y = this.offsetTop - this.parentElement.offsetHeight - tooltip.offsetHeight + 15
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

        // listeners.forEach(function(obj) {
        //     const elem = document.getElementById(obj.id)
        //     console.log(elem)
        //     console.log(tooltip)

        //     elem.addEventListener('mouseenter', (e) => {
        //         elem.style.color = renderColor(obj.hover[0])
        //         elem.style.backgroundColor = renderColor(obj.hover[1])
        //         const bound = elem.getBoundingClientRect()
        //         const x = (bound.left - 10) + 'px';
        //         const y = (bound.top - 20) + 'px';
        //         tooltip.style.transform = 'translate(' + x + ',' + y + ')'
        //         console.log(bound)
        //         console.log(tooltip.style)
        //         tooltip.innerHTML = "ABCDEF"
        //         tooltip.style.opacity = 1
        //     })

        //     elem.addEventListener('mouseleave', () => {
        //         elem.style.color = renderColor(obj.regular[0])
        //         elem.style.backgroundColor = renderColor(obj.regular[1])
        //         tooltip.style.opacity = 0
        //     })
        // })
    }

    setSize(size) {
        this.dom.setAttributeNS(null, 'width', size[0])
        this.dom.setAttributeNS(null, 'height', size[1])
    }
}

return SqlVisualization
