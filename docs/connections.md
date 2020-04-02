# General Rules
This section attempts to describe how the identifiers are introduced and
resolved in the language.


## Definition
Definition is any binding introduced using assignment (`=` operator) syntax.
Definition may be optionally preceeded by a signature in a form `name:type`.

For example, the following is definition:
```
add : Int -> Int -> Int
add a b = a + b
```

or this is a definition with no signature:

```
five = 5
```

Definitions can be also introduced within a nested code block:
```
main =
    a = 2
    b = 2
    add a b = a + b
    add a b
```

Here `main` is a definition and has nested definitions of `a`, `b` and `add`.

### Term discrepancy with IDE codebase
Note: this document uses term "definition" in a wider term than usually IDE
codebase does. Here any assignment-binding is a definition, no matter if it has
any arguments or not. (i.e. this definition is either gui-definition or a gui-node)


## Scope
Scope is the span in the code which shares the bound identifiers set. Scopes can
be seen as span-stree, creating a hierarchical, nested structure.

Nested scope is allowed to:
* access identifiers from the outer scope;
* shadow identifiers from nested scope by introducing new bindings.

Otherwise, it is illegal to bind twice the same identifier within a scope.

// TODO weren't we supposed to support overloading?

// TODO what is duplicate definition and what is unification? Is there a
difference between `=` and `:` after all?

The identifier is bound scope-wide. It is visible and usable in the lines before
the actual binding occurs. Some monads (like IO) can introduce order-dependent
behavior, however. This is not something that GUI can tell by looking at AST.

Symbols introduced into a scope is not visible outside the scope's subtree.

Scopes are introduced by:
* module/file (the root scope);
* code blocks (i.e. the block that follows the line with an trailing operator);
* module-level definitions: for both signature (if present) and assignment-binding;
* `->` operator for its right-hand side.

Example:
```
main : a
main =
    succ = b -> b+1
    succ 0
```

The sample code is a module with a single top-level definition. 

Here we have four scopes, each one nested in the previous one:
* root scope (consisting of unindented lines), where `main` is visible;
* definition `main` scope, where `main` and `a` are visible;
* code block scope (all lines equally indented after
  `main =`), where both `main`, `a` and `succ` are visible
* lambda body scope (right-hand side after `a ->`), where all `main`,
  `succ`, `a` and `b` are visible.

Example:
```
test = a -> b ->
    sum = a + b
```

Here we have root scope, `test`'s scope, then scopes of lambdas (after `a ->`
and after `b ->`) and finally scope for code block.

// TODO what difference this make if code block introduces its own scope?


## Contexts
There are two kinds of context: pattern context and non-pattern context. 

Each position in code is either in a pattern context or not. By default code is
in non-pattern context. Pattern context is introduced locally by certain
language constructs:  `=`, `:` and `->` operators (i.e. definition bindings, signature type ascription and lambdas).

Indide pattern context each usage of a variable name (identifier starting with a
lower case letter) actually binds this identifier with whatever it is being
matched to. The bound identifier is visible and usable within the target scope.

// TODO are operator identifiers bindable? Likely they require special rules.

Pattern context always has the single target scope, where the identifiers are
introduced into. What is the target scope depends on the operator that
introduced pattern context. 

Pattern context is introduced within:
* left-hand side of assignment operator, e.g. `main` in  `main = println
  "Hello"`;
* right-hand side of a colon operator, e.g. `a` in `foo:a`;
* left-hand side of an arrow operator, e.g. `a` in `a -> a + 1`.

Both `=` and `:` introduce identifiers into the scope where they occur, as they
do not introduce any new scope of their own. 

The `->` operator introduces identifiers only into the scope of their right-hand
side, if the lambda is not introduced in what is already a pattern context.

Example:
```
succ = a -> a + 1
foo = a
```

Here lambda introduces `a` only into its right-hand side. The `a` that is being
assigned to `foo` is not the same `a` as in lambda — it must be defined
elsewhere in the module or the code will fail.

However, if `->` appears in a pattern context, its left-hand side identifiers
are introduced into the scope targeted by the outer pattern context.

Example:
```
(a -> b) -> a.default
```

If not for this second rule, the `a` would be visible only in place of
expression `b`. However, now it is visible in the outer lambda body and can be
accessed.

## Examples
Unless otherwise stated, it should be assumed that given examples are lines
occurring within a definition's body code block.


```
a -> a -> b
```

Here the first `a` and second `a` are separate identifiers, the latter shadowing
the first one. If one wanted to express that both arguments are of the same
type, `a -> A -> b` would have been used. `b` refers to an identifier from
graph's scope.

---


```
a -> b = c
```

If such line occurs on the top-level, `a` and `b` are introduced into the
definition scope. Otherwisee, they are introduced into the parent scope.

Does this introduce the `a` into the module's scope? 

(rules say "only if inline `=` does not introduce a new scope <=> on the top level)

---

```
a = Int
foo = 5:a
```

Does `a` in `5:a` refers to the previous line's `a` or is separate?

Marcin: na pewno nie shadowują, mogą się unifikowac lub kolidowac jako
redefinicja



Co jeżeli 
```
a = Int
foo = Int : a
```



# IDE Connection Discovery
IDE presents a definition body as a graph. Code lines of the body of the
definition are displayed as nodes (unless they're definitions).

We want to display connection between nodes, if an identifier introduced by one
node into the graph's scope is used in another node's expression.

Some simplifications are currently assumed:
* Connections care only about usage of symbols introduced by assignment
  definition. For example, variables introduced by `:` operator's right side do
  not form connections. Same for lambda arguments.
* we care only about identifiers introduced into graph's scope: anything that
  appears in subscopes can be disregarded. However, IDE must be aware of
  shadowing to properly tell if an identifier usage actually refers to an
  identifier from graph's scope.
* There is no graph for the module's root scope, so any special rules for the
  root scope might be irrelevant.
* IDE is concerned about producing correct results for correct programs. It does
  not care about diagnosing ill-formed programs, quite the opposite. We want to
  keep output as similar to the correct one as possible. (we will often
  visualize programs that are in progress of editing)
* For the first release IDE can disregard the type ascription operator (`:`). 


// TODO: what is graph's scope? Is this a definition's scope (if there's such
thing) or code block's scope?

Basically, the problem can be reduced to being able to describe for any line in
code block the list of identifiers it introduces into the graph's scope and the
list of identifiers from graph's scope that it uses.


## Connection

Connection is an ordered pair of endpoints: source and destination. Endpoint is
pair of node ID and crumbs. Source endpoint identifiers the node which
introduces the identifier (source of data), and crumb describes the identifier
position in the node's assignment's left-hand side. Destination endpoint
similarly describes position in node's expression where the identifier is used.







---

# TO BE REWRITTEN



TODO
W top levelu jaki jest dokładnie obszar scope'u definicji?
Czy obejmuje sygnaturę?

Czy może być wiele sygnatur do definicji?
Jak dać sygnaturę do czegoś co nie ma żadnej nazwy lub ma wiele nazw?

Różnica między pattern-matchingiem a typowaniem?
Różnica między `a = 5` oraz `5:a`. Co jest wartością, co jest typem?
Sygnatura bez definicji?