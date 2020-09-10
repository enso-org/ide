function loadScript(url) {
    var script = document.createElement("script");
    script.src = url;

    document.head.appendChild(script);
}

loadScript('https://d3js.org/d3.v4.min.js');

class ScatterPlot extends Visualization {
    static inputType = "Any"

    onDataReceived(data) {
        this.setPreprocessor("None");

        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild);
        }
        let width = this.dom.getAttributeNS(null, "width");
        let height = this.dom.getAttributeNS(null, "height");

        const svgElem = document.createElementNS(null, "div");
        svgElem.setAttributeNS(null, "class", "vis-scatterplot");
        svgElem.setAttributeNS(null, "viewBox", 0 + " " + 0 + " " + width + " " + height);
        svgElem.setAttributeNS(null, "width", "100%");
        svgElem.setAttributeNS(null, "height", "100%");
        svgElem.setAttributeNS(null, "transform", "matrix(1 0 0 -1 0 0)");

        // TODO: Remove this.
        svgElem.onmousedown = function () {
            console.log("Clicked");
        }

        this.dom.appendChild(svgElem);

        let margin = {top: 20, right: 20, bottom: 20, left: 20};
        width = width - margin.left - margin.right;
        height = height - margin.top - margin.bottom;

        let svg = d3.select(svgElem)
            .append("svg")
            .attr("width", width + margin.left + margin.right)
            .attr("height", height + margin.top + margin.bottom)
            .append("g")
            .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

        d3.csv("https://raw.githubusercontent.com/holtzy/D3-graph-gallery/master/DATA/iris.csv", function (_data) {
            var x = d3.scaleLinear()
                .domain([4, 8])
                .range([0, width]);
            var xAxis = svg.append("g")
                .attr("transform", "translate(0," + height + ")")
                .call(d3.axisBottom(x));

            var y = d3.scaleLinear()
                .domain([0, 9])
                .range([height, 0]);
            svg.append("g")
                .call(d3.axisLeft(y));

            var clip = svg.append("defs").append("svg:clipPath")
                .attr("id", "clip")
                .append("svg:rect")
                .attr("width", width)
                .attr("height", height)
                .attr("x", 0)
                .attr("y", 0);

            var color = d3.scaleOrdinal()
                .domain(["setosa", "versicolor", "virginica"])
                .range(["#440154ff", "#21908dff", "#fde725ff"])

            var brush = d3.brushX()
                .extent([[0, 0], [width, height]])
                .on("end", updateChart)

            var scatter = svg.append('g')
                .attr("clip-path", "url(#clip)")

            scatter
                .selectAll("circle")
                .data(_data)
                .enter()
                .append("circle")
                .attr("cx", function (d) {
                    return x(d.Sepal_Length);
                })
                .attr("cy", function (d) {
                    return y(d.Petal_Length);
                })
                .attr("r", 8)
                .style("fill", function (d) {
                    return color(d.Species)
                })
                .style("opacity", 0.5)

            scatter
                .append("g")
                .attr("class", "brush")
                .call(brush);

            var idleTimeout

            function idled() {
                idleTimeout = null;
            }

            function updateChart() {
                let extent = d3.event.selection;

                if (!extent) {
                    if (!idleTimeout) return idleTimeout = setTimeout(idled, 350);
                    x.domain([4, 8])
                } else {
                    x.domain([x.invert(extent[0]), x.invert(extent[1])])
                    scatter.select(".brush").call(brush.move, null)
                }

                xAxis.transition().duration(1000).call(d3.axisBottom(x))
                scatter
                    .selectAll("circle")
                    .transition().duration(1000)
                    .attr("cx", function (d) {
                        return x(d.Sepal_Length);
                    })
                    .attr("cy", function (d) {
                        return y(d.Petal_Length);
                    })

            }
        });
    }

    setSize(size) {
        this.dom.setAttributeNS(null, "width", size[0]);
        this.dom.setAttributeNS(null, "height", size[1]);
    }
}

return ScatterPlot;
