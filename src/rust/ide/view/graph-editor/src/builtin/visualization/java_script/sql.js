loadScript('https://cdnjs.cloudflare.com/ajax/libs/sql-formatter/4.0.2/sql-formatter.min.js')

class SqlVisualization extends Visualization {
    static inputType = 'Any'
    static label = 'SQL'

    constructor(api) {
        super(api)
        this.setPreprocessorModule('Standard.Base')
        this.setPreprocessorCode(`
           x ->
               stmt = x.to_sql.prepare
               dialect = x.connection.dialect.name
               Json.from_pairs [['sql', stmt.first], ['interpolations', stmt.second], ['dialect', dialect]] . to_text
        `)
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
        span {
            padding-left: 3px;
            padding-right: 3px;
            padding-top: 1px;
            padding-bottom: 1px;
            border-radius: 5px;
        }
        pre {
            margin-left: 12px;
        }
        .string {
            background-color: rgba(255, 100, 200, 0.8);
        }
        .number {
            background-color: rgba(120, 255, 120, 0.8);
        }
        .boolean {
            background-color: rgba(110, 110, 240, 0.8);
        }
        .other {
            background-color: rgba(200, 200, 200, 0.8);
        }
        </style>
        `

        function renderParameter(param) {
            const item = param[0]
            if (typeof item === 'string') {
                return '<span class="string">\'' + item + "'</span>"
            } else if (typeof item === 'number') {
                return '<span class="number">' + item + '</span>'
            } else if (typeof item === 'boolean') {
                return '<span class="boolean">' + item + '</span>'
            } else {
                return '<span class="other">' + JSON.stringify(item) + '</span>'
            }
        }

        const params = parsedData.interpolations.map(renderParameter)
        let language = 'sql'
        if (parsedData.dialect == 'postgresql') {
            language = 'postgresql'
        }
        const formatted = sqlFormatter.format(parsedData.sql, {
            params: params,
            language: language,
        })

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
        elem.innerHTML = style + '<pre>' + formatted + '</pre>'
        this.dom.appendChild(elem)
    }

    setSize(size) {
        this.dom.setAttributeNS(null, 'width', size[0])
        this.dom.setAttributeNS(null, 'height', size[1])
    }
}

return SqlVisualization
