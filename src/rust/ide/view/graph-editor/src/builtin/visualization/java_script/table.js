/** Table visualization. */

// ============================
// === Style Initialisation ===
// ============================

loadStyleFromString(scrollbarStyle)

// ===========================
// === Table visualization ===
// ===========================

class TableVisualization extends Visualization {
    static inputType = 'Any'
    static label = 'Table'

    constructor(data) {
        super(data)
        this.setPreprocessorModule('Standard.Visualization.Table.Visualization')
        this.setPreprocessorCode(`x -> here.prepare_visualization x 1000`)
    }

    onDataReceived(data) {
        function tableOf(content, level) {
            let open = '<table class="level' + level + '">'
            return open + content + '</table>'
        }

        function hasExactlyKeys(keys, obj) {
            return Object.keys(obj).length === keys.length && keys.every(k => obj.hasOwnProperty(k))
        }

        function getAtNestedKey(data, key) {
            let res = data
            key.forEach(k => (res = res[k]))
            return res
        }

        function repNestedKey(key) {
            return key.join('.')
        }

        function generateNestings(data, key) {
            let first = getAtNestedKey(data[0], key)
            if (!(first instanceof Object)) return [key]
            let firstKeys = Object.keys(first)
            let isNestable = data.every(obj => hasExactlyKeys(firstKeys, getAtNestedKey(obj, key)))
            if (isNestable) {
                let withNests = firstKeys.map(k => key.concat([k]))
                let furtherNestings = withNests.map(k => generateNestings(data, k))
                return [].concat.apply([], furtherNestings)
            } else {
                return [key]
            }
        }

        function isObjectMatrix(data) {
            let isList = Array.isArray(data) && data[0]
            if (!isList || !(typeof data[0] === 'object')) return false
            let firstKeys = Object.keys(data[0])
            return data.every(obj => hasExactlyKeys(firstKeys, obj))
        }

        function genObjectMatrix(data, level) {
            let result = '<tr><th></th>'
            let keys = Object.keys(data[0])
            let nests = [].concat.apply(
                [],
                keys.map(k => generateNestings(data, [k]))
            )
            nests.forEach(key => {
                result += '<th>' + repNestedKey(key) + '</th>'
            })
            result += '</tr>'
            data.forEach((row, ix) => {
                result += '<tr><th>' + ix + '</th>'
                nests.forEach(k => {
                    result += toTableCell(getAtNestedKey(row, k), level)
                })
                result += '</tr>'
            })
            return tableOf(result, level)
        }

        function isMatrix(data) {
            let isList = Array.isArray(data) && data[0]
            if (!isList) return false
            let firstIsArray = Array.isArray(data[0])
            if (!firstIsArray) return false
            let firstLen = data[0].length
            return data.every(d => d.length === firstLen)
        }

        function genMatrix(data, level, header) {
            let result = '<tr><th></th>'
            if (header) {
                header.forEach((elt, ix) => {
                    result += '<th>' + elt + '</th>'
                })
            } else {
                data[0].forEach((elt, ix) => {
                    result += '<th>' + ix + '</th>'
                })
            }
            result += '</tr>'
            const table = []

            data.forEach((d, i) => {
                d.forEach((elem, idx) => {
                    table[idx] = table[idx] || []
                    table[idx].push(elem)
                })
            })

            table.forEach((row, ix) => {
                result += '<tr><th>' + ix + '</th>'
                row.forEach(d => {
                    result += toTableCell(d, level)
                })
                result += '</tr>'
            })
            return tableOf(result, level)
        }

        function genGenericTable(data, level) {
            let result = ''
            if (Array.isArray(data)) {
                data.forEach((point, ix) => {
                    result += '<tr><th>' + ix + '</th>' + toTableCell(point, level) + '</tr>'
                })
            } else {
                result += '<tr>' + toTableCell(data, level) + '</tr>'
            }
            return tableOf(result, level)
        }

        function genRowObjectTable(data, level) {
            let keys = Object.keys(data)
            let result = '<tr>'
            keys.forEach(key => {
                result += '<th>' + key + '</th>'
            })
            result += '</tr><tr>'
            keys.forEach(key => {
                result += toTableCell(data[key], level)
            })
            result += '</tr>'
            return tableOf(result, level)
        }

        function toTableCell(data, level) {
            if (Array.isArray(data)) {
                return '<td>' + genTable(data, level + 1) + '</td>'
            } else if (data instanceof Object) {
                return '<td>' + genRowObjectTable(data, level + 1) + '</td>'
            } else {
                if (data === undefined || data === null) data = ''
                let res = data.toString()
                return '<td class="plaintext">' + (res === '' ? 'N/A' : res) + '</td>'
            }
        }

        function genTable(data, level, header) {
            if (isMatrix(data)) {
                return genMatrix(data, level, header)
            } else if (isObjectMatrix(data)) {
                return genObjectMatrix(data, level)
            } else {
                return genGenericTable(data, level)
            }
        }

        function genDataframe(parsedData) {
            let result = ''
            function addHeader(content) {
                result += '<th>' + content + '</th>'
            }
            function addCell(content) {
                result += '<td class="plaintext">' + content + '</td>'
            }
            result += '<tr>'
            parsedData.indices_header.forEach(addHeader)
            parsedData.header.forEach(addHeader)
            result += '</tr>'
            let rows = 0
            if (parsedData.data.length > 0) {
                rows = parsedData.data[0].length
            } else if (parsedData.indices.length > 0) {
                rows = parsedData.indices[0].length
            }
            for (let i = 0; i < rows; ++i) {
                result += '<tr>'
                parsedData.indices.forEach(ix => addHeader(ix[i]))
                parsedData.data.forEach(col => addCell(col[i]))
                result += '</tr>'
            }
            return tableOf(result, 0)
        }

        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild)
        }

        const style_dark = `
        <style>
        table, .hiddenrows {
            font-family: DejaVuSansMonoBook, sans-serif;
            font-size: 12px;
        }

        table {
            border-spacing: 1px;
            padding: 1px;
        }

        table > tbody > tr:first-child > th:first-child,
        table > tbody > tr:first-child > td:first-child {
            border-top-left-radius: 9px;
        }

        table > tbody > tr:first-child > th:last-child,
        table > tbody > tr:first-child > td:last-child {
            border-top-right-radius: 9px;
        }

        table > tbody > tr:last-child > th:first-child,
        table > tbody > tr:last-child > td:first-child {
            border-bottom-left-radius: 9px;
        }

        table > tbody > tr:last-child > th:last-child,
        table > tbody > tr:last-child > td:last-child {
            border-bottom-right-radius: 9px;
        }

        td {
            color: rgba(255, 255, 255, 0.9);
            padding: 0;
        }

        td.plaintext,
        th {
            padding: 5px;
        }

        th,
        td {
            border: 1px solid transparent;
            background-clip: padding-box;
        }

        th, .hiddenrows {
            color: rgba(255, 255, 255, 0.7);
            font-weight: 400;
        }

        td {
            background-color: rgba(255, 255, 255, 0.03);
        }

        th {
            background-color: rgba(255, 255, 200, 0.1);
        }

        .hiddenrows {
            margin-left: 5px;
            margin-top: 5px;
        }
        </style>
        `

        const style_light = `
        <style>
        table, .hiddenrows {
            font-family: DejaVuSansMonoBook, sans-serif;
            font-size: 12px;
        }

        table {
            border-spacing: 1px;
            padding: 1px;
        }

        table > tbody > tr:first-child > th:first-child,
        table > tbody > tr:first-child > td:first-child {
            border-top-left-radius: 9px;
        }

        table > tbody > tr:first-child > th:last-child,
        table > tbody > tr:first-child > td:last-child {
            border-top-right-radius: 9px;
        }

        table > tbody > tr:last-child > th:first-child,
        table > tbody > tr:last-child > td:first-child {
            border-bottom-left-radius: 9px;
        }

        table > tbody > tr:last-child > th:last-child,
        table > tbody > tr:last-child > td:last-child {
            border-bottom-right-radius: 9px;
        }

        td {
            color: rgba(0, 0, 0, 0.7);
            padding: 0;
        }

        td.plaintext,
            th {
            padding: 5px;
        }

        th,
            td {
            border: 1px solid transparent;
            background-clip: padding-box;
        }

        th, .hiddenrows {
            color: rgba(0, 0, 0, 0.9);
            font-weight: 400;
        }

        td {
            background-color: rgba(0, 0, 0, 0.025);
        }

        th {
            background-color: rgba(30, 30, 20, 0.1);
        }

        .hiddenrows {
            margin-left: 5px;
            margin-top: 5px;
        }
        </style>`

        const tabElem = document.createElement('div')
        tabElem.setAttributeNS(null, 'id', 'vis-tbl-view')
        tabElem.setAttributeNS(null, 'class', 'scrollable')
        tabElem.setAttributeNS(null, 'width', '100%')
        tabElem.setAttributeNS(null, 'height', '100%')
        this.tabElem = tabElem
        this.dom.appendChild(tabElem)
        this.updateTableSize()

        let parsedData = data
        if (typeof data === 'string') {
            parsedData = JSON.parse(data)
        }

        let style = style_light
        if (document.getElementById('root').classList.contains('dark-theme')) {
            style = style_dark
        }

        if (parsedData.error !== undefined) {
            tabElem.innerHTML = 'Error: ' + parsedData.error
        } else if (parsedData.json !== undefined) {
            const table = genTable(parsedData.json, 0, undefined)
            tabElem.innerHTML = style + table
        } else {
            const table = genDataframe(parsedData)
            let suffix = ''
            const allRowsCount = parsedData.all_rows_count
            if (allRowsCount !== undefined) {
                const includedRowsCount = parsedData.data.length > 0 ? parsedData.data[0].length : 0
                const hiddenCount = allRowsCount - includedRowsCount
                if (hiddenCount > 0) {
                    let rows = 'rows'
                    if (hiddenCount == 1) {
                        rows = 'row'
                    }
                    suffix =
                        '<span class="hiddenrows">&#8230; and ' +
                        hiddenCount +
                        ' more ' +
                        rows +
                        '.</span>'
                }
            }
            tabElem.innerHTML = style + table + suffix
        }
    }

    updateTableSize() {
        if (this.tabElem !== undefined) {
            const width = this.dom.getAttributeNS(null, 'width')
            const height = this.dom.getAttributeNS(null, 'height')
            const tblViewStyle = `width: ${width - 5}px;
                 height: ${height - 5}px;
                 overflow: scroll;
                 padding:2.5px;`
            this.tabElem.setAttributeNS(null, 'style', tblViewStyle)
            this.tabElem.setAttributeNS(null, 'viewBox', '0 0 ' + width + ' ' + height)
        }
    }

    setSize(size) {
        this.dom.setAttributeNS(null, 'width', size[0])
        this.dom.setAttributeNS(null, 'height', size[1])
        this.updateTableSize()
    }
}

return TableVisualization
