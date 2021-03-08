loadScript('https://d3js.org/d3.v4.min.js')
loadStyle('https://fontlibrary.org/face/dejavu-sans-mono')

let shortcuts = {
    zoomIn: e => (e.ctrlKey || e.metaKey) && e.key === 'z',
    showAll: e => (e.ctrlKey || e.metaKey) && event.key === 'a',
}

const label_style = 'font-family: DejaVuSansMonoBook; font-size: 10px;'
const x_axis_label_width = 30
const point_label_padding_x = 7
const point_label_padding_y = 2
const animation_duration = 1000
const linear_scale = 'linear'
const visilbe_points = 'visible'
const buttonsHeight = 25

/**
 * A d3.js ScatterPlot visualization.
 *
 * To zoom use scrollwheel
 * To select click and swipe with LMB
 * To deselect click outside of selection with LMB
 * To pan click and swipe with RMB
 * To zoom out click "Fit all" or use key combination "ctrl/cmd+a"
 * To zoom into selection click appropriate button or use key combination "ctrl/cmd+z"
 *
 * Data format (json):
 * {
 *  "axis":{
 *     "x":{"label":"x-axis label""scale":"linear"},
 *     "y":{"label":"y-axis label","scale":"logarithmic"},
 *  },
 *  "focus":{"x":1.7,"y":2.1,"zoom":3.0},
 *  "points":{"labels":"visible" | "invisible"},
 *  "data":[
 *     {"x":0.1,"y":0.7,"label":"foo","color":"FF0000","shape":"circle","size":0.2},
 *     ...
 *     {"x":0.4,"y":0.2,"label":"baz","color":"0000FF","shape":"square","size":0.3}
 *  ]
 * }
 */
class ScatterPlot extends Visualization {
    static inputType = 'Any'
    static label = 'Scatter Plot'

    /**
     * Presents a scatterplot visualization after receiving `data`.
     */
    onDataReceived(data) {
        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild)
        }

        const divElem = this.createDivElem(this.canvasWidth(), this.canvasHeight())
        this.dom.appendChild(divElem)

        let parsedData = this.parseData(data)
        this.updateState(parsedData)

        let svg = d3
            .select(divElem)
            .append('svg')
            .attr('width', this.canvasWidth())
            .attr('height', this.canvasHeight())
            .append('g')
            .attr('transform', 'translate(' + this.margin.left + ',' + this.margin.top + ')')

        let extremesAndDeltas = this.getExtremesAndDeltas(this.dataPoints)
        let scaleAndAxis = this.createAxes(
            this.axis,
            extremesAndDeltas,
            this.box_width,
            this.box_height,
            svg,
            focus
        )
        this.createLabels(this.axis, svg, this.box_width, this.margin, this.box_height)
        let scatter = this.createScatter(
            svg,
            this.box_width,
            this.box_height,
            this.points,
            this.dataPoints,
            scaleAndAxis
        )
        let zoom = this.addPanAndZoom(
            this.box_width,
            this.box_height,
            svg,
            this.margin,
            scaleAndAxis,
            scatter,
            this.points
        )

        // TODO [MM]: In task specification buttons were on top of the visualization, but because
        //            the visualization selector obfuscated them, they're now on the bottom.
        //            This should be fixed in (#898).
        this.createButtonFitAll(
            scaleAndAxis,
            scatter,
            this.points,
            extremesAndDeltas,
            zoom,
            this.box_width
        )
        let selectedZoomBtn = this.createButtonScaleToPoints()
        this.addBrushing(
            this.box_width,
            this.box_height,
            scatter,
            scaleAndAxis,
            selectedZoomBtn,
            this.points,
            zoom
        )
    }

    canvasWidth() {
        return this.dom.getAttributeNS(null, 'width')
    }

    canvasHeight() {
        let height = this.dom.getAttributeNS(null, 'height')
        return height - buttonsHeight
    }

    updateState(parsedData) {
        this.axis = parsedData.axis || {
            x: { scale: linear_scale },
            y: { scale: linear_scale },
        }
        this.focus = parsedData.focus
        this.points = parsedData.points || { labels: 'invisible' }
        this.dataPoints = this.extractValues(parsedData)

        this.margin = this.getMargins(this.axis)
        this.box_width = this.canvasWidth() - this.margin.left - this.margin.right
        this.box_height = this.canvasHeight() - this.margin.top - this.margin.bottom
    }

    extractValues(data) {
        /// Note this is a workaround for #1006, we allow raw arrays and JSON objects to be consumed.
        if (Array.isArray(data)) {
            return this.arrayAsValues(data)
        }
        return data.data || {}
    }

    /**
     * Return a array of values as valid scatterplot input.
     * Uses the values of the array as y-coordinate and the index of the value as its x-coordinate.
     */
    arrayAsValues(dataArray) {
        return dataArray.map((y, ix) => {
            return { x: ix, y }
        })
    }

    parseData(data) {
        let parsedData
        /// Note this is a workaround for #1006, we allow raw arrays and JSON objects to be consumed.
        if (typeof data === 'string' || data instanceof String) {
            parsedData = JSON.parse(data)
        } else {
            parsedData = data
        }
        return parsedData
    }

    /**
     * Adds panning and zooming functionality to the visualization.
     */
    addPanAndZoom(box_width, box_height, svg, margin, scaleAndAxis, scatter, points) {
        let zoomClass = 'zoom'
        let minScale = 0.5
        let maxScale = 20
        const extent = [minScale, maxScale]
        let zoom = d3
            .zoom()
            .filter(function () {
                console.log('filter', d3.event)
                switch (d3.event.type) {
                    case 'mousedown':
                        return d3.event.button === right_button || d3.event.button === mid_button
                    case 'wheel':
                        return d3.event.button === scroll_wheel
                    default:
                        return false
                }
            })
            .wheelDelta(function () {
                const event = d3.event
                // This is the delta used by default in wheelData and which is modified by us.
                const defaultWheelDelta =
                    -event.deltaY * (event.deltaMode === 1 ? 0.05 : event.deltaMode ? 1 : 0.002)
                return defaultWheelDelta * (event.ctrlKey ? 5 : 1)
            })
            .scaleExtent(extent)
            .extent([
                [0, 0],
                [box_width, box_height],
            ])
            .on('zoom', zoomed)
            // .on('mousewheel', zoomed)
            .on('start', start_zoom)

        const zoomElem = scatter
            .append('g')
            .attr('class', 'zoom')
            .attr('width', box_width)
            .attr('height', box_height)
            .style('fill', 'none')
            .style('pointer-events', 'all')
            .call(zoom)

        /**
         * Helper function called on pan/scroll.
         */
        function zoomed() {
            let new_xScale
            let new_yScale
            if (d3.event.sourceEvent.buttons === right_button) {
                const zoom_amount = rmb_zoom_value(d3.event.sourceEvent) / 20.0
                const scale = Math.exp(zoom_amount)
                const focus = start_pos
                const distanceScale = d3.zoomIdentity
                    .translate(focus.x, focus.y)
                    .scale(scale)
                    .translate(-focus.x, -focus.y)

                console.log(d3.event.sourceEvent)
                new_xScale = distanceScale.rescaleX(scaleAndAxis.xScale)
                new_yScale = distanceScale.rescaleY(scaleAndAxis.yScale)
            } else {
                new_xScale = d3.event.transform.rescaleX(scaleAndAxis.xScale)
                new_yScale = d3.event.transform.rescaleY(scaleAndAxis.yScale)
            }

            scaleAndAxis.xAxis.call(d3.axisBottom(new_xScale).ticks(box_width / x_axis_label_width))
            scaleAndAxis.yAxis.call(d3.axisLeft(new_yScale))
            scatter
                .selectAll('path')
                .attr(
                    'transform',
                    d => 'translate(' + new_xScale(d.x) + ',' + new_yScale(d.y) + ')'
                )

            if (points.labels === visilbe_points) {
                scatter
                    .selectAll('text')
                    .attr('x', d => new_xScale(d.x) + point_label_padding_x)
                    .attr('y', d => new_yScale(d.y) + point_label_padding_y)
            }
        }

        /**
         * Return the position of this event in local canvas coordinates.
         */
        function get_pos(event) {
            return { x: event.offsetX, y: event.offsetY }
        }

        /**
         * Return the zoom value computed from the initial right-mouse-button event to the current
         * right-mouse event.
         */
        function rmb_zoom_value(event) {
            const end = get_pos(event)
            let delta_x = end.x - start_pos.x
            let delta_y = end.y - start_pos.y
            return delta_x - delta_y
        }

        /**
         * Helper function called when starting to pan/scroll.
         */
        function start_zoom() {
            start_pos = get_pos(d3.event.sourceEvent)
        }

        return { zoomElem, zoom }
    }

    /**
     * Adds brushing functionality to the plot.
     *
     * Brush is a tool which enables user to select points, and zoom into selection via
     * keyboard shortcut or button event.
     */
    addBrushing(box_width, box_height, scatter, scaleAndAxis, selectedZoomBtn, points, zoom) {
        let extent
        let brushClass = 'brush'

        let brush = d3
            .brush()
            .extent([
                [0, 0],
                [box_width, box_height],
            ])
            .on('start ' + brushClass, updateChart)

        // The brush element must be child of zoom element - this is only way we found to have both zoom and brush
        // events working at the same time. See https://stackoverflow.com/a/59757276 .
        let brushElem = zoom.zoomElem.append('g').attr('class', brushClass).call(brush)

        let self = this

        /**
         * Zooms into selected fragment of plot.
         *
         * Based on https://www.d3-graph-gallery.com/graph/interactivity_brush.html
         * Section "Brushing for zooming".
         */
        const zoomIn = () => {
            let xMin = scaleAndAxis.xScale.invert(extent[0][0])
            let xMax = scaleAndAxis.xScale.invert(extent[1][0])
            let yMin = scaleAndAxis.yScale.invert(extent[1][1])
            let yMax = scaleAndAxis.yScale.invert(extent[0][1])

            scaleAndAxis.xScale.domain([xMin, xMax])
            scaleAndAxis.yScale.domain([yMin, yMax])

            self.zoomingHelper(scaleAndAxis, box_width, scatter, points)
        }

        const zoomInKeyEvent = event => {
            if (shortcuts.zoomIn(event)) {
                zoomIn()
                endBrushing()
            }
        }

        /**
         * Updates plot when brushing.
         */
        function updateChart() {
            let selectionEvent = d3.event.selection
            selectedZoomBtn.style.display = 'inline-block'
            selectedZoomBtn.addEventListener('click', zoomIn, true)
            document.addEventListener('keydown', zoomInKeyEvent, true)
            extent = selectionEvent
        }

        /**
         * Removes brush, keyboard event and zoom button when end event is captured.
         */
        const endBrushing = () => {
            brushElem.call(brush.move, null)
            selectedZoomBtn.style.display = 'none'
            selectedZoomBtn.removeEventListener('click', zoomIn, true)
            document.removeEventListener('keydown', zoomInKeyEvent, true)
        }

        let endEvents = ['click', 'auxclick', 'contextmenu', 'scroll']
        endEvents.forEach(e => document.addEventListener(e, endBrushing, false))
    }

    /**
     * Helper function for zooming in after the scale has been updated.
     */
    zoomingHelper(scaleAndAxis, box_width, scatter, points) {
        scaleAndAxis.xAxis
            .transition()
            .duration(animation_duration)
            .call(d3.axisBottom(scaleAndAxis.xScale).ticks(box_width / x_axis_label_width))
        scaleAndAxis.yAxis
            .transition()
            .duration(animation_duration)
            .call(d3.axisLeft(scaleAndAxis.yScale))

        scatter
            .selectAll('path')
            .transition()
            .duration(animation_duration)
            .attr(
                'transform',
                d => 'translate(' + scaleAndAxis.xScale(d.x) + ',' + scaleAndAxis.yScale(d.y) + ')'
            )

        if (points.labels === visilbe_points) {
            scatter
                .selectAll('text')
                .transition()
                .duration(animation_duration)
                .attr('x', d => scaleAndAxis.xScale(d.x) + point_label_padding_x)
                .attr('y', d => scaleAndAxis.yScale(d.y) + point_label_padding_y)
        }
    }

    /**
     * Creates a plot object and populates it with given data.
     */
    createScatter(svg, box_width, box_height, points, dataPoints, scaleAndAxis) {
        let clip = svg
            .append('defs')
            .append('svg:clipPath')
            .attr('id', 'clip')
            .append('svg:rect')
            .attr('width', box_width)
            .attr('height', box_height)
            .attr('x', 0)
            .attr('y', 0)

        let symbol = d3.symbol()

        let scatter = svg.append('g').attr('clip-path', 'url(#clip)')

        let sizeScaleMultiplier = 100

        scatter
            .selectAll('dataPoint')
            .data(dataPoints)
            .enter()
            .append('path')
            .attr(
                'd',
                symbol.type(this.matchShape()).size(d => (d.size || 1.0) * sizeScaleMultiplier)
            )
            .attr(
                'transform',
                d => 'translate(' + scaleAndAxis.xScale(d.x) + ',' + scaleAndAxis.yScale(d.y) + ')'
            )
            .style('fill', d => '#' + (d.color || '000000'))
            .style('opacity', 0.5)

        if (points.labels === visilbe_points) {
            scatter
                .selectAll('dataPoint')
                .data(dataPoints)
                .enter()
                .append('text')
                .text(d => d.label)
                .attr('x', d => scaleAndAxis.xScale(d.x) + point_label_padding_x)
                .attr('y', d => scaleAndAxis.yScale(d.y) + point_label_padding_y)
                .attr('style', label_style)
                .attr('fill', 'black')
        }

        return scatter
    }

    /**
     * Helper function to match d3 shape from string.
     */
    matchShape() {
        return d => {
            if (d.shape === 'cross') {
                return d3.symbolCross
            }
            if (d.shape === 'diamond') {
                return d3.symbolDiamond
            }
            if (d.shape === 'square') {
                return d3.symbolSquare
            }
            if (d.shape === 'star') {
                return d3.symbolStar
            }
            if (d.shape === 'triangle') {
                return d3.symbolTriangle
            }
            return d3.symbolCircle
        }
    }

    /**
     * Creates labels on axes if they're defined.
     */
    createLabels(axis, svg, box_width, margin, box_height) {
        let fontStyle = '10px DejaVuSansMonoBook'
        if (axis.x.label !== undefined) {
            let padding_y = 20
            svg.append('text')
                .attr('text-anchor', 'end')
                .attr('style', label_style)
                .attr('x', margin.left + this.getTextWidth(axis.x.label, fontStyle) / 2)
                .attr('y', box_height + margin.top + padding_y)
                .text(axis.x.label)
        }

        if (axis.y.label !== undefined) {
            let padding_y = 15
            svg.append('text')
                .attr('text-anchor', 'end')
                .attr('style', label_style)
                .attr('transform', 'rotate(-90)')
                .attr('y', -margin.left + padding_y)
                .attr(
                    'x',
                    -margin.top - box_height / 2 + this.getTextWidth(axis.y.label, fontStyle) / 2
                )
                .text(axis.y.label)
        }
    }

    /**
     * Helper function to get text width to make sure that labels on x axis wont overlap,
     * and keeps it readable.
     */
    getTextWidth(text, font) {
        const canvas = document.createElement('canvas')
        const context = canvas.getContext('2d')
        context.font = font
        const metrics = context.measureText('  ' + text)
        return metrics.width
    }

    /**
     * Creates plot's axes.
     */
    createAxes(axis, extremesAndDeltas, box_width, box_height, svg, focus) {
        let { domain_x, domain_y } = this.getDomains(extremesAndDeltas, focus)

        let xScale = d3.scaleLinear()
        if (axis.x.scale !== linear_scale) {
            xScale = d3.scaleLog()
        }

        xScale.domain(domain_x).range([0, box_width])
        let xAxis = svg
            .append('g')
            .attr('transform', 'translate(0,' + box_height + ')')
            .attr('style', label_style)
            .call(d3.axisBottom(xScale).ticks(box_width / x_axis_label_width))

        let yScale = d3.scaleLinear()
        if (axis.y.scale !== linear_scale) {
            yScale = d3.scaleLog()
        }

        yScale.domain(domain_y).range([box_height, 0])
        let yAxis = svg.append('g').attr('style', label_style).call(d3.axisLeft(yScale))
        return { xScale: xScale, yScale: yScale, xAxis: xAxis, yAxis: yAxis }
    }

    /**
     * Helper function calculating domains for x and y axes.
     *
     * Example:
     * Lets say we have a bunch of points. Those points will have some minimum and maximum value,
     * from which we can calculate the span of points on X and Y axis, hence domain, with added
     * padding to make sure points will fit nicely on the chart.
     */
    getDomains(extremesAndDeltas, focus) {
        let domain_x = [
            extremesAndDeltas.xMin - extremesAndDeltas.paddingX,
            extremesAndDeltas.xMax + extremesAndDeltas.paddingX,
        ]
        let domain_y = [
            extremesAndDeltas.yMin - extremesAndDeltas.paddingY,
            extremesAndDeltas.yMax + extremesAndDeltas.paddingY,
        ]

        if (focus !== undefined) {
            if (focus.x !== undefined && focus.y !== undefined && focus.zoom !== undefined) {
                let padding_x = extremesAndDeltas.dx * (1 / (2 * focus.zoom))
                let padding_y = extremesAndDeltas.dy * (1 / (2 * focus.zoom))
                domain_x = [focus.x - padding_x, focus.x + padding_x]
                domain_y = [focus.y - padding_y, focus.y + padding_y]
            }
        }
        return { domain_x, domain_y }
    }

    /**
     * Helper function calculating extreme values and paddings to make sure data will fit nicely.
     *
     * It traverses through data getting minimal and maximal values, and calculates padding based on
     * span calculated from above values, multiplied by 10% so that the plot is a little bit smaller
     * than the container.
     */
    getExtremesAndDeltas(dataPoints) {
        let xMin = dataPoints[0].x
        let xMax = dataPoints[0].x
        let yMin = dataPoints[0].y
        let yMax = dataPoints[0].y

        dataPoints.forEach(d => {
            if (d.x < xMin) {
                xMin = d.x
            }
            if (d.x > xMax) {
                xMax = d.x
            }
            if (d.y < yMin) {
                yMin = d.y
            }
            if (d.y > yMax) {
                yMax = d.y
            }
        })

        let dx = xMax - xMin
        let dy = yMax - yMin

        let padding_x = 0.1 * dx
        let padding_y = 0.1 * dy

        return {
            xMin: xMin,
            xMax: xMax,
            yMin: yMin,
            yMax: yMax,
            paddingX: padding_x,
            paddingY: padding_y,
            dx: dx,
            dy: dy,
        }
    }

    /**
     * Helper function getting margins for plot's box.
     */
    getMargins(axis) {
        if (axis.x.label === undefined && axis.y.label === undefined) {
            return { top: 20, right: 20, bottom: 20, left: 45 }
        } else if (axis.y.label === undefined) {
            return { top: 10, right: 20, bottom: 35, left: 35 }
        } else if (axis.x.label === undefined) {
            return { top: 20, right: 10, bottom: 20, left: 55 }
        }
        return { top: 10, right: 10, bottom: 35, left: 55 }
    }

    /**
     * Creates HTML div element as container for plot.
     */
    createDivElem(width, height) {
        const divElem = document.createElementNS(null, 'div')
        divElem.setAttributeNS(null, 'class', 'vis-scatterplot')
        divElem.setAttributeNS(null, 'viewBox', 0 + ' ' + 0 + ' ' + width + ' ' + height)
        divElem.setAttributeNS(null, 'width', '100%')
        divElem.setAttributeNS(null, 'height', '100%')
        divElem.setAttributeNS(null, 'transform', 'matrix(1 0 0 -1 0 0)')

        const addStyleToElem = (attr, stl) => {
            let style = document.createElement('style')
            style.innerText = attr + '{' + stl + '}'

            divElem.appendChild(style)
        }

        let darkStrokeColor = `rgba(255,255,255,0.7)`
        let buttonLightColor = `#333`
        let darkBtnHoverColor = `rgba(255,255,255,0.5)`
        let darkSelectionFill = `#efefef`

        addStyleToElem('.selection', 'rx: 4px;stroke: transparent;')
        addStyleToElem(
            'button',
            `
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
        `
        )
        addStyleToElem(
            'button:hover',
            `
            background-color: ${buttonLightColor};
            color: ${darkSelectionFill};
        `
        )

        addStyleToElem(
            '.dark-theme button',
            `
            border: 0;
            background-color: ${darkStrokeColor};
        `
        )
        addStyleToElem(
            '.dark-theme button:hover',
            `
            background-color: ${darkBtnHoverColor};
        `
        )
        addStyleToElem('.dark-theme .selection', `fill: ${darkSelectionFill}`)
        addStyleToElem('.dark-theme line', `stroke: ${darkStrokeColor};`)
        addStyleToElem('.dark-theme .domain', `stroke: ${darkStrokeColor};`)
        addStyleToElem('.dark-theme text', `fill: ${darkStrokeColor};`)

        return divElem
    }

    /**
     * Helper function for button creation.
     */
    createBtnHelper() {
        const btn = document.createElement('button')
        btn.setAttribute('width', '80px')
        btn.setAttribute('height', '20px')
        return btn
    }

    /**
     * Creates a button to fit all points on plot.
     */
    createButtonFitAll(scaleAndAxis, scatter, points, extremesAndDeltas, zoom, box_width) {
        const btn = this.createBtnHelper()

        let text = document.createTextNode('Fit all')
        btn.appendChild(text)

        let self = this
        const unzoom = () => {
            zoom.zoomElem.transition().duration(0).call(zoom.zoom.transform, d3.zoomIdentity)

            let domain_x = [
                extremesAndDeltas.xMin - extremesAndDeltas.paddingX,
                extremesAndDeltas.xMax + extremesAndDeltas.paddingX,
            ]
            let domain_y = [
                extremesAndDeltas.yMin - extremesAndDeltas.paddingY,
                extremesAndDeltas.yMax + extremesAndDeltas.paddingY,
            ]

            scaleAndAxis.xScale.domain(domain_x)
            scaleAndAxis.yScale.domain(domain_y)

            self.zoomingHelper(scaleAndAxis, box_width, scatter, points)
        }

        document.addEventListener('keydown', e => {
            if (shortcuts.showAll(e)) {
                unzoom()
            }
        })

        btn.addEventListener('click', unzoom)
        this.dom.appendChild(btn)
    }

    /**
     * Creates a button to zoom into brushed fragment of plot.
     */
    createButtonScaleToPoints() {
        const btn = this.createBtnHelper()
        let text = document.createTextNode('Zoom to selected')
        btn.appendChild(text)
        btn.setAttribute('width', '120px')
        btn.style.display = 'none'
        this.dom.appendChild(btn)
        return btn
    }

    /**
     * Sets size of this DOM object.
     */
    setSize(size) {
        this.dom.setAttributeNS(null, 'width', size[0])
        this.dom.setAttributeNS(null, 'height', size[1])
    }
}

return ScatterPlot
