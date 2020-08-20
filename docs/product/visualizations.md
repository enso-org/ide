---
layout: developer-doc title: Visualization Workflow category: product tags:
[product]
---



# Visualizations

A visualization is a graphical element which displays data in user-friendly way.
Example visualizations are a geo map visualization, a scatter plot, a sound wave
plot, or a 3D model viewer. Visualizations can be defined in every Enso library,
including user projects.



# Visualization Rendering Technologies

Visualizations can utilize any web technologies to display the data. The most
popular approaches sorted by the performance (fast to slow) are listed below:

- **EnsoGL**  
  EnsoGL is a blazing fast vector GPU renderer utilizing WebGL 2.0 (soon also
  WebGPU) under the hood. It allows displaying hundreads of thousands of shapes
  60 frames per second in a web browser. It was designed from scratch with
  graphical user interfaces and visualizations in mind. For example, the whole
  Enso graphical interface is drawn by EnsoGL.

- **WebGL libraries**  
  Babylon.js, Three.js, and other WebGL libraries deliver a generic API for
  working with GPU-accelerated scenes. As their focus is much more generic than
  EnsoGL, it is more complex to draw shapes using their APIs and the resulting
  performance is often also lower than when using EnsoGL.

- **SVG**  
  SVG is one of the most popular choices because of its maturity, easy to use
  API and a lot of helper libraries, like D3.js. However, the possible
  performance of SVG visualizations can be even several orders of magnitude
  worse than in case of the alternatives listed above. If you do not plan to
  display a lot of data in your visualization, it is perfectly fine to use this
  solution.

- **DOM** Just like SVG, this is a perfect solution to display some specific
  type of data, like a page of text, or a simple table of numbers or strings.
  The performance of DOM-based visualizations is similar to SVG-based ones.



# Visualization Definition

Visualization definition have to always be placed inside of the `$LIB/src/Viz`
folder, where `$LIB` is the root of an Enso library / project. An example file
structure is presented below:

```
── Library_Name
   ╰─ src
      ├─ Main.enso
      ╰─ Viz
         ├─ Scatter_Plot.enso
         ╰─ Scatter_Plot.js
```

The visualization `Scatter_Plot` definition consists of two main components:

- [Required] The display form definition in JavaScript, Rust (comming soon), or
    Enso (comming soon) in `$LIB/src/Viz/Scatter_Plot.{js,rs,enso}`.
- [Optional] The associated visualization pre-processing Enso function `my_viz`
  defined in the `$LIB/src/Viz/Scatter_Plot.enso` file. If defined, the function
  will be automatically evaluated server-side before sending the data to the
  visualization display form.



# Visualization Types (Monitors and Widgets)

Visualizations can either only display the data, or also allow to modify it.
Although there are no big differences from a technical point of view, we label
them **monitors** and **widgets** respectively.


## Monitors
Monitors are capable of displaying data, but cannot modify it.

### Code Representation
Monitors do not influence the data flow logic, and thus they are not represented
in the source code, however, their settings are stored as metadata for a
particular node. The metadata contains user-set visual preferences and do not
affect the data processing workflow, in particular:
- qualified name of the visualization (e.g. Std.Scatter_Plot),
- its parameters (e.g. the zoom level of a geo map, see "Visualization
  Parameters" to learn more),
- its layout mode (e.g. attached to the right edge of the screen).


### Server Pre-Processing
If the associated visualization pre-processing Enso function was defined, it will
be evaluated server side before the display form receives the data. Logically, this
is equivalent to inserting a line of code directly after the variable we are trying
to visualize. For example, when visualizing the variable `location` by using the
`Std.Viz.Map` visualization, the second line represents the logical insertion:
```python
location = Std.Geo.Location 37.7 122.4
pre_processed_data = (location.to Std.Viz.Map).data
```
Please note, that `location.to Std.Viz.Map` uses the Enso conversions syntax to
convert the `location` data to the `Std.Viz.Map` type. This way, the same
visualization definition can be applied to a wide range of types. The only
requirement is that a conversion from that type to the visualization type needs
to be defined.


### Display Modes
Each monitor can be displayed in the following ways:

- **Attached to Nodes**  
  The most common display mode in which the visualization displays the most
  recent result of the node. Each node can be assigned with one or more
  visualizations displayed side by side at the same time. This mode can be
  previewed by holding spacebar or toggled by tapping the spacebar.

- **Detached from Nodes**  
  Monitors can be detached from nodes, moved, scaled, and placed freely across
  the visual canvas to design a dashboard or report. We also plan to provide a
  notebook-like experience where you can write text mixed with visualizations,
  side by side of the node editor. Visualizations embedded in such docs are
  using exactly this display mode. 

- **Fullscreen**  
  Visualization attached to node can grow (animate) to ocupy full IDE visual
  space. This mode can be triggered on the recently selected node (in case many
  nodes are selected, the last selected node will be used) by either pressing
  keeping the spacebar pressed for longer than approx 0.5s, or by tapping it
  twice. In the former case, the visualization shrinks to each original form
  whenever we release space, in the later, whenever we press space again.



## Widgets
Widgets allow both the visualization as well as the modification of data.
Widgets are subset of monitors – each widget can be used as a monitor to only
display the incomming data. In such a case, they are handled just as monitors
and the previous section applies.

### Code Representation
Widgets are represented in the source code in the same way as nodes - each
widget is associated with one statement (expression that starts on the beginning
of a line and can span across multiple lines). The only difference between
normal expression and a widget is that a widget is associated with a metadata
containing the name of the visualization used to display this widget. Please
note that the metadata do not contain any more information, as all data is
encoded in the expression.

which it as "widget". For example, a widget allowing picking a place on a map
can be representaed in the source code as:
```python
    location = Std.Geo.Location 37.7 122.4
    viz_type = location.to Std.Geo.Map
    viz_data = viz_type . zoom 5 . align_to_cities
    location = viz_data.to Std.Geo.Location
# |    label    |    name    |             parameters              |
```

Widgets are represented in the source code almost in the same way as nodes -
each widget is associated with one statement (expression that starts on the
beginning of a line and can span across multiple lines). The only difference
between normal expression and a widget is that widget is associated with a
metadata which it as "widget". For example, a widget allowing picking a place on
a map can be representaed in the source code as:
```python
    viz1 = Std.Viz.map (zoom = 5) (center = Point 37.7 122.4)
# |  label   |    name    |             parameters              |
```
The __label__ is optional, the __name__ uniquely identifies the visualization
implementation, while the __parameters__ are visualization parameters that can
be overriden by the widget on user interaction (for example, after zooming the
map).

The only exception from the above presented form are literals, which value is
used instead of both __name__ and __parameters__ part. They are also assigned
with additional metadata which stores the type of visualization used. For
example, a logarithmic slider widget can be represented as:
```python
radius = 14.7
```
... with attached metadata describing the visualization type to be
`Std.Viz.SliderLog`.


## Display Mode

Widgets can be displayed in the following ways:

- **Nodes**  
  In this display mode, widgets behave just like regular nodes. They have input
  and output ports, can be placed on the graph editor and connected with other
  nodes. There are few important differences:
  - The expressions are hidden permanently, while the input and output ports are
    drawn on top and on the bottom of the widget respectively.
  - The visualization occupies whole node content. For example, the map widget
    looks just like a rectangular map component with additional input and output
    ports.


  They have one input and one output port. If the input port is connected, the
  visualization displays its value and passes its to the output port. In case it
  is not connected, the visualization becomes an interactive widget allowing the
  user to specify data. For example, a map visualization will allow the user to
  manually pick locations. After each change, the new locations will be sent to
  the output port. Under the hood, widgets are represented as nodes and their
  code lines are assigned with a dedicated "visualization" metadata.
  Visualizations generate expressions always in the form of `name = data`, where
  data is a hardcoded data produced from the visualization. For example, when
  user clicks the map to define locations, the data could be a string literal
  containing locations encoded in JSON.







### Monitors
Monitors are n

 For example, a map widget can both display locations, as well as allow users to
pick one by clicking the desired place. 








## Purpose of visualizations
Visualizations have two main purposes:

- **Display results of nodes**  
  Each node can be assigned with one or more visualization. After a node
  computes its new value, the visualization shows it in an understandable way to
  the user. Please note that a single node can be assigned with multiple
  visualizations at the same time. For example, a node might want to display a
  map of locations, and their list at the same time next to each other.

- **Provide interactive way to generate new data**  
  In a widget mode (described in detail later), visualizations provide users
  with an interactive GUI to define data. For example, a map visualization can
  both display locations, as well as allowing the user to pick locations by
  clicking with a mouse. Similarly, the histogram can both display a list of
  numbers, and can be manually draw with the mouse producing such a list.
  Several numbers can be visualized as a table of sliders, which can also be
  used to interactively generate a table of numbers. Image visualizations can
  behave like an image editor, etc.


## Visualization Display Forms
Visualizations can be displayed in the following ways:

- **Attached to nodes** In this mode, visualizations display the most recent
  result of the node. They behave like an integrated part of the node. Whenever
  you move the node, the visualization moves as well. This mode can be toggled
  by tapping the spacebar.

- **Fullscreen**  
  Visualization attached to node can grow (animate) to ocupy full IDE visual
  space. This mode can be triggered on the recently selected node (in case many
  nodes are selected, the last selected node will be used) by either pressing
  keeping the spacebar pressed for longer than approx 0.5s, or by tapping it
  twice. In the former case, the visualization shrinks to each original form
  whenever we release space, in the later, whenever we press space again.

- **Detached**  
  Visualizations attached to nodes can be detached, scaled, and placed freely
  across the visual canvas (we might introduce a special place where you can put
  such visualizations). This is useful when defining dashboards or reports. We
  also plan to provide a notebook-like experience where you can write text mixed
  with visualizations (including widgets for an interactive experience). 

- **Widgets**  
  In this mode visualizations behave like nodes but do not display expressions.
  They have one input and one output port. If the input port is connected, the
  visualization displays its value and passes its to the output port. In case it
  is not connected, the visualization becomes an interactive widget allowing the
  user to specify data. For example, a map visualization will allow the user to
  manually pick locations. After each change, the new locations will be sent to
  the output port. Under the hood, widgets are represented as nodes and their
  code lines are assigned with a dedicated "visualization" metadata.
  Visualizations generate expressions always in the form of `name = data`, where
  data is a hardcoded data produced from the visualization. For example, when
  user clicks the map to define locations, the data could be a string literal
  containing locations encoded in JSON.


### Choosing a Visualization Type.
When a new data is provided to a visualization, the visualization registry
searches for all visualizations that match it (see visualization registry to
learn more). For example, when a data of type `[Int]` (list of ints) is
produced, all visualizations which matches `[Int]`, like `[Int]`, `[a]`, or `a`
will be found. Each type can be associated with a default visualization. For
example, `[Int]` might define that its default visualization is a plot. If no
default visualization is defined, a JSON visualization is used. Each
visualization has a drop-down menu allowinh the user switching to another
visualization type.

### Active Visualizations
When visualizations are displayed on the stage, they are not active by default,
which means, they do not capture keyboard shortcuts. Visualization becomes
active when user clicks it. Visualizations are deactivated by clicking in the
background of the node editor. When a visualization is active, all other
elements should be slightly dimmed, or the visualization should get a selection
border (to be defined). Active visualizations capture all keyboard shortcuts,
but the space bar presses. Fullscreen visualizations are considered active by
default.


## HTML and Native Visualizations
There are two main types of visualizations - Html and Native. The later uses the
BaseGL shape API to draw on the screen. We prefer the later as it integrates
tightly with our framework and allows for much better performance. However,
there is already many visualizations in HTML/JS and we need to provide support
for them as well. HTML visualizations are required to be displayed in dedicated
div elements. This has several consequences. Firstly, the browser needs to
layout them, taking into account the current camera view, etc. It is costly.
Refreshing CSS3D styles of 100 visualizations can absolutely kill the
interactive performance. On the other hand, refreshing the position of 10k
Native visualizations is almost free. Secondly, they need to be handled by our
engine in such way that we can interact with them. For that purpose, the current
Scene implementation defines three layers - top HTML layer, middle WebGL layer,
and bottom HTML layer. The HTML visualizations are created and displayed on the
bottom layer by default. Whenever an HTML visualization gets active, it should
be moved to the top layer.


## Visualization Registry
Visualizations are user-defined. Enso ships with a set of predefined
visualizations, but they are in no way different than user-defined, they are
just defined for you. Visualizations can be defined either as HTML or native
visualization and can be defined in JS or WASM (or any language that compiles to
one of these). Visualizations are stored on disk on the server-side and are
provided to the GUI by the server. Users can upload their custom visualizations
as well. Each visualization is registered in the visualization map. The map maps
an Enso type to a set of visualizations defined for that type. The type might be
very generic, like `[a]` (which in Enso terms means list of any elements).








#### Custom Visualization Example

Every visualization must reside in the `visualization` folder of the user's
project. For instance:

```
└─ ProjectName
   ├─ src
   │  └─ Main.enso
   └─ visualization
      └─ bubble.js
```

Visualizations can be defined as a JavaScript function which returns a class of
a shape specified below. Consider the following definition:

```javascript
console.log("Hi, this definition is being registered now!")

return class BubbleVisualization extends Visualization {
    static inputType = "Any"

    onDataReceived(data) {
        const xmlns = "http://www.w3.org/2000/svg";
        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild);
        }
        const width   = this.dom.getAttributeNS(null, "width");
        const height  = this.dom.getAttributeNS(null, "height");
        const svgElem = document.createElementNS(xmlns, "svg");
        svgElem.setAttributeNS(null, "id"     , "vis-svg");
        svgElem.setAttributeNS(null, "viewBox", "0 0 " + width + " " + height);
        svgElem.setAttributeNS(null, "width"  , "100%");
        svgElem.setAttributeNS(null, "height" , "100%");
        this.dom.appendChild(svgElem);
        data.forEach(data => {
            const bubble = document.createElementNS(xmlns,"circle");
            bubble.setAttributeNS(null,"stroke", "black");
            bubble.setAttributeNS(null,"fill"  , "red");
            bubble.setAttributeNS(null,"r"     , data[2]);
            bubble.setAttributeNS(null,"cx"    , data[0]);
            bubble.setAttributeNS(null,"cy"    , data[1]);
            svgElem.appendChild(bubble);
        });
    }

    setSize(size) {
        this.dom.setAttributeNS(null, "width", size[0]);
        this.dom.setAttributeNS(null, "height", size[1]);
    }
}
```

In particular:

- [Required] **Source code**

  Visualization definition has to be a valid body of JavaScript function which
  returns a class definition. Instances of that class will be considered
  separate visualizations. You are allowed to use global variables / global
  state across visualizations of the same type, but you are highly advised not
  to do so.

- [Required] **`Visualization` superclass**

  The class returned by the definition function should extend the predefined
  `Visualization` class. Classes which do not extend it, will not be registered
  as visualizations. The superclass defines a default constructor and a set of
  utils:
  - The `setPreprocessor(code)` method allowing setting an Enso code which will
    be evaluated on server-side before sending data to visualization.
  - The `dom` field, which will be initialized in the constructor to the DOM
    symbol used to host the visualization content. You are free to modify the
    DOM element, including adding other elements as its children.

- [Optional] **Field `label`**

  The static field `label` is an user-facing name used to identify the
  visualization. You are not allowed to define several visualizations of the
  same name in the same Enso library. In case the field is missing, the name
  will be inferred from the class name by splitting the camel-case name into
  chunks and converting them to lowercase string.

- [Optional] **Field `inputType`**

  The static field `inputType` is used to determine which Enso data types this
  visualization can be used for. Its value should be a valid Enso type, like
  "String | Int". In case the field is an empty string or it is missing, it will
  default to "Any", which is a type containing all other types. It is a rare
  case when you want to define a visualization which is able to work with just
  any data type, so you are highly advised to provide the type definition.

- [Optional] **Field `inputFormat`**

  The static field `inputFormat` is used to determine what format the data
  should be provided to the `onDataReceived` function. Currently, the only valid
  option is "json", but it will be possible to set it to "binary" in the future.
  In the later case, it is up to the visualization author to manage the binary
  stream received from the server.

- [Optional] **Constructor**

  The visualization will be instantiated by providing the constructor with a
  configuration object. The shape of the configuration object is not part of the
  public API and can change between releases of this library. You have to pass
  it unchanged to the superclass constructor.

- [Optional] **Function `onDataReceived`**

  The `onDataReceived(data)` method is called on every new data chunk received
  from the server. Note that the visualization will receive the "full data" if
  you are not using the `setPreprocessor` method.

- [Optional] **Function `setSize`**

  The `setSize(size)` method is called on every size change of the
  visualization. You should not draw outside of the provided area, however, if
  you do so, it will be clipped to the provided area automatically. The `size`
  parameter contains two fields `width` and `height` expressed in pixels.

### Sending Data to Visualizations

#### Lazy Visualizations
Very important information is how visualization architecture works to make them
interactive and fast. Whenever new data is computed by the compiler and
visualization is attached to it, it is sent to GUI to be displayed. However,
sending really big chunks of data will kill the performance. When defining a
visualization user is capable of defining a chunk of Luna code (as a string).
This code is part of the visualization definition and is stored server-side.
Visualizations are allowed to change the code at runtime. This code defines an
Enso function, which will be run by the compiler on data the visualization is
attached to. Only the results of this code will be sent to the GUI. For example,
imagine you want to display a heatmap of 10 million points on a map. And these
points change rapidly. Sending such amount of information via WebSocket could be
too much, and you (as the visualization author) might decide that the
visualization image should be generated on the server, and your visualization is
meant only to display the resulting image. In such a scenario, you can define in
your visualization an Enso function which will compute the image on the server!

#### Binary and Text (JSON) Formats
Each visualization can choose whether it supports either binary or JSON input.
The input format defaults to JSON. The data from the server is always sent to
GUI in a binary channel, however, when JSON format is selected, it is first
converted to JSON representation on the server side. We can assume that all Enso
data types have defined conversion to JSON by default. If the visualization
input is defined as JSON input, the binary stream will be converted to JSON by
the GUI engine before passing to visualization. It is up to the visualization
author to handle the textual or binary form. 
