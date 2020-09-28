function loadScript(url) {
    let script = document.createElement("script");
    script.src = url;

    document.head.appendChild(script);
}

loadScript('https://d3js.org/d3.v4.min.js');

const label_style = "font-family: dejavuSansMono; font-size: 11px;";

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
 *     "labels" : "visible" | "invisible"
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
    static label     = "Scatter Plot (JS)"

    onDataReceived(data) {
        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild);
        }

        let width     = this.dom.getAttributeNS(null, "width");
        let height    = this.dom.getAttributeNS(null, "height");
        const divElem = this.createDivElem(width, height);

        let parsedData = JSON.parse(data);
        let axis       = parsedData.axis || {x: {scale: "linear" }, y: {scale: "linear" }};
        let focus      = parsedData.focus;
        let points     = parsedData.points || {labels: "invisible"};
        let dataPoints = parsedData.data || {};

        let margin     = this.getMargins(axis);
        let box_width  = width - margin.left - margin.right;
        let box_height = height - margin.top - margin.bottom;

        // FIXME : SVG eagerly gets all pointer events from top of it, even if
        //         node overlaps it. Should be debugged with (#801).
        let svg = d3.select(divElem)
            .append("svg")
            .attr("width", width)
            .attr("height", height)
            .append("g")
            .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

        let {xMin, xMax, yMin, yMax, padding_x, padding_y, dx, dy} = this.getExtremesAndDeltas(dataPoints);

        let {x, y, xAxis, yAxis} = this.createAxes(axis, xMin, padding_x, xMax, box_width, svg, box_height, yMin, padding_y, yMax, focus, dx, dy);

        this.createLabels(axis, svg, box_width, margin, box_height);

        let scatter = this.createScatter(svg, box_width, box_height, points, dataPoints, x, y);

        this.addBrushing(box_width, box_height, scatter, x, y);
    }

    addBrushing(box_width, box_height, scatter, x, y) {
        let brush = d3.brush()
            .extent([[0, 0], [box_width, box_height]])
            .on("start brush", updateChart)

        scatter
            .append("g")
            .attr("class", "brush")
            .call(brush);

        function updateChart() {
            extent = d3.event.selection
            scatter.classed("selected", d => isBrushed(extent, x(d.x), y(d.y) ))
        }

        function isBrushed(brush_coords, cx, cy) {
            var x0 = brush_coords[0][0],
                x1 = brush_coords[1][0],
                y0 = brush_coords[0][1],
                y1 = brush_coords[1][1];
            return x0 <= cx && cx <= x1 && y0 <= cy && cy <= y1;
        }
    }

    createScatter(svg, box_width, box_height, points, dataPoints, x, y) {
        let clip = svg.append("defs").append("svg:clipPath")
            .attr("id", "clip")
            .append("svg:rect")
            .attr("width", box_width)
            .attr("height", box_height)
            .attr("x", 0)
            .attr("y", 0);

        let symbol = d3.symbol();

        let scatter = svg.append('g')
            .attr("clip-path", "url(#clip)")

        scatter
            .selectAll("dataPoint")
            .data(dataPoints)
            .enter()
            .append("path")
            .attr("d", symbol.type(d => {
                if (d.shape === undefined)       { return d3.symbolCircle   }
                if (d.shape === "cross")         { return d3.symbolCross    }
                else if (d.shape === "diamond")  { return d3.symbolDiamond  }
                else if (d.shape === "square")   { return d3.symbolSquare   }
                else if (d.shape === "star")     { return d3.symbolStar     }
                else if (d.shape === "triangle") { return d3.symbolTriangle }
                else                             { return d3.symbolCircle   }
            }))
            .attr('transform', d => "translate(" + x(d.x) + "," + y(d.y) + ")")
            .style("fill", d => "#" + (d.color || "000000"))
            .style("opacity", 0.5)
            .size(d => 10 * d.size)

        if (points.labels === "visible") {
            scatter.selectAll("dataPoint")
                .data(dataPoints)
                .enter()
                .append("text")
                .text(d => d.label)
                .attr('transform', d => "translate(" + x(d.x) + "," + y(d.y) + ")")
                .attr("style", label_style)
                .attr("fill", "black");
        }

        return scatter;
    }

    createLabels(axis, svg, box_width, margin, box_height) {
        if (axis.x.label !== undefined) {
            let padding_y = 20;
            svg.append("text")
                .attr("text-anchor", "end")
                .attr("style", label_style)
                .attr("x", box_width / 2 + margin.left)
                .attr("y", box_height + margin.top + padding_y)
                .text(axis.x.label);
        }

        if (axis.y.label !== undefined) {
            let padding_x = 30;
            let padding_y = 10;
            svg.append("text")
                .attr("text-anchor", "end")
                .attr("style", label_style)
                .attr("transform", "rotate(-90)")
                .attr("y", -margin.left + padding_y)
                .attr("x", -margin.top - box_height / 2 + padding_x)
                .text(axis.y.label);
        }
    }

    createAxes(axis, xMin, padding_x, xMax, box_width, svg, box_height, yMin, padding_y, yMax, focus, dx, dy) {
        let {domain_x, domain_y} = this.getDomains(xMin, padding_x, xMax, yMin, padding_y, yMax, focus, dx, dy);

        let x = d3.scaleLinear();
        if (axis.x.scale !== "linear") { x = d3.scaleLog(); }

        x.domain(domain_x)
            .range([0, box_width]);
        let xAxis = svg.append("g")
            .attr("transform", "translate(0," + box_height + ")")
            .call(d3.axisBottom(x))
            .selectAll("text")
            .attr("transform", "translate(-10,5)rotate(-45)")

        let y = d3.scaleLinear()
        if (axis.y.scale !== "linear") { y = d3.scaleLog(); }

        y.domain(domain_y)
            .range([box_height, 0]);
        let yAxis = svg.append("g")
            .call(d3.axisLeft(y));
        return {x, y, xAxis, yAxis};
    }

    getDomains(xMin, padding_x, xMax, yMin, padding_y, yMax, focus, dx, dy) {
        let domain_x = [xMin - padding_x, xMax + padding_x];
        let domain_y = [yMin - padding_y, yMax + padding_y];
        if (focus !== undefined) {
            if (focus.x !== undefined && focus.y !== undefined && focus.zoom !== undefined) {
                let delta_x = dx * (1 / (2 * (focus.zoom)));
                let delta_y = dy * (1 / (2 * (focus.zoom)));
                domain_x = [focus.x - delta_x, focus.x + delta_x];
                domain_y = [focus.y - delta_y, focus.y + delta_y];
            }
        }
        return {domain_x, domain_y};
    }

    getExtremesAndDeltas(dataPoints) {
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

        let padding_x = 0.1 * dx;
        let padding_y = 0.1 * dy;

        return {xMin, xMax, yMin, yMax, padding_x, padding_y, dx, dy};
    }

    getMargins(axis) {
        if (axis.x.label === undefined && axis.y.label === undefined) {
            return {top: 20, right: 20, bottom: 20, left: 20};
        } else if (axis.x.label === undefined) {
            return {top: 10, right: 20, bottom: 35, left: 20};
        } else if (axis.y.label === undefined) {
            return {top: 20, right: 10, bottom: 20, left: 40};
        }
        return {top: 10, right: 10, bottom: 35, left: 40};
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
