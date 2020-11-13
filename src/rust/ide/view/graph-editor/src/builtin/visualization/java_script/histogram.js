loadScript('https://d3js.org/d3.v4.min.js');
loadStyle('https://fontlibrary.org/face/dejavu-sans-mono')

let shortcuts = {
    zoomIn  : (e) => ((e.ctrlKey || e.metaKey) && e.key === 'z'),
    showAll : (e) => ((e.ctrlKey || e.metaKey) && e.key === 'a')
}

const LABEL_STYLE        = "font-family: DejaVuSansMonoBook; font-size: 10px;";
const X_AXIS_LABEL_WIDTH = 30;
const ANIMATION_DURATION = 1000;
const LINEAR_SCALE       = "linear";
const LIGHT_PLOT_COLOR   = "#00E890";
const DARK_PLOT_COLOR    = "#E0A63B";


/**
 * A d3.js histogram visualization.
 *
 *
 * Data format (json):
 * {
 *  "axis" : {
 *     "x" : { "label" : "x-axis label" },
 *     "y" : { "label" : "y-axis label" },
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
    static label     = "Histogram (JS)"

    onDataReceived(data) {
        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild);
        }
        this.init(data)
    }

    init(data) {
        const width         = this.dom.getAttributeNS(null,"width");
        let height          = this.dom.getAttributeNS(null,"height");
        const buttonsHeight = 25;
        height              = height - buttonsHeight;
        const divElem       = this.createDivElem(width,height);
        this.dom.appendChild(divElem);

        const parsedData = JSON.parse(data);

        const axis_scale = parsedData.axis || {x:{scale:LINEAR_SCALE},y:{scale:LINEAR_SCALE}};
        const focus      = parsedData.focus;
        const dataPoints = parsedData.data || {};
        const margin     = this.getMargins(axis_scale);
        const box_width  = width - margin.left - margin.right;
        const box_height = height - margin.top - margin.bottom;

        const svg = d3.select(this.dom)
            .append("svg")
            .attr("width",width)
            .attr("height",height)
            .append("g")
            .attr("transform","translate(" + margin.left + "," + margin.top + ")");

        const extremesAndDeltas = this.getExtremesAndDeltas(dataPoints);
        this.createLabels(axis_scale,svg,box_width,margin,box_height);
        const [bars,scale,axis] = this.createHistogram(extremesAndDeltas,box_width,svg,box_height,dataPoints,focus);
        const zoom = this.initPanAndZoom(box_width,box_height,svg,bars,margin,scale,axis,dataPoints);

        // TODO [MM]: In task specification buttons were on top of the visualization, but because
        //            the visualization selector obfuscated them, they're now on the bottom.
        //            This should be fixed in (#898).
        this.createButtonFitAll(scale,axis,svg,bars,extremesAndDeltas,box_width,zoom);
        const selectedZoomBtn = this.createButtonScaleToPoints();
        this.initBrushing(box_width,box_height,svg,bars,scale,axis,selectedZoomBtn,dataPoints,zoom);
    }

    /**
     * Adds panning and zooming functionality to the visualization.
     */
    initPanAndZoom(box_width, box_height, svg, bars, margin, scale, axis, points) {
        let zoomClass = "zoom";
        let minScale  = .5;
        let maxScale  = 20;
        const extent  = [minScale,maxScale];
        let zoom = d3.zoom().filter(function () {
            let right_button = 2
            let mid_button   = 1
            let scroll_wheel = 0
            switch (d3.event.type) {
                case "mousedown": return d3.event.button === right_button || d3.event.button === mid_button
                case "wheel": return d3.event.button === scroll_wheel
                default: return false
            }
        }).scaleExtent(extent)
            .extent([[0,0],[box_width,box_height]])
            .on(zoomClass,zoomed)

        let zoomElem = svg.append("g")
            .attr("class",zoomClass)
            .attr("width",box_width)
            .attr("height",box_height)
            .style("fill","none")
            .style("pointer-events","all")
            .call(zoom);

        const self = this;
        /**
         * Helper function called on pan/scroll.
         */
        function zoomed() {
            console.log("ZOOMED");
            scale.zoom = d3.event.transform.k;
            let tmpScale = Object.assign({}, scale);
            tmpScale.x = d3.event.transform.rescaleX(scale.x);
            self.rescale(tmpScale,axis,box_width,bars,false)
        }

        return {zoomElem,zoom};
    }

    /**
     * Adds brushing functionality to the plot.
     *
     * Brush is a tool which enables user to select points, and zoom into selection via
     * keyboard shortcut or button event.
     */
    initBrushing(box_width, box_height, svg, bars, scale, axis, selectedZoomBtn, points, zoom) {
        let extent;
        let brushClass = "brush";

        let brush = d3.brushX()
            .extent([[0,0],[box_width,box_height]])
            .on("start " + brushClass,updateChart)

        // The brush element must be child of zoom element - this is only way we found to have both zoom and brush
        // events working at the same time. See https://stackoverflow.com/a/59757276 .
        let brushElem = zoom.zoomElem.append("g")
            .attr("class",brushClass)
            .call(brush)

        let self = this;

        /**
         * Zooms into selected fragment of plot.
         *
         * Based on https://www.d3-graph-gallery.com/graph/interactivity_brush.html
         * Section "Brushing for zooming".
         */
        const zoomIn = () => {
            console.log("ZOOMIN");
            let xMin = scale.x.invert(extent[0]);
            let xMax = scale.x.invert(extent[1]);

            scale.x.domain([xMin,xMax]);
            console.log(extent, box_width);
            const dx = (extent[1]-extent[0]);
            scale.zoom = scale.zoom * (box_width / dx);

            self.rescale(scale,axis,box_width,bars,points,true);
        }

        const zoomInKeyEvent = (event) => {
            if (shortcuts.zoomIn(event)) {
                zoomIn();
                endBrushing();
            }
        };

        /**
         * Updates plot when brushing.
         */
        function updateChart() {
            let selectionEvent            = d3.event.selection;
            selectedZoomBtn.style.display = "inline-block";
            selectedZoomBtn.addEventListener("click",zoomIn,true)
            document.addEventListener('keydown',zoomInKeyEvent,true);
            extent = selectionEvent;
        }

        /**
         * Removes brush, keyboard event and zoom button when end event is captured.
         */
        const endBrushing = () => {
            brushElem.call(brush.move,null);
            selectedZoomBtn.style.display = "none";
            selectedZoomBtn.removeEventListener("click",zoomIn,true)
            document.removeEventListener('keydown',zoomInKeyEvent,true);
        };

        let endEvents = ['click','auxclick','contextmenu','scroll']
        endEvents.forEach(e => document.addEventListener(e,endBrushing,false));
    }

    /**
     * Helper function for zooming in after the scale has been updated.
     */
    rescale(scale, axis, box_width, bars, with_animation) {
        const animation_duration = (with_animation ? ANIMATION_DURATION : 0.0);
        console.log(animation_duration);
        axis.x.transition().duration(animation_duration)
            .call(d3.axisBottom(scale.x).ticks(box_width / X_AXIS_LABEL_WIDTH));
        bars
            .transition().duration(animation_duration)
            .attr('transform',d => "translate(" + scale.x(d.x0) + "," + scale.y(d.length) + ")scale(" + scale.zoom + ",1)")
    }

    createHistogram(extremesAndDeltas,box_width,svg,box_height,dataPoints,focus) {
        let domain_x = [extremesAndDeltas.xMin - extremesAndDeltas.paddingX,
            extremesAndDeltas.xMax + extremesAndDeltas.paddingX];

        if (focus !== undefined) {
            if (focus.x !== undefined && focus.zoom !== undefined) {
                let padding_x = extremesAndDeltas.dx * (1 / (2 * (focus.zoom)));
                domain_x = [focus.x - padding_x,focus.x + padding_x];
            }
        }

        let x = d3.scaleLinear()
            .domain(domain_x)
            .range([0,box_width]);
        let xAxis = svg.append("g")
            .attr("transform","translate(0," + box_height + ")")
            .call(d3.axisBottom(x));

        let histogram = d3.histogram()
            .value(d => d.x)
            .domain(x.domain())
            .thresholds(x.ticks(3)); // FIXME

        let bins = histogram(dataPoints);

        let y = d3.scaleLinear()
            .range([box_height,0]);
        y.domain([0,d3.max(bins,d => d.length)]);
        let yAxis = svg.append("g")
            .call(d3.axisLeft(y));

        let accentColor = LIGHT_PLOT_COLOR;
        if (document.getElementById("root").classList.contains("dark")){
            accentColor = DARK_PLOT_COLOR;
        }

        let bars = svg.
             append("g")
            .attr('clip-path','url(#hist-clip-path)')
            .selectAll("rect")
            .data(bins)
            .enter()
            .append("rect")
            .attr("x", 1)
            .attr("transform",d => "translate(" + x(d.x0) + "," + y(d.length) + ")")
            .attr("width",d => x(d.x1) - x(d.x0))
            .attr("height",d => box_height - y(d.length))
            .style("fill",accentColor)

        // Create clip path
        let defs = svg.append('defs')
        defs.append('clipPath')
            .attr('id', 'hist-clip-path')
            .append('rect')
            .attr('width', box_width)
            .attr('height', box_height)


        return [bars,{x:x,y:y,zoom:1.0},{x:xAxis,y:yAxis}];
    }

    /**
     * Creates labels on axes if they're defined.
     */
    createLabels(axis,svg,box_width,margin,box_height) {
        let fontStyle = "10px DejaVuSansMonoBook";
        if (axis.x.label !== undefined) {
            let padding_y = 20;
            svg.append("text")
                .attr("text-anchor","end")
                .attr("style",LABEL_STYLE)
                .attr("x",margin.left + (this.getTextWidth(axis.x.label,fontStyle) / 2))
                .attr("y",box_height + margin.top + padding_y)
                .text(axis.x.label);
        }

        if (axis.y.label !== undefined) {
            let padding_y = 15;
            svg.append("text")
                .attr("text-anchor","end")
                .attr("style",LABEL_STYLE)
                .attr("transform","rotate(-90)")
                .attr("y",-margin.left + padding_y)
                .attr("x",-margin.top - (box_height/2) + (this.getTextWidth(axis.y.label,fontStyle) / 2))
                .text(axis.y.label);
        }
    }

    /**
     * Helper function to get text width to make sure that labels on x axis wont overlap,
     * and keeps it readable.
     */
    getTextWidth(text,font) {
        const canvas  = document.createElement("canvas");
        const context = canvas.getContext("2d");
        context.font  = font;
        const metrics = context.measureText("  " + text);
        return metrics.width;
    }

    /**
     * Helper function calculating extreme values and paddings to make sure data will fit nicely.
     *
     * It traverses through data getting minimal and maximal values, and calculates padding based on
     * span calculated from above values, multiplied by 10% so that the plot is a little bit smaller
     * than the container.
     */
    getExtremesAndDeltas(dataPoints) {
        let xMin = dataPoints[0].x;
        let xMax = dataPoints[0].x;

        dataPoints.forEach(d => {
            if (d.x < xMin) { xMin = d.x }
            if (d.x > xMax) { xMax = d.x }
        });

        let dx        = xMax - xMin;
        let padding_x = 0.1 * dx;

        return {xMin:xMin,xMax:xMax,paddingX:padding_x,dx:dx};
    }

    /**
     * Helper function getting margins for plot's box.
     */
    getMargins(axis) {
        if (axis.x.label === undefined && axis.y.label === undefined) {
            return {top:20,right:20,bottom:20,left:20};
        } else if (axis.y.label === undefined) {
            return {top:10,right:20,bottom:35,left:20};
        } else if (axis.x.label === undefined) {
            return {top:20,right:10,bottom:20,left:45};
        }
        return {top:10,right:10,bottom:35,left:45};
    }

    /**
     * Creates HTML div element as container for plot.
     */
    createDivElem(width,height) {
        const divElem = document.createElementNS(null,"div");
        divElem.setAttributeNS(null,"class","vis-scatterplot");
        divElem.setAttributeNS(null,"viewBox",0 + " " + 0 + " " + width + " " + height);
        divElem.setAttributeNS(null,"width","100%");
        divElem.setAttributeNS(null,"height","100%");
        divElem.setAttributeNS(null,"transform","matrix(1 0 0 -1 0 0)");

        const addStyleToElem = (attr,stl) => {
            let style       = document.createElement("style");
            style.innerText = attr + "{" + stl + "}"

            divElem.appendChild(style);
        }

        let darkStrokeColor   = `rgba(255,255,255,0.7)`;
        let buttonLightColor  = `#333`;
        let darkBtnHoverColor = `rgba(255,255,255,0.5)`;
        let darkSelectionFill = `#efefef`;

        addStyleToElem('.selection','rx: 4px;stroke: transparent;')
        addStyleToElem('button',`
            margin-left: 5px; 
            margin-bottom: 5px;
            display: inline-block;
            padding: 2px 10px;
            outline: none;
            background-color: transparent;
            border: 1px solid ${buttonLightColor};
            color: ${buttonLightColor};
            border-radius: 14px;
            font-size: 10px;
            font-family: DejaVuSansMonoBook;
            vertical-align: top;
            transition: all 0.3s ease;
        `)
        addStyleToElem('button:hover',`
            background-color: ${buttonLightColor};
            color: ${darkSelectionFill};
        `)

        addStyleToElem('.dark button',`
            border: 0;
            background-color: ${darkStrokeColor};
        `)
        addStyleToElem('.dark button:hover',`
            background-color: ${darkBtnHoverColor};
        `)
        addStyleToElem('.dark .selection',`fill: ${darkSelectionFill}`)
        addStyleToElem('.dark line',`stroke: ${darkStrokeColor};`)
        addStyleToElem('.dark .domain',`stroke: ${darkStrokeColor};`)
        addStyleToElem('.dark text',`fill: ${darkStrokeColor};`)

        return divElem;
    }

    /**
     * Helper function for button creation.
     */
    createButtonElement() {
        const btn = document.createElement("button");
        btn.setAttribute("width","80px");
        btn.setAttribute("height","20px");
        return btn
    }

    /**
     * Creates a button to fit all points on plot.
     */
    createButtonFitAll(scale,axis,svg,bars,extremesAndDeltas,box_width,zoom) {
        const btn = this.createButtonElement()

        let text = document.createTextNode("Fit all");
        btn.appendChild(text);

        let self = this;
        const unzoom = () => {
            zoom.zoomElem.transition().duration(0).call(zoom.zoom.transform,d3.zoomIdentity);

            let domain_x = [extremesAndDeltas.xMin - extremesAndDeltas.paddingX,
                extremesAndDeltas.xMax + extremesAndDeltas.paddingX];

            scale.x.domain(domain_x);
            scale.zoom = 1.0;
            self.rescale(scale,axis,box_width,bars,true);
        }

        document.addEventListener('keydown',e => {
            if (shortcuts.showAll(e)) { unzoom() }
        });

        btn.addEventListener("click",unzoom)
        this.dom.appendChild(btn);
    }

    /**
     * Creates a button to zoom into brushed fragment of plot.
     */
    createButtonScaleToPoints() {
        const btn = this.createButtonElement()
        let text  = document.createTextNode("Zoom to selected");
        btn.appendChild(text);
        btn.setAttribute("width","120px");
        btn.style.display = "none";
        this.dom.appendChild(btn);
        return btn;
    }

    /**
     * Sets size of this DOM object.
     */
    setSize(size) {
        this.dom.setAttributeNS(null,"width",size[0]);
        this.dom.setAttributeNS(null,"height",size[1]);
    }
}

return Histogram;
