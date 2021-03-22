/** Histogram Visualization. */
// TODO refactor this to avoid loading on startup. See issue #985 .
loadScript('https://d3js.org/d3.v4.min.js')
loadStyle('https://fontlibrary.org/face/dejavu-sans-mono')

let shortcuts = {
    zoomIn: e => (e.ctrlKey || e.metaKey) && e.key === 'z',
    showAll: e => (e.ctrlKey || e.metaKey) && e.key === 'a',
    debugPreprocessor: e => (e.ctrlKey || e.metaKey) && e.key === 'd',
}

const LABEL_STYLE = 'font-family: DejaVuSansMonoBook; font-size: 10px;'
const MARGIN = 25
const X_AXIS_LABEL_WIDTH = 10
const Y_AXIS_LABEL_WIDTH = 10
const ANIMATION_DURATION = 1000
const LINEAR_SCALE = 'linear'
const LIGHT_PLOT_COLOR = '#00E890'
const DARK_PLOT_COLOR = '#E0A63B'
const DEFAULT_NUMBER_OF_BINS = 10
const BUTTON_HEIGHT = 25

/**
 * A d3.js histogram visualization.
 *
 *
 * Data format (json):
 {
   "axis" : {
      "x" : { "label" : "x-axis label", "scale" : "linear" },
      "y" : { "label" : "y-axis label", "scale" : "logarithmic" },
   },
   "focus" { "x" : 1.7, "y" : 2.1, "zoom" : 3.0 },
   "color" :  "rgb(1.0,0.0,0.0)" },
   "bins" : 10,
   "data" : [
     "values" : [0.1, 0.2, 0.1, 0.15, 0.7],
   ]
 }
 */
class Histogram extends Visualization {
    static inputType = 'Any'
    static label = 'Histogram'

    onDataReceived(data) {
        const { parsedData, isUpdate } = this.parseData(data)
        if (!ok(parsedData)) {
            console.error('Histogram got invalid data: ' + data.toString())
        }
        this.updateState(parsedData, isUpdate)

        if (!this.isInitialised()) {
            this.initCanvas()
            this.initLabels()
            this.initHistogram()
            this.initDebugShortcut()
        }

        this.updateLabels()
        this.updateHistogram()
    }

    parseData(data) {
        let parsedData
        if (typeof data === 'string' || data instanceof String) {
            parsedData = JSON.parse(data)
        } else {
            parsedData = data
        }
        const isUpdate = parsedData.update === 'diff'
        return { parsedData, isUpdate }
    }

    /**
     * Indicates whether this visualisation has been initialised.
     */
    isInitialised() {
        ok(this.svg)
    }

    /**
     * Update the internal data and plot settings with the ones from the new incoming data.
     * If no new settings/data have been provided the old ones will be kept.
     */
    updateState(data, isUpdate) {
        if (isUpdate) {
            this._axisSpec = ok(data.axis) ? data.axis : this._axisSpec
            this._focus = ok(data.focus) ? data.focus : this._focus
            this._dataValues = ok(data.data.values) ? data.data.values : this.data
            this._bins = ok(data.bins) ? data.bins : this._bins
        } else {
            this._axisSpec = data.axis
            this._focus = data.focus
            this._dataValues = this.extractValues(data)
            this._bins = data.bins
        }
    }

    extractValues(rawData) {
        /// Note this is a workaround for #1006, we allow raw arrays and JSON objects to be consumed.
        if (ok(rawData.data)) {
            return rawData.data.values
        } else if (Array.isArray(rawData)) {
            return rawData
        }
        return []
    }

    /**
     * Return the focus area of the histogram. If none is set, returns undefined.
     */
    focus() {
        return this._focus
    }

    /**
     * Return specification for the axis. This includes scales (logarithmic/linear) and labels.
     */
    axisSpec() {
        return (
            this._axisSpec || {
                x: { scale: LINEAR_SCALE },
                y: { scale: LINEAR_SCALE },
            }
        )
    }

    /**
     * Return vales to plot.
     */
    data() {
        return this._dataValues || {}
    }

    /**
     * Return the number of bins to use for the histogram.
     */
    binCount() {
        if (!ok(this._bins)) {
            return DEFAULT_NUMBER_OF_BINS
        } else {
            return Math.max(1, self._bins)
        }
    }

    /**
     * Return the layout measurements for the plot. This includes the outer dimensions of the
     * drawing area as well as the inner dimensions of the plotting area and the margins.
     */
    canvasDimensions() {
        const width = this.dom.getAttributeNS(null, 'width')
        let height = this.dom.getAttributeNS(null, 'height')
        height = height - BUTTON_HEIGHT
        const margin = this.margins()
        return {
            inner: {
                width: width - margin.left - margin.right,
                height: height - margin.top - margin.bottom,
            },
            outer: { width, height },
            margin,
        }
    }

    /**
     * Initialise the drawing svg and related properties, e.g., canvas size and margins.
     */
    initCanvas() {
        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild)
        }

        this.canvas = this.canvasDimensions()
        const container = this.createOuterContainerWithStyle(
            this.canvas.outer.width,
            this.canvas.outer.height
        )
        this.dom.appendChild(container)

        this.svg = d3
            .select(container)
            .append('svg')
            .attr('width', this.canvas.outer.width)
            .attr('height', this.canvas.outer.height)
            .append('g')
            .attr(
                'transform',
                'translate(' + this.canvas.margin.left + ',' + this.canvas.margin.top + ')'
            )

        this.yAxis = this.svg.append('g').attr('style', LABEL_STYLE)
        this.xAxis = this.svg.append('g').attr('style', LABEL_STYLE)

        this.plot = this.svg.append('g').attr('clip-path', 'url(#hist-clip-path)')

        // Create clip path
        const defs = this.svg.append('defs')
        defs.append('clipPath')
            .attr('id', 'hist-clip-path')
            .append('rect')
            .attr('width', this.canvas.inner.width)
            .attr('height', this.canvas.inner.height)
    }

    /**
     * Initialise the histogram with the current data and settings.
     */
    initHistogram() {
        this.updateHistogram()
        const zoom = this.initPanAndZoom()
        // TODO [MM]: In task specification buttons were on top of the visualization, but because
        //            the visualization selector obfuscated them, they're now on the bottom.
        //            This should be fixed in (#898).
        this.createButtonFitAll(zoom)
        const selectedZoomBtn = this.createButtonScaleToPoints()
        this.initBrushing(selectedZoomBtn, zoom)
    }

    initDebugShortcut() {
        document.addEventListener('keydown', e => {
            if (shortcuts.debugPreprocessor(e)) {
                this.setPreprocessor('x -> "[1,2,3,4]"')
                e.preventDefault()
            }
        })
    }

    /**
     * Initialise panning and zooming functionality on the visualization.
     */
    initPanAndZoom() {
        const canvas = this.canvas
        const zoomClass = 'zoom'
        const minScale = 0.5
        const maxScale = 20
        const extent = [minScale, maxScale]
        const zoom = d3
            .zoom()
            .filter(function () {
                let right_button = 2
                let mid_button = 1
                let scroll_wheel = 0
                switch (d3.event.type) {
                    case 'mousedown':
                        return d3.event.button === right_button || d3.event.button === mid_button
                    case 'wheel':
                        return d3.event.button === scroll_wheel
                    default:
                        return false
                }
            })
            .scaleExtent(extent)
            .extent([
                [0, 0],
                [canvas.inner.width, canvas.inner.height],
            ])
            .on(zoomClass, zoomed)

        const zoomElem = this.svg
            .append('g')
            .attr('class', zoomClass)
            .attr('width', canvas.inner.width)
            .attr('height', canvas.inner.height)
            .style('fill', 'none')
            .style('pointer-events', 'all')
            .call(zoom)

        const self = this
        /**
         * Helper function called on pan/scroll.
         */
        function zoomed() {
            self.scale.zoom = d3.event.transform.k
            let tmpScale = Object.assign({}, self.scale)
            tmpScale.x = d3.event.transform.rescaleX(self.scale.x)
            self.rescale(tmpScale, false)
        }

        return { zoomElem, zoom }
    }

    /**
     * Initialise brushing functionality on the visualization.
     *
     * Brush is a tool which enables user to select points, and zoom into selection via
     * keyboard shortcut or button event.
     */
    initBrushing(selectedZoomBtn, zoom) {
        let extent
        const brushClass = 'brush'

        const brush = d3
            .brushX()
            .extent([
                [0, 0],
                [this.canvas.inner.width, this.canvas.inner.height],
            ])
            .on('start ' + brushClass, updateChartOnBrush)

        // The brush element must be child of zoom element - this is only way we found to have both
        // zoom and brush events working at the same time. See https://stackoverflow.com/a/59757276 .
        const brushElem = zoom.zoomElem.append('g').attr('class', brushClass).call(brush)

        const self = this

        /**
         * Zooms into selected fragment of plot.
         *
         * Based on https://www.d3-graph-gallery.com/graph/interactivity_brush.html
         * Section "Brushing for zooming".
         */
        const zoomIn = () => {
            const xMin = self.scale.x.invert(extent[0])
            const xMax = self.scale.x.invert(extent[1])

            self.scale.x.domain([xMin, xMax])
            const dx = extent[1] - extent[0]
            self.scale.zoom = self.scale.zoom * (self.canvas.inner.width / dx)

            self.rescale(self.scale, true)
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
        function updateChartOnBrush() {
            const selectionEvent = d3.event.selection
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
     * Helper function for rescaling the data points with a new scale.
     */
    rescale(scale, with_animation) {
        const animation_duration = with_animation ? ANIMATION_DURATION : 0.0
        this.xAxis
            .transition()
            .duration(animation_duration)
            .call(d3.axisBottom(scale.x).ticks(this.binCount()))
        this.plot
            .selectAll('rect')
            .transition()
            .duration(animation_duration)
            .attr(
                'transform',
                d =>
                    'translate(' +
                    scale.x(d.x0) +
                    ',' +
                    scale.y(d.length) +
                    ')scale(' +
                    scale.zoom +
                    ',1)'
            )
    }

    /**
     * Update the d3 histogram with the current data.
     *
     * Binds the new data to the plot, creating new bars, removing old ones and
     * updates the axes accordingly.
     */
    updateHistogram() {
        const extremesAndDeltas = this.extremesAndDeltas()
        const dataPoints = this.data()
        const focus = this.focus()

        let domain_x = [
            extremesAndDeltas.xMin - extremesAndDeltas.paddingX,
            extremesAndDeltas.xMax + extremesAndDeltas.paddingX,
        ]

        if (focus !== undefined) {
            if (focus.x !== undefined && focus.zoom !== undefined) {
                let padding_x = extremesAndDeltas.dx * (1 / (2 * focus.zoom))
                domain_x = [focus.x - padding_x, focus.x + padding_x]
            }
        }

        const x = d3.scaleLinear().domain(domain_x).range([0, this.canvas.inner.width])

        this.xAxis
            .attr('transform', 'translate(0,' + this.canvas.inner.height + ')')
            .call(d3.axisBottom(x))

        const histogram = d3
            .histogram()
            .value(d => d)
            .domain(x.domain())
            .thresholds(x.ticks(this.binCount()))

        const bins = histogram(dataPoints)

        const y = d3.scaleLinear().range([this.canvas.inner.height, 0])
        y.domain([0, d3.max(bins, d => d.length)])

        const yAxisTicks = y.ticks().filter(tick => Number.isInteger(tick))

        const yAxis = d3.axisLeft(y).tickValues(yAxisTicks).tickFormat(d3.format('d'))

        this.yAxis.call(yAxis)

        let accentColor = LIGHT_PLOT_COLOR

        if (document.getElementById('root').classList.contains('dark')) {
            accentColor = DARK_PLOT_COLOR
        }

        const items = this.plot.selectAll('rect').data(bins)

        this.bars = items
            .enter()
            .append('rect')
            .attr('x', 1)
            .attr('transform', d => 'translate(' + x(d.x0) + ',' + y(d.length) + ')')
            .attr('width', d => x(d.x1) - x(d.x0))
            .attr('height', d => this.canvas.inner.height - y(d.length))
            .style('fill', accentColor)

        items.exit().remove()

        this.scale = { x, y, zoom: 1.0 }
    }

    /**
     * Creates labels on axes if they are defined.
     */
    initLabels() {
        this.yAxisLabel = this.svg
            .append('text')
            .attr('text-anchor', 'end')
            .attr('style', LABEL_STYLE)
            .attr('transform', 'rotate(-90)')
        this.xAxisLabel = this.svg
            .append('text')
            .attr('text-anchor', 'end')
            .attr('style', LABEL_STYLE)
    }

    /**
     * Update labels with current data.
     */
    updateLabels() {
        const axis = this.axisSpec()
        const canvas = this.canvas

        const fontStyle = '10px DejaVuSansMonoBook'
        if (axis.x.label !== undefined) {
            this.xAxisLabel
                .attr('y', canvas.inner.height + canvas.margin.bottom - X_AXIS_LABEL_WIDTH / 2.0)
                .attr('x', canvas.inner.width / 2.0 + this.textWidth(axis.x.label, fontStyle) / 2)
                .text(axis.x.label)
        }
        // Note: y axis is rotated by 90 degrees, so x/y is switched.
        if (axis.y.label !== undefined) {
            this.yAxisLabel
                .attr('y', -canvas.margin.left + Y_AXIS_LABEL_WIDTH)
                .attr('x', -canvas.inner.height / 2 + this.textWidth(axis.y.label, fontStyle) / 2)
                .text(axis.y.label)
        }
    }

    /**
     * Return the text width. Ensures that labels on x axis wont overlap to keeps them readable.
     */
    textWidth(text, font) {
        const canvas = document.createElement('canvas')
        const context = canvas.getContext('2d')
        context.font = font
        const metrics = context.measureText('  ' + text)
        return metrics.width
    }

    /**
     * Return the extrema of the data and and paddings that ensure data will fit into the
     * drawing area.
     *
     * It traverses through data getting minimal and maximal values, and calculates padding based on
     * span calculated from above values, multiplied by 10% so that the plot is a little bit smaller
     * than the container.
     */
    extremesAndDeltas() {
        const dataPoints = this.data()
        let xMin = dataPoints[0]
        let xMax = dataPoints[0]

        dataPoints.forEach(value => {
            if (value < xMin) {
                xMin = value
            }
            if (value > xMax) {
                xMax = value
            }
        })

        const dx = xMax - xMin
        const paddingX = 0.1 * dx

        return { xMin, xMax, paddingX, dx }
    }

    /**
     * Return margins for plots drawing area.
     */
    margins() {
        const axis = this.axisSpec()
        const noXAxis = axis.x.label === undefined
        const noYAxis = axis.y.label === undefined

        const top = MARGIN / 2.0
        const right = MARGIN / 2.0
        if (noXAxis && noYAxis) {
            return { top, right, bottom: MARGIN, left: MARGIN }
        } else if (noYAxis) {
            return {
                top,
                right,
                bottom: MARGIN + X_AXIS_LABEL_WIDTH,
                left: MARGIN,
            }
        } else if (noXAxis) {
            return {
                top,
                right,
                bottom: MARGIN,
                left: MARGIN + Y_AXIS_LABEL_WIDTH,
            }
        }
        return {
            top,
            right,
            bottom: MARGIN + X_AXIS_LABEL_WIDTH,
            left: MARGIN + Y_AXIS_LABEL_WIDTH,
        }
    }

    /**
     * Creates HTML div element as container for plot.
     */
    createOuterContainerWithStyle(width, height) {
        const divElem = document.createElementNS(null, 'div')
        divElem.setAttributeNS(null, 'class', 'vis-histogram')
        divElem.setAttributeNS(null, 'viewBox', 0 + ' ' + 0 + ' ' + width + ' ' + height)
        divElem.setAttributeNS(null, 'width', '100%')
        divElem.setAttributeNS(null, 'height', '100%')

        const addStyleToElem = (attr, stl) => {
            const style = document.createElement('style')
            style.innerText = attr + '{' + stl + '}'

            divElem.appendChild(style)
        }

        const darkStrokeColor = `rgba(255,255,255,0.7)`
        const buttonLightColor = `#333`
        const darkBtnHoverColor = `rgba(255,255,255,0.5)`
        const darkSelectionFill = `#efefef`

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
            '.dark button',
            `
            border: 0;
            background-color: ${darkStrokeColor};
        `
        )
        addStyleToElem(
            '.dark button:hover',
            `
            background-color: ${darkBtnHoverColor};
        `
        )
        addStyleToElem('.dark .selection', `fill: ${darkSelectionFill}`)
        addStyleToElem('.dark line', `stroke: ${darkStrokeColor};`)
        addStyleToElem('.dark .domain', `stroke: ${darkStrokeColor};`)
        addStyleToElem('.dark text', `fill: ${darkStrokeColor};`)

        return divElem
    }

    /**
     * Create a button HTML element with default size.
     */
    createButtonElement() {
        const btn = document.createElement('button')
        btn.setAttribute('width', '80px')
        btn.setAttribute('height', '20px')
        return btn
    }

    /**
     * Creates a button that when clicked pans and zooms the plot to fit all data points.
     */
    createButtonFitAll(zoom) {
        const extremesAndDeltas = this.extremesAndDeltas()
        const btn = this.createButtonElement()

        let text = document.createTextNode('Fit all')
        btn.appendChild(text)

        const self = this
        const reset_zoom_and_pan = () => {
            zoom.zoomElem.transition().duration(0).call(zoom.zoom.transform, d3.zoomIdentity)

            let domain_x = [
                extremesAndDeltas.xMin - extremesAndDeltas.paddingX,
                extremesAndDeltas.xMax + extremesAndDeltas.paddingX,
            ]

            self.scale.x.domain(domain_x)
            self.scale.zoom = 1.0
            self.rescale(self.scale, true)
        }

        document.addEventListener('keydown', e => {
            if (shortcuts.showAll(e)) {
                reset_zoom_and_pan()
            }
        })

        btn.addEventListener('click', reset_zoom_and_pan)
        this.dom.appendChild(btn)
    }

    /**
     * Creates a button to zoom into brushed fragment of plot.
     */
    createButtonScaleToPoints() {
        const btn = this.createButtonElement()
        const text = document.createTextNode('Zoom to selected')
        btn.appendChild(text)
        btn.setAttribute('width', '120px')
        btn.style.display = 'none'
        this.dom.appendChild(btn)
        return btn
    }

    /**
     * Sets size of the main parent DOM object.
     */
    setSize(size) {
        console.warn("setting size", size)
        this.dom.setAttributeNS(null, 'width', size[0])
        this.dom.setAttributeNS(null, 'height', size[1])
    }
}

/**
 * Checks if `t` has defined type and is not null.
 */
function ok(t) {
    return t !== undefined && t !== null
}

return Histogram
