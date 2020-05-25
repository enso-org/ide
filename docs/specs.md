# IDE Projects Scope and Estimation



## Unsaved Projects and Handling Lost Connection

### Story

When opening IDE (or creating new project in future) we should have opened project with "unsaved" state. Such project have no name and it should not be persisted when closed. 

IDE should not require the connection to Project Manager to being able to display initial unsaved project. Similarily losing connection should not disallow the user from working,  however operations requiring communication with Engine will be not available.

When there is no connection with Engin, an alert should be displayed on the screen. IDE should constantly try to reconnect with Engine and Language Server. When connection will be reestablished, all changes which were done in IDE should be applied in Engine.

### Assumptions

* Engine will have API for opening unsaved project.



## Node Searcher

### Story

The Searcher panel appears in two cases:
* when user starts to edit node - the node became an Searcher input and panel is displayed below, 
* when user press tab with mouse over Graph Editor Panel - the new Seracher input appears with pane below.

The Searcher panel has its _mode_ which is determined when it is shown:
* when the existing node is editer, the mode is `EditNode` parametrized by the id of edited node.
* when the searcher is shown by pressing tab, the mode is
  * `AddNodeToSelection` if exactly one node is selected on scene. This mode is parametrized by selected node's id.
  * `General` (TODO maybe another name? Because it's not neccesarily about nodes) otherwise.

When the mode is `AddNodeToSelection` then there should be displayed connection between node and the Searcher input in the same way as nodes are connected.

Additionally the Searcher have _context_ which may change over time:
* `FunctionChosen` parametrized with function pointer, indicating that user have chosen the sugestion with associated _method pointer_. See _Node Suggestions_ section.
* `None` in any other case.

The displayed content in Searcher panel depends on its _mode_, and current input.
* When mode is `General` and the text input is empty, the Searcher Panel should display Main Menu which consist of tiles with associated action. Actions may be run by mouse click on appriopriate tile.
* Otherwise the Searcher Panel should display a list of suggestions. The exact content of the list depends on _mode_, current input and _context_. 

Each suggestion have a label and an icon. One of the list elements may be selected: initially the first one. The user changes selection by using arrow keys or by moving mouse pointer over elements. When pressing arrow up when the first element in is selected the selection is removed at all. When none element is selected, pressing arrow down select the first element.

#### Node Suggestions

The first category of suggestions displayed in Searcher are possible expresions which may be added to the expression in Searcher input. 

In this section we define two parts of searcher input for convenience: _expression_ and _pattern_.
* If user did not edit text input after he started editing or picked any suggestion (by pressing Tab), the whole input is an _expression_ and _pattern_ is empty.
* Otherwise the input is parsed to Prefix Chain. The last element of chain become _pattern_, and the rest become _expression_. If there is no other element, the _expression_ is empty.

When suggestion list needs to be created or updated, the engine method described in _Assumptions_ section is called:
* First argument:
  * If the searcher's mode is `AddNodeToSelection` and _expression_ is empty, the first output port with Var is passed as "self" variant.
  * Otherwise the _expression_ is passed as "expression" variant.
* Second argument: If the searcher context is `FunctionChosen` the parameter of this context is passed.

The returned list is then filtered by _pattern_ and then displayed in "Node" section of Searcher's suggestions list. When one of those suggestions is selected, and it has documentation, the documentation panel is visible on the right side of suggestion lists. There are following actions avialable for elements if this list:
* When user press Tab, the _pattern_ part (defined above) of searcher input is replaced with selected suggestion. The carret is put at the input end. If the suggestion has assigned method pointer, the Searcher context should be changed to `FunctionChosen` with this pointer as parameter.
* When user press Enter, or do mouse click on suggestion, it is applied to Searcher input as user would press Tab, and:
  * if searcher mode was `EditNode`, the expression of edited node is set to the searcher input;
  * otherwise the new node is created in the position of searcher, with searcher input as an expression.

### Assumptions

The engine implements an method for getting list of suggestions.
The function takes:
* One of:
  * an expression (string) to which we want to attach further expression,
  * a Var which should be put as a self argument of returned suggestions.
* An optional _method pointer_ being a definition called in the passed expression.

It returns:
* A list of suggestions. Each suggestion consists of:
  * an expression (string). It should contain self argument if it was specified,
  * icon,
  * optional method pointer which indicates the definition which will be called by the expression when this suggestion will be applied,
  * optional documentation in html format (?).
* An optional information about expected type of the expression which will applied to the expression specified in input (So e.g. IDE can display FileTree next to list of suggestions).

#### Estimate



## Project Operations

#### Story

In the Main Menu displayed in Searcher there should be a tile labeled "Recent Projects". After clicking it, the list of recent projects obtained from Engine is displayed. When user chose the project, the current project is closed and the chosen one is opened.

During closing project, the information about currently displayed definition is stored in IDE config in project directory. When the project is opened, this definition should be read from file and shown in Graph Editor. If for some reason the definition is gone, Graph Editor should show the main function in Main module. If such definition/module also does not exists, it should be recreated with appriopriate alert displayed on screen.

The another actions available in Main Menu are: 
* New project - closes the current one and opens new unsaved project
* Save project - in case the currently opened project is unsaved. It prompts for project name.

#### Estimate



## File Browser

### Story

FileBrowser is a panel containing the file listing of the root Project directory. The root Project directory is a first contentId returned by project/open method of Language Server. The listed files have icons. When user clicks in directory, the analogous panel shows on the right side, with listing of chosen directory, then the user can click an directory in the new panel, and the next panel arrives, etc. When chosing another directory in some panel, all panels to the right are hidden and one is shown, with the newly selected directory content.

The FileBrowser should appear in Searcher above the suggestion list and below the Searcher input in the following situation:
* The suggestion provider for nodes received information from Engine, that the expected type is `FilePath`, and
* the _pattern_ part of the Searcher input is empty.

If user choose file in this FileBrowser, the file path should be appended to the expression in Searcher input.





## Multiple Definitions

### Story

#### Create Definitions

The suggestion list on searcher has a new element in `Actions` category, labeled _Create function &lt;searcher input&gt;_. The action is available when:
* One of this condition is met:
  * the searcher mode is `General`;
  * the searcher mode is `AddNodeToSelection` and the type of selected node output is known;
* the searcher context is `None`,
* and the current Searcher input contains a valid function name.

When this suggestion is chosen (by mouse click, or pressing Enter having it selected):
* The Searcher panel disappears
* A new definition in the module of currently displayed graph is created, whose name is took from searcher input.  If searcher mode was `AddNodeToSelection`, and the type of selected node output is `T`, the created definition is an extension method for type `T`, i.e. the function is defined as `T.<searcher input>`.
* A node in the currently displayed graph is created, which is a call of freshly created definition. If searcher mode was `AddNodeToSelection`, the selected node output became a self parameter of the call.

#### Entering Graph

Nodes in the Graph Editor can have associated definition which is called in their expressions. The information about it can be obtaned from short value updates received from Language Server. When one node is selected and user press Enter a new frame is pushed to execution context, being a call with id of selected node's expression.

The current execution context is displayed as a bread crumbs panel on top of the Graph Editor. The first crumb is a entry point definition name, and the next are names of called definition. When user click on the crumb the frames represented by crumbs after the clicked one are removed. The crumbs of removed frames are grayed out, and then the user can click on them to restore all frames up to clicked one.

After each execution context change graph should display the definition of last call of execution context stack.

#### Input and Output Nodes

Each argument of definition displayed in graph editor is represented by input node. The node's label is a parameter name. The Var with parameter name is used to determine and modifying connections between the input node and the other nodes, analogously as it is with connection between ordinary nodes.

The result of the displayed function is represented by output nodes. Therefore the last line of definition's block is treated in a special way:
* if it is assignment, it is displayed as ordinary node and there is no output node;
* if it is `Nothing`, it isn't displayed at all and there is no output node.
* if it is a chain of `,` operators, there is one output node for each chain's element. The output node's label and input span-tree (ports) are derived from the element AST;
* in any other case the last line is represented by single output port, whose label and span-tree is derived from AST of the line.

The user can add input and output ports using searcher. If the searcher has mode `General` and its context is `None`:
* if the Searcher input is a valid parameter name, there is a suggeston in `Actions` category _Add input node &lt;searcher input&gt;_. Chosing this suggestion hides searcher and add an input parameter to the definition. TODO how to define type of parameter?
* if the Searcher input is empty, there is a suggestion in `Actions` category _Add output node_. Chosing this will adjust definition code in such way that the new output node appears with an expression `_`.



## Modules Management



## Labeled Empty Ports



## Caching Searcher Suggestions



## Text Editor for Definition



## Searching Documentation



## Learning guide



## Explore Community Scenes



