# NEXT RELEASE
TODO: dodac daty
TODO: https://github.com/fkirc/skip-duplicate-actions/#skip-duplicate-workflow-runs

This is a minor update exposing more control over the anonymous data collection. You are now allowed 
to run the application with `--no-errors-data-collection`, and `--no-data-collection`. The options 
disable the collection of errors data (keeping the collection of other events), and disables the 
collection of all data, respectively. To learn more what data is collected in Enso alpha releases, 
read the notes of the `Enso 2.0.0-alpha.1` release.



# Enso 2.0.0-alpha.1
This is the first release of Enso, a general-purpose programming language and environment for 
interactive data processing. It is a tool that spans the entire stack, going from high-level 
visualization and communication to the nitty-gritty of backend services, all in a single language.

## Anonymous Data Collection
Please note that this release collects anonymous usage data which will be used to improve Enso and 
prepare it for a stable release. We will switch to opt-in data collection in stable version 
releases. The usage data will not contain your code (expressions above nodes), however, reported 
errors may contain brief snippets of out of context code that specifically leads to the error, like 
"the method 'foo' does not exist on Number".

**The following data will be collected**:
- Session length.
- Graph editing events. An event when nodes are created, deleted, moved, connected, disconnected, 
  collapsed, or edited. This will not include any information about node expressions used.
- Navigation events. An event when the view is changed to show an inner or outer node scope.
- Visualization events. An event when visualization is opened, closed, or switched. It does not 
  include the data contained in the visualization or the rendered visualization itself.
- Project management events. An event when a project was renamed.
- Errors, including errors about IDE crashes, WASM panics, Project Manager errors, Language 
  Server errors, Compiler errors, etc.
- Performance statistics. Minimum, maximum, average GUI refresh rate (frames per second).
