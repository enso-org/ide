function loadScript(url) {
    var script = document.createElement("script");
    script.src = url;

    document.head.appendChild(script);
}

loadScript('https://d3js.org/d3.v4.min.js');

/**
 * A d3.js ScatterPlot visualization.
 *
 * Data format (json):
 * {
 *  "axis" : {
 *     "x" : { "label" : "x-axis label", "scale" : "linear" },
 *     "y" : { "label" : "y-axis label", "scale" : "logarithmic" },
 *  },
 *  "focus" { "x" : 1.7, "y" : 2.1, "zoom" : 3.0 },
 *  "points" : {
 *     "labels" : "visible"
 *  }
 *  "data" : [
 *     { "x" : 0.1, "y" : 0.7, "label" : "foo", "color" : "rgb(1.0,0.0,0.0)", "shape" : "circle", "size" : 0.2 },
 *     ...
 *     { "x" : 0.4, "y" : 0.2, "label" : "baz", "color" : "rgb(0.0,0.0,1.0)", "shape" : "square", "size" : 0.3 }
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

        ////////////////////////////////////////////////////////////////////////
        console.log("-----===== READING =====-----")

        let parsedData  = JSON.parse(data);
        console.log(parsedData);

        let axis = parsedData.axis || {x: {scale: "linear" }, y: {scale: "linear" }};

        let focus = parsedData.focus;
        console.log(focus);

        let points = parsedData.points || {labels: "invisible"};
        console.log(points);

        let dataPoints = parsedData.data || {};
        console.log(dataPoints);

        ///////////
        /// Box ///
        ///////////

        let margin = {top: 10, right: 10, bottom: 35, left: 40};
        if (axis.x.label === undefined && axis.y.label === undefined) {
            margin = {top: 20, right: 20, bottom: 20, left: 20};
        }
        width      = width - margin.left - margin.right;
        height     = height - margin.top - margin.bottom;

        let svg = d3.select(divElem)
            .append("svg")
            .attr("width", width + margin.left + margin.right)
            .attr("height", height + margin.top + margin.bottom)
            .append("g")
            .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

        ////////////////////////////////////////////////////////////////////////

        ////////////
        /// Axes ///
        ////////////

        var x = d3.scaleLinear();
        if(axis.x.scale !== "linear") {
            x = d3.scaleLog();
        }

        x.domain([0, 1]) // read domain as minX-maxX
            .range([0, width]);
        svg.append("g")
            .attr("transform", "translate(0," + height + ")")
            .call(d3.axisBottom(x));

        /////////////

        var y = d3.scaleLinear()
        if(axis.y.scale !== "linear") {
            y = d3.scaleLog();
        }

        y.domain([0, 1]) // read domain as minY-maxY
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

        var clip = svg.append("defs").append("svg:clipPath")
            .attr("id", "clip")
            .append("svg:rect")
            .attr("width", width)
            .attr("height", height)
            .attr("x", 0)
            .attr("y", 0);

        var scatter = svg.append('g')
            .attr("clip-path", "url(#clip)")

        // FIXME
        scatter
            .selectAll("circle")
            .data(parsedData.dataPoints)
            .enter()
            .append("circle")
            .attr("cx", function (d) {
                return x(d.x);
            })
            .attr("cy", function (d) {
                return y(d.y);
            })
            .attr("r", function (d) {
                return 10 * d.size;
            })
            .style("fill", function (d) {
                return color(d.color)
            })
            .style("opacity", 0.5)
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
