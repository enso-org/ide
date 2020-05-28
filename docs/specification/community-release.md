# Community Release Features Specification

* [Unsaved Projects and Handling Lost Connection](#unsaved-projects-and-handling-lost-connection)
* [Node Searcher](#node-searcher)
* [Project Operations](#project-operations)



### Unsaved Projects and Handling Lost Connection

[Estimate](./README.md#note-about-estimates): 26 days

#### Story

The IDE application can take a project name as a command line argument. With this option, IDE should
open the specified project, or create if it does not exist. When IDE is run without this option
we should see new _unsaved_ project.

IDE should not require the connection to Project Manager to being able to display initial
unsaved project. Similarly, losing connection should not disallow the user from working, but
operations requiring communication with Engine will not be available.

When there is no connection with Engine, an alert should be displayed on the screen. IDE should
constantly try to reconnect with Engine and Language Server. When connection will be
reestablished, all changes done in IDE should be applied in Engine.

In case of lost connection with Language Server, if the reconnecting does not succeed, the
project should be reopened using Project Manager.

If IDE is run with specified project name, but the connection to Project Manager and Language Server
failed, IDE displays an alert and opens new unsaved project.

#### Assumptions

* Project Manager API allows for:
  * creating/opening and saving _unsaved_ projects.
  * lookup projects by name.



### Node Searcher

[Estimate](./README.md#note-about-estimates): 38 days
* View:
  * Design the searcher panel (different views): 13
  * Create FRP connections: 8
  * Animations: 5
* Logic:
  * Obtaining list from Engine: 5
  * Filtering list: 5
  * Inserting/updating node: 2

#### Story

The Searcher panel appears when:
* when user starts to edit node - the node became Searcher input and panel appears below, 
* when user press tab with mouse over Graph Editor Panel - the new Searcher input appears with
  Searcher panel below.

The way the Searcher panel is brought to the screen affects its look and behaviour, therefore we
introduce Searcher _mode_, which may be one of the following variants:
* _Node Edit_ when the existing node is edited.
* _Add to Selection_ if when the new input appears by pressing Tab, and exactly one node is
  selected on the scene.
* _General_ in any other case.

When the mode is _Add to Selection_ then there should be displayed connection between the selected
node and the Searcher input.

Additionally, the Searcher have _context_ which may change over time:
* _Function Chosen_ with assigned _method pointer_, indicating that user have chosen the
 suggestion with associated _method pointer_. See [Node Suggestions](#node-suggestions).
* `None` in any other case.

The displayed content in Searcher panel depends on its _mode_, and current input.
* When mode is _General_ and the text input is empty, the Searcher Panel should display main menu.
  The Main Menu whose entries are arranged in tiles.
* Otherwise, the Searcher Panel should display a list of suggestions. The exact content of the
  list depends on _mode_, current input and _context_. 

Each suggestion has a label and an icon. One of the list elements may be selected, initially
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

When suggestion list needs to be created or updated, the engine method described in _Assumptions_ is
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

##### EnsoGL

EnsoGL library supports displaying rich line of text.

##### Engine

The engine implements a method for getting list of suggestions. 

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

[Estimate](./README.md#note-about-estimates): 17 days.

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
