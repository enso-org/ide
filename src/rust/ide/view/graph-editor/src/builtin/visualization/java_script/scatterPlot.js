function loadScript(url) {
    let script = document.createElement("script");
    script.src = url;

    document.head.appendChild(script);
}

loadScript('https://d3js.org/d3.v4.min.js');

/**
 * A d3.js ScatterPlot visualization.
 *
 * To zoom in just select wanted fragment of the plot.
 * To zoom out double click on the plot.
 *
 * Data format (json):
 * {
 *  "axis" : {
 *     "x" : { "label" : "x-axis label", "scale" : "linear" },
 *     "y" : { "label" : "y-axis label", "scale" : "logarithmic" },
 *  },
 *  "focus" { "x" : 1.7, "y" : 2.1, "zoom" : 3.0 },
 *  "points" : {
 *     "labels" : "visible" | "invisible",
 *     "connected" : "yes" | "no"
 *  }
 *  "data" : [
 *     { "x" : 0.1, "y" : 0.7, "label" : "foo", "color" : "FF0000", "shape" : "circle", "size" : 0.2 },
 *     ...
 *     { "x" : 0.4, "y" : 0.2, "label" : "baz", "color" : "0000FF", "shape" : "square", "size" : 0.3 }
 *  ]
 * }
 */
class ScatterPlot extends Visualization {
    static inputType = "Any"

    onDataReceived(data) {
        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild);
        }

        let width     = this.dom.getAttributeNS(null, "width");
        let height    = this.dom.getAttributeNS(null, "height");
        const divElem = this.createDivElem(width, height);

        let parsedData = JSON.parse(data);
        let axis       = parsedData.axis || {x: {scale: "linear" }, y: {scale: "linear" }};
        let focus      = parsedData.focus; // TODO : This should be dropped as isn't easily doable with d3.js.
        let points     = parsedData.points || {labels: "invisible", connected: "no"};
        let dataPoints = parsedData.data || {};

        ///////////
        /// Box ///
        ///////////

        let margin = {top: 10, right: 10, bottom: 35, left: 40};
        if (axis.x.label === undefined && axis.y.label === undefined) {
            margin = {top: 20, right: 20, bottom: 20, left: 20};
        } else if (axis.x.label === undefined) {
            margin = {top: 10, right: 20, bottom: 35, left: 20};
        } else if (axis.y.label === undefined) {
            margin = {top: 20, right: 10, bottom: 20, left: 40};
        }
        width  = width - margin.left - margin.right;
        height = height - margin.top - margin.bottom;

        let svg = d3.select(divElem)
            .append("svg")
            .attr("width", width + margin.left + margin.right)
            .attr("height", height + margin.top + margin.bottom)
            .append("g")
            .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

        /////////////

        let xMin = dataPoints[0].x;
        let xMax = dataPoints[0].x;
        let yMin = dataPoints[0].y;
        let yMax = dataPoints[0].y;

        dataPoints.forEach(d => {
            if (d.x < xMin) { xMin = d.x }
            if (d.x > xMax) { xMax = d.x }
            if (d.y < yMin) { yMin = d.y }
            if (d.y > yMax) { yMax = d.y }
        });

        let dx = xMax - xMin;
        let dy = yMax - yMin;
        dx = 0.1 * dx;
        dy = 0.1 * dy;

        ////////////
        /// Axes ///
        ////////////

        let x = d3.scaleLinear();
        if(axis.x.scale !== "linear") {
            x = d3.scaleLog();
        }

        x.domain([xMin - dx, xMax + dx])
            .range([0, width]);
        let xAxis = svg.append("g")
            .attr("transform", "translate(0," + height + ")")
            .call(d3.axisBottom(x));

        /////////////

        let y = d3.scaleLinear()
        if(axis.y.scale !== "linear") {
            y = d3.scaleLog();
        }

        y.domain([yMin - dy, yMax + dy])
            .range([height, 0]);
        svg.append("g")
            .call(d3.axisLeft(y));


        //////////////
        /// Labels ///
        //////////////

        if(axis.x.label !== undefined) {
            svg.append("text")
                .attr("text-anchor", "end")
                .attr("style","font-family: dejavuSansMono; font-size: 11px;")
                .attr("x", width / 2 + margin.left)
                .attr("y", height + margin.top + 20)
                .text(axis.x.label);
        }

        /////////////

        if(axis.y.label !== undefined) {
            svg.append("text")
                .attr("text-anchor", "end")
                .attr("style","font-family: dejavuSansMono; font-size: 11px;")
                .attr("transform", "rotate(-90)")
                .attr("y", -margin.left + 10)
                .attr("x", -margin.top - height / 2 + 30)
                .text(axis.y.label);
        }


        //////////////
        /// Shapes ///
        //////////////

        let clip = svg.append("defs").append("svg:clipPath")
            .attr("id", "clip")
            .append("svg:rect")
            .attr("width", width)
            .attr("height", height)
            .attr("x", 0)
            .attr("y", 0);

        let symbol = d3.symbol();

        let scatter = svg.append('g')
            .attr("clip-path", "url(#clip)")

        /////////////

        if (points.connected === "yes") {
            scatter.append("path")
                .datum(dataPoints)
                .attr("fill", "none")
                .attr("stroke", d => "#" + (d.color || "000000") )
                .attr("stroke-width", 1.5)
                .attr("d", d3.line()
                    .x( d => x(d.x) )
                    .y( d => y(d.y) )
                )
        }

        /////////////

        scatter
            .selectAll("dataPoint")
            .data(dataPoints)
            .enter()
            .append("path")
            .attr("d", symbol.type( d => {
                if(d.shape === undefined ){ return d3.symbolCircle }
                if(d.shape === "cross"){ return d3.symbolCross
                } else if (d.shape === "diamond"){ return d3.symbolDiamond
                } else if (d.shape === "square"){ return d3.symbolSquare
                } else if (d.shape === "star"){ return d3.symbolStar
                } else if (d.shape === "triangle"){ return d3.symbolTriangle
                } else { return d3.symbolCircle }
            }))
            .attr('transform',d => "translate("+x(d.x)+","+y(d.y)+")")
            .style("fill"   , d => "#" + (d.color || "000000"))
            .style("opacity", 0.5)
            .size(d => 10 * d.size)

        /////////////

        if (points.labels === "visible") {
            scatter.selectAll("dataPoint")
                .data(dataPoints)
                .enter()
                .append("text")
                .text( d => d.label)
                .attr('transform',d => "translate("+x(d.x)+","+y(d.y)+")")
                .attr("font-size", "12px")
                .attr("fill", "black");
        }

        ////////////////
        /// Brushing ///
        ////////////////

        let brush = d3.brushX()
            .extent([[0, 0], [width, height]])
            .on("end", updateChart)


        if (points.connected !== "yes") {
            scatter
                .append("g")
                .attr("class", "brush")
                .call(brush);
        }

        let idleTimeout

        function idled() {
            idleTimeout = null;
        }

        function updateChart() {
            let extent = d3.event.selection;

            if (!extent) {
                if (!idleTimeout) return idleTimeout = setTimeout(idled, 350);
                x.domain([xMin - dx, xMax + dx]);
            } else {
                x.domain([x.invert(extent[0]), x.invert(extent[1])]);
                scatter.select(".brush").call(brush.move, null);
            }

            xAxis.transition().duration(1000).call(d3.axisBottom(x));
            scatter
                .selectAll("path")
                .transition().duration(1000)
                .attr('transform',d => "translate("+x(d.x)+","+y(d.y)+")")

            if (points.labels === "visible") {
                scatter
                    .selectAll("text")
                    .transition().duration(1000)
                    .attr('transform',d => "translate("+x(d.x)+","+y(d.y)+")")
            }
        }
    }

    createDivElem(width, height) {
        const divElem = document.createElementNS(null, "div");
        divElem.setAttributeNS(null, "class", "vis-scatterplot");
        divElem.setAttributeNS(null, "viewBox", 0 + " " + 0 + " " + width + " " + height);
        divElem.setAttributeNS(null, "width", "100%");
        divElem.setAttributeNS(null, "height", "100%");
        divElem.setAttributeNS(null, "transform", "matrix(1 0 0 -1 0 0)");

        this.dom.appendChild(divElem);
        return divElem;
    }

    setSize(size) {
        this.dom.setAttributeNS(null, "width", size[0]);
        this.dom.setAttributeNS(null, "height", size[1]);
    }
}

return ScatterPlot;
