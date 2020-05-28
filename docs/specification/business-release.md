# Business Release Features Specification

* [File Browser](#file-browser)
* [Multiple Definitions](#multiple-definitions)
* [Modules Management](#modules-management)
* [Labeled Empty Ports](#labeled-empty-ports)
* [Caching Searcher Suggestions](#caching-searcher-suggestions)
* [Text Editor for Definition](#text-editor-for-definition)
* [Searching Documentation](#searching-documentation)
* [Learning Guide](#learning-guide)
* [Expanded Nodes and Panels](#expanded-nodes-and-panels)
* [Explore Community Scenes](#explore-community-scenes)



### File Browser

[Estimate](./README.md#note-about-estimates): 15 days

#### Story

FileBrowser is a panel containing the file listing of the root Project directory. The root
Project directory is a first contentId returned by project/open method of Language Server. The
listed files have icons. When user clicks in a directory, the analogous panel shows on the right
side, with listing of chosen directory, then the user can click a directory in the new panel,
and the next panel arrives, etc. When choosing another directory in some panel, all panels to the
right are hidden and one is shown, with the newly selected directory content.

The FileBrowser should appear in Searcher above the suggestion list and below the Searcher input
in the following situation:
* The suggestion provider for nodes received information from Engine, that the expected type is
  `FilePath`, and
* the _pattern_ part of the Searcher input is empty. (See Node Suggestions section).

If user choose file in this FileBrowser, the file path should be appended to the expression in
the Searcher input.



### Multiple Definitions

[Estimate](./README.md#note-about-estimates): 40 days
* Create Definitions: 5
* Entering Graph: 8
* Input nodes: 11
* Output nodes: 8
* Ordering of input and outputs (UNSPECIFIED): 8

#### Story

##### Create Definitions

The suggestion list in Searcher has a new element in `Actions` category, labeled _Create
function &lt;searcher input&gt;_. The action is available when:
* One of this condition is met:
  * the searcher mode is `General`;
  * the searcher mode is `AddNodeToSelection` and the type of selected node output is known;
* the searcher context is `None`,
* and the current Searcher input contains a valid function name.

When this suggestion is chosen (by mouse click, or pressing Enter having it selected):
* The Searcher panel disappears
* A new definition in the module of the currently displayed graph is created, whose name is taken
  from searcher input.  If searcher mode was `AddNodeToSelection`, and the type of selected node
  output is `T`, the created definition is an extension method for type `T`, i.e. the function
  is defined as `T.<searcher input>`.
* A node in the currently displayed graph is created, which is a call of freshly created
  definition. If searcher mode was `AddNodeToSelection`, the selected node output became a self
  parameter of the call.

##### Entering Graph

Nodes in the Graph Editor can have associated definition which is called in their expressions. 
The information about it can be obtained from short value updates received from Language Server.
When one node is selected and user press Enter a new frame is pushed to execution context, being
a call with id of selected node's expression.

The current execution context is displayed as a bread crumbs panel on top of the Graph Editor.
The first crumb is a entry point definition name, and the next are names of called definition. When 
user click on the crumb the frames represented by crumbs after the clicked one are removed. The
crumbs of removed frames are grayed out, and then the user can click on them to restore all frames 
up to clicked one.

After each execution context change graph should display the definition of last call of execution 
context stack.

##### Input and Output Nodes

Each argument of definition displayed in graph editor is represented by input node. The node's label
is a parameter name. The Var with parameter name is used to determine and modifying connections
between the input node and the other nodes, analogously as it is with connection between ordinary
nodes.

The result of the displayed function is represented by output nodes. Therefore the last line of
definition's block is treated in a special way:
* if it is assignment, it is displayed as ordinary node and there is no output node;
* if it is `Nothing`, it isn't displayed at all and there is no output node.
* if it is a chain of `,` operators, there is one output node for each chain's element. The
  output node's label and input span-tree (ports) are derived from the element AST;
* in any other case the last line is represented by single output port, whose label and span-tree
  is derived from AST of the line.

The user can add input and output ports using searcher. If the searcher has mode `General` and its 
context is `None`:
* if the Searcher input is a valid parameter name, there is a suggestion in _Actions_ category
  _Add input node &lt;searcher input&gt;_. Choosing this suggestion hides searcher and add an
   input parameter to the definition. TODO how to define type of parameter?
* if the Searcher input is empty, there is a suggestion in _Actions_ category _Add output node_.
  Choosing this will adjust definition code in such way that the new output node appears with an
  expression `_`.

TODO: How do we define order of parameters and outputs?



### Modules Management



### Labeled Empty Ports



### Caching Searcher Suggestions



### Text Editor for Definition



### Searching Documentation



### Learning guide



### Expanded Nodes and Panels



### Explore Community Scenes
