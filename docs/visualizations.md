# Visualization workflow

## Purpose of visualizations
Visualizations have two main purposes:

- Display results of computations. Each node can be assigned with a
  visualization (displayed next to the node). After a node computes its new
  value, the visualization shows it in an understandable way to the user.

- Control the computations. Think about visualizations as a separate component,
  just as nodes, with their own input and output ports. Whenever new data
  appears on the input port, the visualization displays it. However, when the
  input port is not connected, the visualization behaves like a data-producing
  widget. For example, a map visualization can display locations encoded in its
  input data. It can also allow us to interactively pick new locations with the
  mouse when no input is connected and sends the manually selected locations to
  the output port. Similarly, the histogram can be used to manually draw a
  histogram with the mouse to set data on the next nodes. A number visualization
  is, at the same time, a slider. Image visualizations can behave like an image
  editor, etc.


## Visualization Display Forms
Visualizations can be displayed in the following ways:

- **Attached to nodes**  
  Their input and output ports are not visible in this form. They are used only
  to display the result of the attached node. Whenever you move the node, the
  visualization moves as well. The visualization can be shown this way by
  selecting nodes by clicking the "space" button.

- **Fullscreen**  
  Similar to the previous mode, but the visualization occupies the full visual
  space. This form can be triggered on a recently selected node (even if many
  nodes are selected, we remember the last selected one) by either pressing
  space and keeping it pressed for longer than approx 0.5 s, or by clicking
  space twice. In the former case, the visualization stops being fullscreen
  whenever we release space, in the later, whenever we click space again.

- **Detached**  
   This form is used to build dashboards and reports. You can detach a
   visualization and place it anywhere you want (we might introduce a special
   place where you can put such visualizations). Think about building a PDF-like
   report for your supervisor - you want to describe your research there and
   place some visualizations and widgets (like sliders, which are a
   visualization as well) inside the text for an interactive experience. Please
   note that a single node can be assigned with multiple visualizations at the
   same time. For example, a node might want to display a map of locations and
   their list at the same time next to each other.

- **Widgets**  
  This form behaves just like a node on the stage, but it does not have an
  expression - it is only the visualization. It has input and output ports.
  Whenever the input port is connected, it behaves like in the attached mode -
  it displays the results it gets on the input and passes them to the output
  port. When the input port is not connected, the visualization behaves like a
  widget, allowing you to produce data to the nodes connected to its output
  port.

### Choosing a Visualization Type.
When a new data is provided to a visualization, the visualization registry
searches for all visualizations that match it (see visualization registry to
learn more). For example, when a data of type `[Int]` (list of ints) is
produced, all visualizations which map `[Int]`, `[a]`, or a will be found. Each
type can be associated with a default visualization. For example, `[Int]` might
define that its default visualization is a plot. If no default visualization is
defined, a JSON visualization is used by default. Next to the visualization
view, there is a menu to change the visualization type - it lists all
visualization types that match the current data type.


### Active Visualizations
When visualizations are displayed on the stage, they are not active by default,
which means, they do not capture keyboard shortcuts. Visualization can be made
active after a user clicks on it. Visualizations are deactivated by clicking in
the background of the node editor. When a visualization is active, all other
elements should be slightly dimmed, or the visualization should get a selection
border (to be decided). Active visualizations capture all keyboard shortcuts,
but the "space" button presses. Fullscreen visualizations are considered active
by default.

## HTML and Native Visualizations
There are two main types of visualizations - Html and Native. The later uses the
BaseGL shape API to draw on the screen. We prefer the later as it integrates
tightly with our framework and allows for much better performance. However,
there already exist many visualizations in HTML/JS that we need to provide
support for them as well. HTML visualizations are required to be displayed in
dedicated div elements. This has several consequences. Firstly, the browser
needs to layout them, taking into account the current camera view, etc. It is
costly. Refreshing CSS3D styles of 100 visualizations can absolutely kill
interactive performance. On the other hand, refreshing the position of 10k
Native visualizations is almost free. Secondly, they need to be handled by our
engine in such a way that we can interact with them. For that purpose, the
current Scene implementation defines three layers - top HTML layer, middle WebGL
layer, and bottom HTML layer. The HTML visualizations are created and displayed
on the bottom layer by default. Whenever an HTML visualization gets active, it
should be moved to the top layer.


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

### Defining a Visualization
This needs to be described in detail. For now, we can just assume that the user
is allowed to create visualization and register it in Enso.

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
