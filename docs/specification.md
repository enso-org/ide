# IDE Features Specification

The goal of this document is to provide specification of Enso IDE features in such detail, that
one's could reliable plan work and estimate all tasks required to implement such featire in IDE.

### Note about Estimates

Each estimate in this document is expressed in _development days_ which is roughly the number of 
8-hours working days a senior developer need to deliver this feature **assuming he does not
work on anything else**. So the estimate includes developing, dev-testing, writing unit test and
review of another developer. It should contain also the buffer for possible unexpected problems
related to the feature. The estimate does not include design talks, meetings, fixing old bugs, etc.

[This spreadsheet](https://docs.google.com/spreadsheets/d/1_FfeCXc7TZESKsrfHFihaCWDOz-LBVQNLYsTv_ptXhc/edit#gid=1520864697) 
may be used if you need express the time required for some set of features in the real team sprints.


## Planned Releases

Community Release:
* [Unsaved Projects and Handling Lost Connection](#unsaved-projects-and-handling-lost-connection)
* [Node Searcher](#node-searcher)

Business Release:
* [Project Operations](#project-operations)
* [File Browser](#file-browser)
* [Multiple Definitions](#multiple-definitions)
* [Modules Management](#modules-management)
* [Labeled Empty Ports](#labeled-empty-ports)
* [Caching Searcher Suggestions](#caching-searcher-suggestions)
* [Text Editor for Definition](#text-editor-for-definition)
* [Searching Documentation](#searching-documentation)
* [Learning Guide](#learning-guide)
* [Explore Community Scenes](#explore-community-scenes)



### Unsaved Projects and Handling Lost Connection

Estimate: 24 days

#### Story

When opening IDE (or creating new project in future) we should have opened project with "unsaved" 
state.

IDE should not require the connection to Project Manager to being able to display initial
unsaved project. Similarly, losing connection should not disallow the user from working, but
operations requiring communication with Engine will not be available.

When there is no connection with Engine, an alert should be displayed on the screen. IDE should
constantly try to reconnect with Engine and Language Server. When connection will be
reestablished, all changes done in IDE should be applied in Engine.

In case of lost connection with Language Server, if the reconnecting does not succeed, the
project should be reopened using Project Manager.

#### Assumptions

* Engine will have API for opening unsaved project.



### Node Searcher

Estimate: 38 days
* View:
  * Design the searcher panel (different views): 13
  * Create FRP connections: 8
  * Animations: 5
* Logic:
  * Obtaining list from Engine: 5
  * Filtering list: 5
  * Inserting/updating node: 2

#### Story

The Searcher panel appears in two cases:
* when user starts to edit node - the node became Searcher input and panel appears below, 
* when user press tab with mouse over Graph Editor Panel - the new Searcher input appears with
  Searcher panel below.

The way the Searcher panel is brought to the screen affects its look and behaviour, therefore we
introduce Searcher _mode_ term, which is:
* _Node Edit_ when the existing node is edited.
* _Add to Selection_ if when the new input appears by pressing Tab, and exactly one node is
  selected on the scene.
* _General_ in any other case.

When the mode is `_Add to Selection_ then there should be displayed connection between node and
the Searcher input similar to the connections between nodes.

Additionally, the Searcher have _context_ which may change over time:
* _Function Chosen_ with assigned _method pointer_, indicating that user have chosen the
 suggestion with associated _method pointer_. See [Node Suggestions](#node-suggestions).
* `None` in any other case.

The displayed content in Searcher panel depends on its _mode_, and current input.
* When mode is _General_ and the text input is empty, the Searcher Panel should display main menu
  whose entries are arranged in tiles.
* Otherwise, the Searcher Panel should display a list of suggestions. The exact content of the
  list depends on _mode_, current input and _context_. 

Each suggestion have a label and an icon. One of the list elements may be selected, initially
the first one. The user changes selection by using arrow keys or by moving mouse pointer over
elements. When pressing arrow up when the first element in is selected the selection is removed
entirely. When none element is selected, pressing arrow down select the first element.

##### Node Suggestions

The first category of suggestions displayed in Searcher are possible expressions which may be
added to the expression in Searcher input. 

In this section we define two parts of searcher: _expression_ and _pattern_.
* If user did not edit text input after he started editing or picked any suggestion, the whole
 input is an _expression_ and _pattern_ is empty.
* Otherwise, the input is parsed to Prefix Chain. The last element of chain become _pattern_, and
  the rest become _expression_. If there is only one element, the _expression_ is empty.

When suggestion list needs to be created or updated, the engine method described in _Assumtions_ is
 called:
* First argument:
  * If the searcher's mode is _Add to Selection_ and _expression_ is empty, the first output
    port with Var is passed as "self" variant.
  * Otherwise, the _expression_ is passed as "expression" variant.
* Second argument: If the searcher context is _Function Chosen_ the method pointer assigned to this
  context is passed.

Then the returned list is filtered by _pattern_ and then displayed in "Node" section of Searcher's
suggestions list. When one of those suggestions is selected, and it has documentation, the
documentation panel is visible on the right side of suggestion lists. There are following
actions available for elements if this list:
* When user press Tab, the _pattern_ part (defined above) of searcher input is replaced with
  selected suggestion. The cursor is put at the input end. If the suggestion has assigned method
  pointer, the Searcher context should be changed to _Function Chosen_ with this pointer assigned.
* When user press Enter, or do mouse click on suggestion, it is applied to Searcher input as
  user would press Tab, and:
  * if searcher mode was _Node Edit_, the expression of edited node is set to the searcher input;
  * otherwise, the new node is created in the position of searcher, with searcher input as an
    expression.

#### Assumptions

The engine implements an method for getting list of suggestions. 

The function takes:
* One of:
  * an _expression_ (string) to which we want to attach further expression,
  * a Var which should be put as a self argument of returned suggestions.
* An optional _method pointer_ being a definition called in the _expression_.

It returns:
* A list of suggestions. Each suggestion consists of:
  * an expression (string). It should contain self argument if it was specified,
  * icon,
  * an optional method pointer which indicates the definition which will be called by the
    expression when this suggestion will be applied,
  * an optional documentation in HTML format (?).
* An optional information about expected type of the expression which will applied to the
  expression specified in input (So e.g. IDE can display File Tree next to list of suggestions).



### Project Operations

Estimate: 17

##### Story

In the Main Menu displayed in Searcher there should be a tile labeled "Recent Projects". After
clicking it, the list of recent projects obtained from Engine is displayed. When user chose the
project, the current project is closed and the chosen one is opened.

During closing project, the information about currently displayed definition is stored in IDE
config in project directory. When the project is opened, this definition should be read from
file and shown in Graph Editor. If for some reason the definition is gone, Graph Editor should
show the main function in Main module. If such definition/module also does not exist, it should
be recreated with appropriate alert displayed on screen.

The another actions available in Main Menu are: 
* New project - closes the current one and opens new unsaved project
* Save project - in case the currently opened project is unsaved. It prompts for project name.



### File Browser

Estimate: 15

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
 Searcher input.



### Multiple Definitions

Estimate: 40
* Create Definitions: 5
* Entering Graph: 8
* Input nodes: 11
* Output nodes: 8
* Ordering of input and outputs (UNSPECIFIED): 8

#### Story

##### Create Definitions

The suggestion list on searcher has a new element in `Actions` category, labeled _Create
function &lt;searcher input&gt;_. The action is available when:
* One of this condition is met:
  * the searcher mode is `General`;
  * the searcher mode is `AddNodeToSelection` and the type of selected node output is known;
* the searcher context is `None`,
* and the current Searcher input contains a valid function name.

When this suggestion is chosen (by mouse click, or pressing Enter having it selected):
* The Searcher panel disappears
* A new definition in the module of currently displayed graph is created, whose name is took
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



### Explore Community Scenes
