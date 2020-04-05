# General Rules
This document describes language rules that are relevant to connection discovery
and operations. The purpose is not to specify the whole language. Just enough
for IDE team members to be able to reason where identifiers are introduced and
what entity identifier usage refers to.

This is the base allowing IDE to describe what connections are in the displayed
graph. The covered topics are mostly identifiers, scopes and their interactions.


## Identifier
Identifier is a name that denotes a value, i.e. that is bound to a value. The
compiler also keeps track of its type, i.e. the set of possible values. Syntactically we recognize:
* variable names, being names that do not contain upper-cased characters, like `foo2`
  or `make_new`;
* constructor names, which are like variables but with first character and every
  character directly following underscore upper-cased (e.g. `Foo2` or
  `Make_New`);
* operator names, being built solely from operator symbols (e.g. `+` or `<$>`).
  Specifically, an operator name may contain only characters from the following
  set: `!$%&*+-/<>?^~|:\,.()[]{}=`. Not every sequence of these characters is a valid operator name, as they can collide with other language constructs.
  
Any other names that do not match requirements above, like `HTTP`, `foO` or
`Make_new` are not allowed and their behavior will not be specified.

In non-pattern contexts, referring to the existing identifier is
case-insensitive. So `foo` can be referred to either as `foo` or `Foo` with no
difference.  In pattern context, lower-cased names are used to introduce a
binding (or a constraint), while upper-cased name will refer to an already bound
identifier. In short, variable name is allowed to shadow, while constructor 
always unambiguously refers to an externally visible identifier.

Operator names behave either as:
* variable names when placed in a prefix position (e.g. `+` in `+ ab`);
* or as constructor names when placed in an infix position (e.g. `,` in `a,b`).

Number and text literals (like `5` or `"Hello"`) are treated as
constructor names.

Identifiers can be introduced by a binding (e.g. using assignment or lambda
argument matching) or when adding type constraints (using type ascriptions). 

## Scope
Scope is the span in the code which shares the identifiers set of visible,
available identifiers. Scopes are a span-tree structure, covering the whole program code.

Nested scope is allowed to:
* access identifiers from the outer scope;
* shadow identifiers from nested scope with a new binding;
* introduce new constraints on the identifiers from parent (or own) scopes.

The same identifier may be bound multiple times in the same scope (overloading).
It is allowed only for method overloads that differ in the type of the `this`
parameter. This limitation may be relaxed in the future, if proper motivating use-cases are found.

The identifier is always accessible scope-wide, both before and after the line
introducing it. Some monadic contexts (like IO) can introduce order-dependent
behavior, however. This is not something that IDE is (or can be) concerned about
when figuring out connections.

Identifier is bound by using a variable name in the pattern context. Exact behavior depends on the language construction that was to introduce the identifier.

Identifier introduced into a scope is visible only in the scope's subtree
(lexical scoping). 

Scopes, in the core language, are introduced by:
* module (file), i.e. the root scope;
* code blocks, i.e. indented blocks that follow a trailing operator;
* arrow (lambda) `->` operator for its right-hand side.

Some other constructs seemingly introduce scope (like function definitions) but
this is because they are desugared into some construct that introduces scope (like lambdas).

TODO: Consider if there are any special rules for signatures on definitions, or
is this just type ascription lying next to a definition.

### Examples
Example:
```
main =
    succ = a -> a+1
    succ 0
```

The sample code is a module with a single top-level definition. 

Here we have four scopes, each one nested in the previous one:
* root scope (consisting of unindented lines), where `main` is visible;
* definition and code block (all lines equally indented after
  `main =`) scopes, where both `main` and `succ` are visible
* lambda body scope (right-hand side after `a ->`), where all `main`,
  `succ` and `a` are visible.

Example:
```
test = a -> b ->
    sum = a + b
```

Here we have root scope, then scopes of lambdas (after `a ->`
and after `b ->`) and finally scope for code block.


Example:

```
main = 
    foo : a
    foo = 2

    bar : a
    bar = 3
```

While `main` as root-level has its own scope (as a definition in root it is
treated as method and desugared to lambda), `foo` and `bar` do not. `a`
introduced by type signatures belongs to the `main`'s scope, and is shared by
both nested definitions.

## Patterns
Patterns are scopes in the code where variables can be used to introduce new
identifiers into some scope. Constructors names are used to pattern match
against and potentially destructure more complex values.

The following scopes are treated as patterns:
* left-hand side of assignment operator, e.g. `main` in  `main = println
  "Hello"`;
* right-hand side of a colon operator, e.g. `a` in `foo:a`;
* left-hand side of an arrow operator, e.g. `a` in `a -> a + 1`.

Details will follow with description of these operators.

// TODO What about `case … of` ? Are there any other pattern-introducing constructs?

# Introducing identifiers
Common notation used in the examples uses French quotation marks as following:
* `«name»` for names introduced into the graph's scope. They are potential
  source endpoints of the connections in the graph.
* `»name«` for names used from graph's scope. They are potential destination
  endpoints of the connections in the graph.

Before "executing" code, these «» markers should be removed. They are just to
quickly convey expected results in the code sample content, and not repeat
"introduces `name` and uses `name2`" for each line description.


## Assignment
The assignment operator `=` is deeply magical. Its basic form is `name =
body`. It introduces `name` into the parent scope, by binding it with a value `body`.

Example:
```
«five» = 5
```

The name `five` introduced into the parent scope is bound to a value of
expression `5`.

Assignment operator is also used to define functions, extension methods and
perform pattern matching. For different cases appropriate desugaring is
applied. See sections below for details for particular details.

Roughly speaking, if a name is a variable name, it is introduced and its
arguments (if present) are visible only in the definition body. If the name is
constructor name though, it will pattern-match on variables in its arguments positions.

If macros are used in the definition, it is assumed that if it appears in a
pattern context, variable names it matched shall be bound — or otherwise just
used from the containing scope.
Basically, it is similar to if a grouped expression with tokens matched by a
macro was in place.

In any place where variable name is used in a pattern, it can be substituted by
an underscore `_` to disregard the value without introducing any identifier.

A single line can contain at most one assignment. The following code is not
valid: 
```
foo = bar = baz # invalid!
```


If a name introduced by an assignment is already available in the parent scope,
it becomes shadowed.

Example:
```
«foo» = 2
bar =
    «foo» = 5     # shadowing
    a = »foo« + 5 # refers to `foo` from line above
# here `foo` is `2` again
```

If the name was already assigned to in the current scope, it is not allowed to
bind it again. 

Example:

```
«foo» = 5
«foo» = 5 # error, symbol defined twice in the same scope
```

The excpetion to this rule are function overloads, described in a separate
section later.

### Non-trivial cases overview
These include functions (variable name followed by arguments), pattern matching
(constructor name with optional arguments) and extension methods. Each of these
is described in a greater detail below, here are just a few quick examples.

---

```
«foo» a b = a + b
```

Only the "base" name (of the prefix application chain) is introuced. Arguments
are visible in the body scope. Therefore, `a` and `b` in the body scope refer to
the function arguments and not to variables from the parent scope.

---

```
Foo «a» «b» = »bar«
```
Here we perform pattern matching to introduce `a` and `b`, fields of constructor
`Foo`. The `bar` refers to some identifier from the parent scope which should be
already defined.

---

```
a.«hello» = »print« "Hello"
```

Introduces name `hello` being an extension method defined on `a`. In this
position `a` will denote practically "any type" but is visible only in the
definition body (as it appears as the type of implicit `this` parameter).

--- 
TODO:
Example:
```
foo = a ->
    a = 5
```
TODO: Does the `a = 5` is shadowing `a` from lambda argument? Or is this
multiple definition error? If the block introduces scope, it should shadow. If
however lambda body is THE scope where argument is introduced and is same as
lambda code block, it should be an error. 
Or should it get some special rule?

---

### Function definitions
If the assignment's left-hand side is a prefix application chain, where the
left-most element (the callable) is a variable name, the assignment is said to
be a function definition. Each prefix argument is converted into a lambda
argument in the assignment body.

```
«log_name» object = »print« object.»name«
```
is desugared into:
```
«log_name» = object -> »print« object.»name«
```
which in turn can be desugared into:
```
«log_name» = object -> »print« (»name« object)
```

This desugaring shows why only `log_name` is introduced into the scope, while
`object` is visible only in the definition's body.

If the operator appears in the function name position, it can be defined as
well:
```
«^» a n = a * a ^ (n - 1)
```

This introduces name `^` into the scope. It uses already defined `*` and `-`
operators. (to avoid clutter the operators are not marked with »«)

### Pattern matching
If the assignment's left-hand side is a prefix application chain where the
function is a constructor name, it will be desugared into a pattern match.

Example:
```
»Some« «value» = »get_opt«
```

will be desugared into:

```
«value» = case »get_opt« of
  »Some« b -> b
  _      -> error 
```

Therefore, only `value` will be introduced into the parent scope. `Some` and
`get_opt` must be defined, the former being an atom with at least single field.

Using operators in the infix position will also attempt to pattern match its
operands. For example:
```
«x»,«y» = »get_position«
```

TODO Describe what it gets desugared to when there is more than one variable
name introduced? Now it is inconsistent, if we say that only canonical form of
assignment is `variable_name = value`.

### Extensions methods
If the application target uses accessor operator `.`, e.g. `Int.add`, the last
segment of target is the introduced indentifier and the previous segments are
used to type the implicit `this` parameter. 

For example:
```
»Foo«.«bar» = 5
```
translated to:
```
«bar» this:»Foo« = 5
```

Which is then desugared into a lambda. The introduced name is only `bar`.

If there are any prefix application arguments following the accessor-style
target, they will be treated as arguments following implicit `this`.

### Overloading
Only the methods that take `this` as the first parameter can be overloaded. Each
overload of the given name must have different type of `this`.

However, the type of `this` will be often inferred by the typechecker and it
IDE cannot tell if given overloads are valid or not.

Example:

```
«foo» this:«a» = »body1«

«foo» this:«b» = »body2«
```

In this case `a` and `b` for each `foo` definition will be inferred by the
compiler. If they end up being different types, overloads are valid. If they are
the same, an error will be raised.

### Root scope assignments

Any binding in the rootscope gets an implicit `this` parameter that describes a
module. Example:
```
main = print "Hello"
```

`main` here is a function binding that is desugared to lambda. As such its body
has its own, separate scope. That would not have been a case, if such lines
appeared in any non-root code block.

## Lambdas
`arg -> value` is the syntax for lambdas. Left-hand side is a pattern for the
argument (lambdas are always unary) and the right-hand side is its body.
Right-hand side, i.e. the lambda body, introduces its own scope.

The `->`'s pattern introduces identifiers only into the scope of the right-hand
side — if the lambda is not introduced in what is already a pattern context.

Example:
```
«succ» = a -> a + 1
«foo» = »a«
```

Here lambda introduces `a` only into its right-hand side. The `a` that is being
assigned to `foo` is not the same `a` as in lambda — it must be defined
elsewhere in the module or the code will fail.

However, if `->` appears in a pattern context, its left-hand side identifiers
are introduced into the scope targeted by the outer pattern context.

Example:
```
(a -> b) -> a.»default«
```

If not for this second rule, the `a` would be visible only in place of
expression `b`. However, now it is visible in the outer lambda body and can be
accessed. The only externally provided identifier must be `default` method.

--- 
Lambdas may not appear in the assignment's pattern (i.e. values cannot be
pattern-matched into lambda).

So the following is not valid:
```
a -> b = foo
```

---
Example:
```
a -> a -> b
```

Here the first `a` and second `a` are separate identifiers, the latter shadowing
the first one. If one wanted to express that both arguments are of the same
type, `a -> A -> b` would have been used. `b` refers to an identifier from
graph's scope (it is in the body's position, not pattern).

OPEN QUESTION: actually it might be "nice" to have both `a` unified in such
case. 



## Type ascription
The type ascription operator `:` introduces pattern scope for its right hand
side. The basic form is `value:type`. It says that `value` be of the given
`type`, i.e. that all its possible values belong to the set of atoms represented
by type.

The effect of this can be two-fold. If `value` is of (at least partially) known
type, appropriate constraints will be introduced on the types denoted by
variable names appearing in the pattern context. If the identifier was not
defined, it will appear in the current scope.

For example:
```
5 : «a»?
```

This introduces constraint on the type `a` that its value set must include atom
`5`. If the `a` is already visible in the scope, this constraint will be added.
Otherwise, `a` will be introduced into the current scope with that constraint.

// TODO: What if parent scope only ascribes identifier with type constraint but
only nested scope assigns to it? 

// TODO: Open design question, if `a` should modify existing variable or should
always try to shadow it. What is the difference between `5 : a` and `5 : A`?
(except the latter not being able to ever introduce a new identifier)

// TODO: Is it legal or sensible to ascribe a variable when there is no binding?


When type is known, the type ascription can be used to constrain type of the
value:

```
»a« : 5
```

This says that value of `a` is of type `5`. Type `5` has only a single allowed
value: `5`. This will tell compiler to error out if program tries to bind `5`
with any value that is not known to be `5`.

However, this example refers to some `a` already being visible in scope and does
not introduce any identifiers.

The `type` in this expression is pattern context and can be used to constrain
the type variables. It is legal to assign constraints on an identifier using `:`
multiple times in any of the scopes where identifier is visible.

Signatures are just type ascriptions that happen to preceed the assignments.
They have no special rules currently defined. This area needs further design
work.

TODO: Open question: does empty type (`Void`) exists in the language?

TODO signatures and their relation with scoping. Difference for root and
non-root definitions.

Examples:

```
add : a -> a -> a
add a b = a + b
```

TODO: 
* With current rules `a` from the signature gets introduced into parent scope
  will be unified with other uses of `a` in other definitions.
* or, actually, we want this to happen only in the root scope. When in
  definition body, the `a` actually should be unified between signatures.
  Doesn't sound that clean though.
* Does argument-introduced `a` shadows the signature-introduced `a`?

TODO: `b:A` — does this add constraint to `a` or just `b` ? 

# Current engine limitations
Note: "current" means "in the scope of the first alpha release of enso",
not "at the moment of writing this document". 

## Extension methods
The extensions methods (taking `this` as the first parameter) can be defined
only using the sugared syntax.

While both 
```
foo this:a = print "hello"
```

and 

```
a.foo = print "hello"
```

are equivalent, engine currently supports only the latter.

IDE can assume that all extension methods will be introduced using the
`Type.name` syntax sugar.

// TODO what if non-first argument is named `this` ? Is the magic happening only
for this particuar name? Is it sensitive for its position in the arguments list?
// TODO What happens if the `this`-taking function in defined in the root scope
where already `this` is implicitly provided? What about taking `this` in a
method defined using the sugared syntax? (e.g. `Int.print this:Int = ...`)


## Type ascription
The type ascription and signatures are not properly supported. IDE should
disregard them for the time being.


# Node Connections in IDE
IDE presents a definition body as a graph. Code lines of the body of the
definition are displayed as nodes (unless they're definitions).

We want to display connection between nodes, if an identifier introduced by one
node into the graph's scope is used in another node's expression.

## Connection
Connection is an ordered pair of endpoints: source and destination. Endpoint is
pair of node ID and crumbs. Source endpoint identifiers the node which
introduces the identifier (source of data), and crumb describes the identifier
position in the node's assignment's left-hand side. Destination endpoint
similarly describes position in node's expression body where the identifier is used.

Later, higher layers will GUI shall merge this information with the "span tree"
describing the structure of the node's pattern and body. The low level
"double-representation" deals only with AST and is not concerned with view-level
datastructures like expresion span-stree.


## Discovery rules
Connections when identifiers from graph scope are used in node expressions. They
are between node that introduces identifier and node that uses identifier.


Some simplifications are currently assumed:
* Connections care only about usage of symbols introduced by assignment
  definition. For example, symbols introduced by `:` operator's right side do
  not form connections. Same for lambda arguments.
* We care only about identifiers introduced into graph's scope: anything that
  appears in subscopes can be disregarded. However, IDE must be aware of
  shadowing to properly tell if an identifier usage actually refers to an
  identifier from graph's scope.
* There is no graph for the module's root scope, so any special rules for the
  root scope might be irrelevant.
* IDE is concerned about producing correct results for correct programs. It does
  not care about diagnosing ill-formed or "not yet supported" programs, quite
  the opposite. We want to keep output as similar to the correct one as
  possible. (we will often visualize programs that are in progress of editing).
* For the first release IDE can disregard the type ascription operator (`:`). 

// TODO: Specify what is exactly the graph's scope? Is this a lambda body scope
or the scope introduced by the block following the lambda? Likely both of these
should be somehow coalesced, to avoid issues with definitions with inline bodies.

// TODO: Actually, can we display graphs for argument-less blocks being node
bodies? Scoping could get quite strange then.

Basically, the problem can be reduced to being able to describe for any line in
code block the list of identifiers it introduces into the graph's scope and the
list of identifiers from graph's scope that it uses.

If the identifier is introduced by assignment's left-hand side and is used in
the other node's expression, the connection should be recognized.


## Connection operations
// TODO: complete in future, when implementing them

Because definitions can be sensitive about their order (e.g. because of IO
monadic context), when creating connections, lines should be reordered to match
the order of topologically sorted nodes from the graph. (when possible)

In future this behavior should be depend on definition's monadic context
provided by the language server.

