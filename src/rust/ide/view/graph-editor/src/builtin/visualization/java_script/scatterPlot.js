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
        let height    = this.dom.getAttributeNS(null, "height") - 25;
        const divElem = this.createDivElem(width, height);
        this.dom.appendChild(divElem);

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

        let extremesAndDeltas = this.getExtremesAndDeltas(dataPoints);

        let scaleAndAxis = this.createAxes(axis, extremesAndDeltas, box_width, box_height, svg, focus);

        this.createLabels(axis, svg, box_width, margin, box_height);

        let scatter = this.createScatter(svg, box_width, box_height, points, dataPoints, scaleAndAxis);

        this.addBrushing(box_width, box_height, scatter, scaleAndAxis);

        // TODO: Visualization selector obfuscates button, so it is now on the bottom, should be on top.
        this.createButtonUnzoom(scaleAndAxis, scatter, points, extremesAndDeltas);

        this.addPanAndZoom(box_width, box_height, svg, margin, scaleAndAxis, scatter, points);
    }

    addPanAndZoom(box_width, box_height, svg, margin, scaleAndAxis, scatter, points) {
        let zoom = d3.zoom().filter(function () {
            switch (d3.event.type) {
                case "mousedown": return d3.event.button === 2
                case "wheel": return d3.event.button === 0
                default: return false
            }
        }).scaleExtent([.5, 20])
            .extent([[0, 0], [box_width, box_height]])
            .on("zoom", zoomed);

        svg.append("rect")
            .attr("width", box_width)
            .attr("height", box_height)
            .style("fill", "none")
            .style("pointer-events", "all")
            .attr('transform', 'translate(' + margin.left + ',' + margin.top + ')')
            .call(zoom);

        function zoomed() {
            let new_xScale = d3.event.transform.rescaleX(scaleAndAxis.xScale);
            let new_yScale = d3.event.transform.rescaleY(scaleAndAxis.yScale);

            scaleAndAxis.xAxis.call(d3.axisBottom(new_xScale).ticks(7));
            scaleAndAxis.yAxis.call(d3.axisLeft(new_yScale).ticks(7));
            scatter.selectAll("path")
                .attr('transform', d => "translate(" + new_xScale(d.x) + "," + new_yScale(d.y) + ")")

            if (points.labels === "visible") {
                scatter.selectAll("text")
                    .attr('transform', d => "translate(" + new_xScale(d.x) + "," + new_yScale(d.y) + ")")
            }
        }
    }

    addBrushing(box_width, box_height, scatter, scaleAndAxis) {
        let brush = d3.brush()
            .extent([[0, 0], [box_width, box_height]])
            .on("start brush", updateChart)

        scatter.append("g")
            .attr("class", "brush")
            .call(brush)

        function updateChart() {
            let extent = d3.event.selection
            scatter.classed("selected", d => isBrushed(extent, scaleAndAxis.xScale(d.x), scaleAndAxis.yScale(d.y) ))
        }

        function isBrushed(brush_coords, cx, cy) {
            var x0 = brush_coords[0][0],
                x1 = brush_coords[1][0],
                y0 = brush_coords[0][1],
                y1 = brush_coords[1][1];
            return x0 <= cx && cx <= x1 && y0 <= cy && cy <= y1;
        }
    }

    createScatter(svg, box_width, box_height, points, dataPoints, scaleAndAxis) {
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
            .attr('transform', d => "translate(" + scaleAndAxis.xScale(d.x) + "," + scaleAndAxis.yScale(d.y) + ")")
            .style("fill", d => "#" + (d.color || "000000"))
            .style("opacity", 0.5)
            .size(d => 10 * d.size)

        if (points.labels === "visible") {
            scatter.selectAll("dataPoint")
                .data(dataPoints)
                .enter()
                .append("text")
                .text(d => d.label)
                .attr('transform', d => "translate(" + scaleAndAxis.xScale(d.x) + "," + scaleAndAxis.yScale(d.y) + ")")
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

    createAxes(axis, extremesAndDeltas, box_width, box_height, svg, focus) {
        let {domain_x, domain_y} = this.getDomains(extremesAndDeltas, focus);

        let xScale = d3.scaleLinear();
        if (axis.x.scale !== "linear") { xScale = d3.scaleLog(); }

        xScale.domain(domain_x).range([0, box_width]);
        let xAxis = svg.append("g")
            .attr("transform", "translate(0," + box_height + ")")
            .call(d3.axisBottom(xScale).ticks(7))

        let yScale = d3.scaleLinear()
        if (axis.y.scale !== "linear") { yScale = d3.scaleLog(); }

        yScale.domain(domain_y).range([box_height, 0]);
        let yAxis = svg.append("g")
            .call(d3.axisLeft(yScale));
        return {xScale: xScale, yScale: yScale, xAxis: xAxis, yAxis: yAxis};
    }

    getDomains(extremesAndDeltas, focus) {
        let domain_x = [extremesAndDeltas.xMin - extremesAndDeltas.paddingX,
            extremesAndDeltas.xMax + extremesAndDeltas.paddingX];
        let domain_y = [extremesAndDeltas.yMin - extremesAndDeltas.paddingY,
            extremesAndDeltas.yMax + extremesAndDeltas.paddingY];

        if (focus !== undefined) {
            if (focus.x !== undefined && focus.y !== undefined && focus.zoom !== undefined) {
                let padding_x = extremesAndDeltas.dx * (1 / (2 * (focus.zoom)));
                let padding_y = extremesAndDeltas.dy * (1 / (2 * (focus.zoom)));
                domain_x = [focus.x - padding_x, focus.x + padding_x];
                domain_y = [focus.y - padding_y, focus.y + padding_y];
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

        return {xMin: xMin, xMax: xMax, yMin: yMin, yMax: yMax, paddingX: padding_x, paddingY: padding_y, dx: dx, dy: dy};
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

        return divElem;
    }

    createButtonUnzoom(scaleAndAxis, scatter, points, extremesAndDeltas) {
        const btn = document.createElement("button");
        btn.setAttribute("width", "80px");
        btn.setAttribute("height", "20px");
        btn.setAttribute("style", "margin-left: 5px; margin-top: 5px;");

        var text = document.createTextNode("Fit all");
        btn.appendChild(text);

        function unzoom() {
            let domain_x = [extremesAndDeltas.xMin - extremesAndDeltas.paddingX,
                extremesAndDeltas.xMax + extremesAndDeltas.paddingX];
            let domain_y = [extremesAndDeltas.yMin - extremesAndDeltas.paddingY,
                extremesAndDeltas.yMax + extremesAndDeltas.paddingY];

            scaleAndAxis.xScale.domain(domain_x);
            scaleAndAxis.yScale.domain(domain_y);

            scaleAndAxis.xAxis.transition().duration(1000)
                .call(d3.axisBottom(scaleAndAxis.xScale).ticks(7));
            scaleAndAxis.yAxis.transition().duration(1000)
                .call(d3.axisLeft(scaleAndAxis.yScale));

            scatter.selectAll("path")
                .transition().duration(1000)
                .attr('transform', d => "translate(" + scaleAndAxis.xScale(d.x) + "," + scaleAndAxis.yScale(d.y) + ")")

            if (points.labels === "visible") {
                scatter.selectAll("text")
                    .transition().duration(1000)
                    .attr('transform', d => "translate(" + scaleAndAxis.xScale(d.x) + "," + scaleAndAxis.yScale(d.y) + ")")
            }
        }

        btn.addEventListener("click",unzoom)
        this.dom.appendChild(btn);
    }

    setSize(size) {
        this.dom.setAttributeNS(null, "width", size[0]);
        this.dom.setAttributeNS(null, "height", size[1]);
    }
}

return ScatterPlot;
