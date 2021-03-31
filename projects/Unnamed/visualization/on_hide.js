
// noinspection JSAnnotator
return class BubbleVisualization extends Visualization {
    static inputType = "Any";

    onHide() {
        console.log("hiding")
        console.log(this)
    }
};