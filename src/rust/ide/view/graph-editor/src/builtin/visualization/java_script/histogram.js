function loadScript(url) {
    let script = document.createElement("script");
    script.src = url;

    document.head.appendChild(script);
}

loadScript('https://d3js.org/d3.v4.min.js');

/**
 * A d3.js histogram visualization.
 *
 *
 * Data format (json):
 * {
 *  "axis" : {
 *     "x" : { "label" : "x-axis label", "scale" : "linear" },
 *     "y" : { "label" : "y-axis label", "scale" : "logarithmic" },
 *  },
 *  "data" : [
 *     { "x" : 0.1},
 *     ...
 *     { "x" : 0.4}
 *  ]
 * }
 */
class Histogram extends Visualization {
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

        var margin = {top: 10, right: 30, bottom: 30, left: 40},
            width      = width - margin.left - margin.right,
            height     = height - margin.top - margin.bottom;

        // append the svg object to the body of the page
        var svg = d3.select(divElem)
            .append("svg")
            .attr("width", width + margin.left + margin.right)
            .attr("height", height + margin.top + margin.bottom)
            .append("g")
            .attr("transform",
                "translate(" + margin.left + "," + margin.top + ")");

        // get the data
        d3.csv("https://raw.githubusercontent.com/holtzy/data_to_viz/master/Example_dataset/1_OneNum.csv", function(data) {

            // X axis: scale and draw:
            var x = d3.scaleLinear()
                .domain([0, d3.max(data, d => +d.price )])
                .range([0, width]);
            svg.append("g")
                .attr("transform", "translate(0," + height + ")")
                .call(d3.axisBottom(x));

            // set the parameters for the histogram
            var histogram = d3.histogram()
                .value( d => d.price )
                .domain(x.domain())
                .thresholds(x.ticks(70));

            // And apply this function to data to get the bins
            var bins = histogram(data);

            // Y axis: scale and draw:
            var y = d3.scaleLinear()
                .range([height, 0]);
            y.domain([0, d3.max(bins, d => d.length )]);
            svg.append("g")
                .call(d3.axisLeft(y));

            // append the bar rectangles to the svg element
            svg.selectAll("rect")
                .data(bins)
                .enter()
                .append("rect")
                .attr("x", 1)
                .attr("transform", d => "translate(" + x(d.x0) + "," + y(d.length) + ")" )
                .attr("width", d => x(d.x1) - x(d.x0) -1 )
                .attr("height", d => height - y(d.length) )
                .style("fill", "#69b3a2")

        });
    }

    createDivElem(width, height) {
        const divElem = document.createElementNS(null, "div");
        divElem.setAttributeNS(null, "class", "vis-histogram");
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

return Histogram;
