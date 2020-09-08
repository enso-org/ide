class TableViewVisualization extends Visualization {
    static inputType = "Any"

    onDataReceived(data) {

        var tableOf = function (content, level) {
            var open = '<table class="level' + level + '">';
            return open + content + "</table>";
        }

        var hasExactlyKeys = function (keys, obj) {
            return Object.keys(obj).length == keys.length && keys.every(k => obj.hasOwnProperty(k));
        };

        var getAtNestedKey = function (data, key) {
            var res = data;
            key.forEach(function (k) { res = res[k]; });
            return res;
        }

        var repNestedKey = function (key) {
            return key.join(".");
        }

        var generateNestings = function (data, key) {
            var first = getAtNestedKey(data[0], key);
            if (!(first instanceof Object)) return [key];
            var firstKeys = Object.keys(first);
            var isNestable = data.every(obj => hasExactlyKeys(firstKeys, getAtNestedKey(obj, key)));
            if (isNestable) {
                var withNests = firstKeys.map(k => key.concat([k]));
                var furtherNestings = withNests.map(k => generateNestings(data, k));
                return [].concat.apply([], furtherNestings);
            } else {
                return [key];
            }
        }

        var isObjectMatrix = function (data) {
            var isList = Array.isArray(data) && data[0];
            if (!isList || !(typeof data[0] === "object"))  return false;
            var firstKeys = Object.keys(data[0]);
            return data.every(obj => hasExactlyKeys(firstKeys, obj));
        }

        var genObjectMatrix = function (data, level) {
            var result = "<tr><th></th>";
            var keys   = Object.keys(data[0]);
            var nests  = [].concat.apply([], keys.map(k => generateNestings(data,[k])));
            nests.forEach(function (key) {
                result += ("<th>" + repNestedKey(key) + "</th>");
            });
            result += "</tr>";
            data.forEach(function (row, ix) {
                result += ("<tr><th>" + ix + "</th>");
                nests.forEach(function (k) {
                    result += toTableCell(getAtNestedKey(row, k), level);
                });
                result += ("</tr>")
            });
            return tableOf(result, level);
        }

        var isMatrix = function (data) {
            var isList = Array.isArray(data) && data[0];
            if (!isList) return false;
            var firstIsArray = Array.isArray(data[0]);
            if (!firstIsArray) return false;
            var firstLen = data[0].length;
            var eachHasProperLen = data.every(d => d.length == firstLen);
            return eachHasProperLen;
        }

        var genMatrix = function (data, level, header) {
            var result = "<tr><th></th>";
            if (header) {
                header.forEach(function (elt, ix) {
                    result += ("<th>" + elt + "</th>");
                });
            } else {
                data[0].forEach(function (elt, ix) {
                    result += ("<th>" + ix + "</th>");
                });
            };
            result += "</tr>";
            table = []

            data.forEach(function(d, i) {
                d.forEach(function(elem, idx) {
                    table[idx] = table[idx] || []
                    table[idx].push(elem)
                })
            })

            table.forEach(function (row, ix) {
                result += ("<tr><th>" + ix + "</th>");
                row.forEach(function (d) {
                    result += toTableCell(d, level);
                });
                result += ("</tr>")
            });
            return tableOf(result, level);
        }

        var genGenericTable = function (data, level) {
            var result = "";
            data.forEach(function (point, ix) {
                result += ("<tr><th>" + ix + "</th>" + toTableCell(point, level) + "</tr>");
            });
            return tableOf(result, level);
        }

        var genRowObjectTable = function (data, level) {
            var keys = Object.keys(data);
            var result = "<tr>";
            keys.forEach(function (key) {
                result += ("<th>" + key + "</th>");
            });
            result += "</tr><tr>";
            keys.forEach(function (key) {
                result += toTableCell(data[key], level);
            });
            result += "</tr>";
            return tableOf(result, level);
        }

        var toTableCell = function (data, level) {
            if (Array.isArray(data)) {
                return "<td>" + genTable(data, level + 1) + "</td>";
            } else if (data instanceof Object) {
                return "<td>" + genRowObjectTable(data, level + 1) + "</td>";
            } else {
                if (data === undefined || data === null) data = "";
                var res = data.toString();
                return '<td class="plaintext">' + (res === "" ? "N/A" : res) + '</td>';
            }
        }

        var genTable = function (data, level, header) {
            if (isMatrix(data)) {
                return genMatrix(data, level, header);
            } else if (isObjectMatrix(data)) {
                return genObjectMatrix(data, level);
            } else {
                return genGenericTable(data, level);
            }
        }

        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild);
        }

        const style_dark = `
        <style>
        table {
            font-family: sans-serif;
            font-size: 12px;
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
        
        th {
            color: rgba(255, 255, 255, 0.7);
            font-weight: 400;
        }
        
        td,
        th {
            background-color: rgba(255, 255, 255, 0.03);
        }
        </style>
        `;

        const style_light = `
        <style>
        table {
            font-family: sans-serif;
            font-size: 12px;
        }

        td {
            color: rgba(0, 0, 0, 0.9);
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

        th {
            color: rgba(0, 0, 0, 0.7);
            font-weight: 400;
        }

        td,
            th {
            background-color: rgba(0, 0, 0, 0.03);
        }
        </style>`

        const width   = this.dom.getAttributeNS(null, "width");
        const height  = this.dom.getAttributeNS(null, "height");
        const tabElem = document.createElement("div");
        tabElem.setAttributeNS(null,"id"     ,"vis-tbl-view");
        tabElem.setAttributeNS(null,"viewBox","0 0 " + width + " " + height);
        tabElem.setAttributeNS(null,"width"  ,"100%");
        tabElem.setAttributeNS(null,"height" ,"100%");
        const tblViewStyle = `width: ${width-10}px;
                              height: ${height-10}px;
                              overflow: scroll;
                              padding:2.5px;`;
        tabElem.setAttributeNS(null,"style"  ,tblViewStyle);
        this.dom.appendChild(tabElem);

        var parsedData    = JSON.parse(data);
        var table         = genTable(parsedData.data || parsedData, 0, parsedData.header);
        tabElem.innerHTML = style_dark+table;

    }

    setSize(size) {
        this.dom.setAttributeNS(null, "width", size[0]);
        this.dom.setAttributeNS(null, "height", size[1]);
    }
}

return TableViewVisualization;
