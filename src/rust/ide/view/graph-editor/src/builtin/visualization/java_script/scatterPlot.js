function loadScript(url) {
    let script = document.createElement("script");
    script.src = url;

    document.head.appendChild(script);
}

function loadStyle(url) {
    let style   = document.createElement("link");
    style.href  = url;
    style.rel   = "stylesheet";
    style.media = "screen";
    style.type  = "text/css";

    document.head.appendChild(style);
}

function addStyleToHead(attr,stl) {
    let style       = document.createElement("style");
    style.innerText = attr + "{" + stl + "}"

    document.head.appendChild(style);
}




loadScript('https://d3js.org/d3.v4.min.js');
loadStyle('https://fontlibrary.org/face/dejavu-sans-mono')
addStyleToHead('.selection','rx: 4px;stroke: transparent;')

const label_style   = "font-family: DejaVuSansMonoBook; font-size: 10px;";
const num_width     = 30;
const lbl_padding_x = 7;
const lbl_padding_y = 2;

/**
 * A d3.js ScatterPlot visualization.
 *
 * To zoom use scrollwheel
 * To select click and swipe with LMB
 * To deselect click outside of selection with LMB
 * To pan click and swipe with RMB
 * To zoom out click "Fit all" or use key combination "ctrl+a"
 * To zoom into selection click appropriate button or use key combination "ctrl+s"
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

        let width        = this.dom.getAttributeNS(null, "width");
        let height       = this.dom.getAttributeNS(null, "height");
        const btnPadding = 25;
        height           = height - btnPadding;
        const divElem    = this.createDivElem(width, height);
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

        let zoom = this.addPanAndZoom(box_width, box_height, svg, margin, scaleAndAxis, scatter, points);

        // TODO: Visualization selector obfuscates button, so it is now on the bottom, should be on top.
        this.createButtonFitAll(scaleAndAxis, scatter, points, extremesAndDeltas, zoom, box_width);

        let selectedZoomBtn = this.createButtonScaleToPoints();

        this.addBrushing(box_width, box_height, scatter, scaleAndAxis, selectedZoomBtn, points, zoom);
    }

    addPanAndZoom(box_width, box_height, svg, margin, scaleAndAxis, scatter, points) {
        let zoom = d3.zoom().filter(function () {
            switch (d3.event.type) {
                case "mousedown": return d3.event.button === 2 || d3.event.button === 1
                case "wheel": return d3.event.button === 0
                default: return false
            }
        }).scaleExtent([.5, 20])
            .extent([[0, 0], [box_width, box_height]])
            .on("zoom", zoomed);

        let zoomElem = scatter.append("g")
            .attr("class", "zoom")
            .attr("width", box_width)
            .attr("height", box_height)
            .style("fill", "none")
            .style("pointer-events", "all")
            .call(zoom);

        function zoomed() {
            let new_xScale = d3.event.transform.rescaleX(scaleAndAxis.xScale);
            let new_yScale = d3.event.transform.rescaleY(scaleAndAxis.yScale);

            scaleAndAxis.xAxis.call(d3.axisBottom(new_xScale).ticks(box_width/num_width));
            scaleAndAxis.yAxis.call(d3.axisLeft(new_yScale));
            scatter.selectAll("path")
                .attr('transform', d => "translate(" + new_xScale(d.x) + "," + new_yScale(d.y) + ")")

            if (points.labels === "visible") {
                scatter.selectAll("text")
                    .attr("x", d => new_xScale(d.x) + lbl_padding_x)
                    .attr("y", d => new_yScale(d.y) + lbl_padding_y)
            }
        }

        return {zoomElem: zoomElem, zoom: zoom};
    }

    addBrushing(box_width, box_height, scatter, scaleAndAxis, selectedZoomBtn, points, zoom) {
        let extent;
        let brush = d3.brush()
            .extent([[0, 0], [box_width, box_height]])
            .on("start brush", updateChart)

        // The brush element must be child of zoom element - this is only way we found to have both zoom and brush
        // events working at the same time. See https://stackoverflow.com/a/59757276 .
        let brushElem = zoom.zoomElem.append("g")
            .attr("class", "brush")
            .call(brush)

        let self = this;
        function zoomIn() {
            let xMin = scaleAndAxis.xScale.invert(extent[0][0]);
            let xMax = scaleAndAxis.xScale.invert(extent[1][0]);
            let yMin = scaleAndAxis.yScale.invert(extent[1][1]);
            let yMax = scaleAndAxis.yScale.invert(extent[0][1]);

            scaleAndAxis.xScale.domain([xMin, xMax]);
            scaleAndAxis.yScale.domain([yMin, yMax]);

            self.zoomingHelper(scaleAndAxis, box_width, scatter, points);

            brushElem.call(brush.move, null);
        }

        const zoomInKeyEvent = function (event) {
            if (event.ctrlKey && event.key === 's') {
                zoomIn();
                selectedZoomBtn.style.display = "none";
            }
        };

        function updateChart() {
            let s = d3.event.selection;
            selectedZoomBtn.style.display = "inline-block";
            selectedZoomBtn.addEventListener("click",zoomIn,true)
            document.addEventListener('keydown', zoomInKeyEvent,true);
            extent = s;
        }

        const endBrushing = function (_) {
            brushElem.call(brush.move, null);
            selectedZoomBtn.style.display = "none";
            selectedZoomBtn.removeEventListener("click",zoomIn,true)
            document.removeEventListener('keydown', zoomInKeyEvent,true);
        };

        document.addEventListener('click'      , endBrushing,false);
        document.addEventListener('auxclick'   , endBrushing,false);
        document.addEventListener('contextmenu', endBrushing,false);
        document.addEventListener('scroll'     , endBrushing,false);
    }

    zoomingHelper(scaleAndAxis, box_width, scatter, points) {
        scaleAndAxis.xAxis.transition().duration(1000)
            .call(d3.axisBottom(scaleAndAxis.xScale).ticks(box_width / num_width));
        scaleAndAxis.yAxis.transition().duration(1000)
            .call(d3.axisLeft(scaleAndAxis.yScale));

        scatter.selectAll("path")
            .transition().duration(1000)
            .attr('transform', d => "translate(" + scaleAndAxis.xScale(d.x) + "," + scaleAndAxis.yScale(d.y) + ")")

        if (points.labels === "visible") {
            scatter.selectAll("text")
                .transition().duration(1000)
                .attr("x", d => scaleAndAxis.xScale(d.x) + lbl_padding_x)
                .attr("y", d => scaleAndAxis.yScale(d.y) + lbl_padding_y)
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

        let size_scale = 100

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
            }).size(d => (d.size || 1.0) * size_scale))
            .attr('transform', d => "translate(" + scaleAndAxis.xScale(d.x) + "," + scaleAndAxis.yScale(d.y) + ")")
            .style("fill", d => "#" + (d.color || "000000"))
            .style("opacity", 0.5)

        if (points.labels === "visible") {
            scatter.selectAll("dataPoint")
                .data(dataPoints)
                .enter()
                .append("text")
                .text(d => d.label)
                .attr("x", d => scaleAndAxis.xScale(d.x) + lbl_padding_x)
                .attr("y", d => scaleAndAxis.yScale(d.y) + lbl_padding_y)
                .attr("style", label_style)
                .attr("fill", "black");
        }

        return scatter;
    }

    createLabels(axis, svg, box_width, margin, box_height) {
        let fontStyle = "10px DejaVuSansMonoBook";
        if (axis.x.label !== undefined) {
            let padding_y = 20;
            svg.append("text")
                .attr("text-anchor", "end")
                .attr("style", label_style)
                .attr("x", margin.left + (this.getTextWidth(axis.x.label, fontStyle) / 2))
                .attr("y", box_height + margin.top + padding_y)
                .text(axis.x.label);
        }

        if (axis.y.label !== undefined) {
            let padding_x = 30;
            let padding_y = 15;
            svg.append("text")
                .attr("text-anchor", "end")
                .attr("style", label_style)
                .attr("transform", "rotate(-90)")
                .attr("y", -margin.left + padding_y)
                .attr("x", -margin.top - (box_height/2) + (this.getTextWidth(axis.y.label, fontStyle) / 2))
                .text(axis.y.label);
        }
    }

    getTextWidth(text, font) {
        const canvas  = document.createElement("canvas");
        const context = canvas.getContext("2d");
        context.font  = font;
        const metrics = context.measureText("  " + text);
        return metrics.width;
    }

    createAxes(axis, extremesAndDeltas, box_width, box_height, svg, focus) {
        let {domain_x, domain_y} = this.getDomains(extremesAndDeltas, focus);

        let xScale = d3.scaleLinear();
        if (axis.x.scale !== "linear") { xScale = d3.scaleLog(); }

        xScale.domain(domain_x).range([0, box_width]);
        let xAxis = svg.append("g")
            .attr("transform", "translate(0," + box_height + ")")
            .attr("style", label_style)
            .call(d3.axisBottom(xScale).ticks(box_width/num_width))

        let yScale = d3.scaleLinear()
        if (axis.y.scale !== "linear") { yScale = d3.scaleLog(); }

        yScale.domain(domain_y).range([box_height, 0]);
        let yAxis = svg.append("g")
            .attr("style", label_style)
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
            return {top: 20, right: 20, bottom: 20, left: 45};
        } else if (axis.x.label === undefined) {
            return {top: 10, right: 20, bottom: 35, left: 35};
        } else if (axis.y.label === undefined) {
            return {top: 20, right: 10, bottom: 20, left: 60};
        }
        return {top: 10, right: 10, bottom: 35, left: 60};
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

    createBtnHelper() {
        const btn = document.createElement("button");
        const style = `
            margin-left: 5px; 
            margin-bottom: 5px;
            display: inline-block;
            padding: 2px 10px;
            outline: none;
            background-color: transparent;
            border: 1px solid #333;
            color: #333;
            border-radius: 14px;
            font-size: 10px;
            vertical-align: top;
            transition: all 0.3s ease;
        `;
        btn.setAttribute("width", "80px");
        btn.setAttribute("height", "20px");
        btn.setAttribute("style", style);

        btn.onmouseover = function() {
            btn.style.backgroundColor = "#333";
            btn.style.color = "#e5e5e5";
        }

        btn.onmouseout = function() {
            btn.style.backgroundColor = "transparent";
            btn.style.color = "#333";
        }

        return btn
    }

    createButtonFitAll(scaleAndAxis, scatter, points, extremesAndDeltas, zoom, box_width) {
        const btn = this.createBtnHelper()

        let text = document.createTextNode("Fit all");
        btn.appendChild(text);

        let self = this;
        function unzoom() {
            zoom.zoomElem.transition().duration(0).call(zoom.zoom.transform, d3.zoomIdentity);

            let domain_x = [extremesAndDeltas.xMin - extremesAndDeltas.paddingX,
                extremesAndDeltas.xMax + extremesAndDeltas.paddingX];
            let domain_y = [extremesAndDeltas.yMin - extremesAndDeltas.paddingY,
                extremesAndDeltas.yMax + extremesAndDeltas.paddingY];

            scaleAndAxis.xScale.domain(domain_x);
            scaleAndAxis.yScale.domain(domain_y);

            self.zoomingHelper(scaleAndAxis, box_width, scatter, points);
        }

        document.addEventListener('keydown', function(event) {
            if (event.ctrlKey && event.key === 'a') {
                unzoom()
            }
        });

        btn.addEventListener("click",unzoom)
        this.dom.appendChild(btn);
    }

    createButtonScaleToPoints() {
        const btn = this.createBtnHelper()
        let text  = document.createTextNode("Zoom to selected");
        btn.appendChild(text);
        btn.setAttribute("width", "120px");
        btn.style.display = "none";
        this.dom.appendChild(btn);
        return btn;
    }

    setSize(size) {
        this.dom.setAttributeNS(null, "width", size[0]);
        this.dom.setAttributeNS(null, "height", size[1]);
    }
}

return ScatterPlot;
