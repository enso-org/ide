# General Rules
This section attempts to describe how the identifiers are introduced and
resolved in the language.

## Identifier
Identifier is a name that may denote value or type. Syntactically we recognize:
* variables, being names starting with a lower-case character, like `foo` or
  `main`.
* constructors, being names starting with an upper-case character, like `Foo` or
  `Option`.
* operators, being special symbols like `+` or `<$>`.

In non-pattern contexts, referring to an existing identifier is
case-insensitive. So `foo` can be referred to as `Foo`.

In pattern context, lower-cased names are used to introduce a binding (or a
constraint), while upper-cased name will refer to an already bound identifier.

Operators behave as variables in prefix position (e.g. `+` in `+ a b`) or as
constructors in an infix position (e.g. `,` in `a,b`).

## Scope
Scope is the span in the code which shares the bound identifiers set. Scopes can
be seen as a span-stree structure, creating a hierarchical structure.

Nested scope is allowed to:
* access identifiers from the outer scope;
* shadow identifiers from nested scope by introducing new bindings;
* introduce new constraints on the identifiers from parent (or own) scopes.

The same identifier may be bound to multiple times in the same scope
(overloading). It is allowed only for method overloads that differ in the type of
the `this` parameter. 

The identifier is bound scope-wide. It is visible and usable in the lines before
the actual binding occurs. Some monads (like IO) can introduce order-dependent
behavior, however. This is not something that IDE is (or can be) concerned about
when figuring out connections.

Identifier is bound by using a variable-type identifier in the pattern context.
Exact behavior depends on the language construction that was to introduce the identifier.

Identifier introduced into a scope is visible only in the scope's subtree
(lexical scoping). 

Scopes are introduced by:
* module/file (the root scope);
* code blocks (i.e. the block that follows the line with an trailing operator);
* module-level definitions: for both signature (if present) and assignment-binding;
* `->` operator for its right-hand side.

### Examples


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


Example:

```
main = 
    foo : a
    foo = 2

    bar : a
    bar = 3
```

While `main` as root-level has its own scope, `foo` and `bar` do not. `a`
introduced by type signatures belongs to the `main`'s scope, and is shared by both
nested definitions.

## Patterns
Patterns are context in the code where variables can be used to introduce new
identifiers into some scope. Constructors (that also include literals) are used
to pattern match against and potentially destructure more complex values.

Pattern context is introduced within:
* left-hand side of assignment operator, e.g. `main` in  `main = println
  "Hello"`;
* right-hand side of a colon operator, e.g. `a` in `foo:a`;
* left-hand side of an arrow operator, e.g. `a` in `a -> a + 1`.

Details will follow with description of these operators.



## Assignment
The assignment operator `=` is deeply magical. Its basic form is `name =
body`, where it introduces `name` into the parent scope.

Example:
```
five = 5
```

Introduces the name `five` into the parent scope.

Assignment operator is also  used to define functions, extension methods and
perform pattern matching. For each of these cases appropriate desugaring is
applied. See sections below for details for particular cases.

Roughly speaking, if the name is a variable, it is introduced and its arguments
(if present) are visible only in the definition body. If the name is
constructor, it will be pattern-matched and any variables used 
for constructor arguments will be bound.

If any macros are used in the definition, it is assumed that if it appears in a
pattern context, their vars introduce variables — or otherwise use variables.
Basically, it is similar to if a grouped expression with tokens matched by a
macro was in place.

In any place where variable is used in the pattern, it can be substituted by
underscore `_` to disregard the value without introducing any identifier.


Examples:
```
foo a b = a + b
```
introduces name `foo`

---

```
Foo a b = bar
```
introduces names `a` and `b`

---

```
a.hello = print "Hello"
```
introduces name `hello`


### Function definitions
If the assignment's left-hand side is a prefix application chain, where the
left-most name (i.e. the function name) is a variable, the assignment is said to
be a function definition. Each prefix argument is converted into a lambda
argument.

```
log_name object = print object.name
```
is desugared into:
```
log_name = object -> print object.name
```

This desugaring shows why only `log_name` is introduced into the scope, while
`object` is visible only in the definition's body.

If the operator appears in the function name position, it can be defined as
well:
```
^ a n = a * a ^ (n-1)
```

### Pattern matching
If the assignment's left-hand side is a prefix application chain where the
left-most name is a constructor, it will be desugared into a pattern match.

Example:
```
Some value = get_opt
```

will be desugared into:

```
value = case get_opt of
  Some b -> b
  _      -> error 
```

Therefore, `value` will be introduced into the parent scope.

Using operators in the infix position will also attempt to pattern match its
operands. For example:
```
x,y = get_position # introduces `x` and `y`
```

### Extensions methods
If the application target uses accessor operator `.`, e.g. `Int.add`, the last
segment is the introduced indentifier and the previous segments are used to type
the implicit `this` parameter.

For example:
```
Foo.bar = 5
```
translated to:
```
bar this:Foo = 5
```

Which is then desugared into a lambda. The introduced name is only `bar`.


## Lambdas
`arg -> value` is the syntax for lambdas. Left-hand side is a pattern for the
argument (lambdas are always unary) and the right-hand side is its body. Lambda
body has its own scope.

The `->` pattern introduces identifiers only into the scope of their right-hand
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



TODO

## Type ascription
The type ascription operator `:` introduces pattern scope for its right hand
side. The basic form is `value:type`. The type identifiers used in the
right-hand side will be constrained to include appropriate values in their value
set.

It is legal to assign constraints on an identifier using `:` multiple times in
any of the scopes where identifier is visible.

When variable name appears in type pattern, the type denoted by this identifier
will be required to contain given `value`. If variable does not denote any type
visible in the scope, the identifier will be introduced into the current scope.

TODO: Open design question — perhaps variables after `:` should be only allowed
to introduce new identifiers but not to constrain existing ones.


TODO examples:
```
a : 5
A : 5
5 : a
5 : A
```

TODO: Open question: does empty type exist? Apparently it makes more sense in
lazy languages, rather then strict ones.


TODO TODO TODO


TODO signatures and their relation with scoping. Difference for root and
non-root definitions.

Examples:


```
add : Int -> Int -> Int
add a b = a + b
```

---

## Current engine limitations
Note: "current" means "in the scope of the first alpha release of enso",
not "at the moment of writing this document". 

### Extension methods
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

// TODO what if non-first argument is named `this` ? Is the magic happening only
for this particuar name?


### Type ascription
The type ascription and signatures are not properly supported. IDE should
disregard them for the time being.




# IDE Connection Discovery
IDE presents a definition body as a graph. Code lines of the body of the
definition are displayed as nodes (unless they're definitions).

We want to display connection between nodes, if an identifier introduced by one
node into the graph's scope is used in another node's expression.

Some simplifications are currently assumed:
* Connections care only about usage of symbols introduced by assignment
  definition. For example, symbols introduced by `:` operator's right side do
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
---
---






# TO BE REWRITTEN
---

# TO BE REWRITTEN


## Assignments
Assignment operator is used to define identifiers. Its left-hand side is a pattern
context. Pattern context means that usage of a variable name (identifier
starting with a lower case letter) actually binds this identifier with whatever
it is being matched to. The bound identifier is visible and usable within the
target scope.

When upper-cased variable is used in a pattern context, it must refer to an
existing identifier and will perform pattern matching. Example:

```
Some a = foo
```

This introduces `a`, while using `Some` and `foo`.

// TODO jak to dokładnie ma się po zdesugorawoaniu? 
`Some = a -> foo` ? 


Assignments are used to bind values to identifiers. For example:

```
foo = 5
```
This introduces an identifier `foo` into the containing scope.

If `foo` was already introduced by a parent scope, it will be shadowed.

Example:
```
foo = 5
main =
    foo = 5 # this is a nested scope, shadowing occurs
```


If `foo` was already introduced by the current scope, error will be raised. 

Example:

```
foo = 5
foo = 5 # error, symbol defined twice in the same scope
```



## Contexts
There are two kinds of context: pattern context and non-pattern context. 

Each position in code is either in a pattern context or not. By default code is
in non-pattern context. Pattern context is introduced locally by certain
language constructs:  `=`, `:` and `->` operators (i.e. definition bindings, signature type ascription and lambdas).

Inside a pattern context each usage of a variable name (identifier starting with a
lower case letter) actually binds this identifier with whatever it is being
matched to. The bound identifier is visible and usable within the target scope.

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

OPEN QUESTION: actually it might be "nice" to have both `a` refer to the same
identifier here.

---

Overloading.

```
# root scope

foo this:a = …

foo this:a = …
```

In this case the `a` for each `foo` definition will be inferred by the compiler.
If the `a` ends up being different for them, they are valid overloads.
Otherwise, it is an error of having multiple definitions for the same name.


---


```
a -> b = c
```

If such line occurs on the top-level, `a` and `b` are introduced into the
definition scope. Otherwisee, they are introduced into the parent scope.

Does this introduce the `a` into the module's scope? 

(rules say "only if inline `=` does not introduce a new scope <=> on the top level)

Nie moze być `->` po lewej.

---

```
a = Int
foo = 5:a
```

What if

```
a = Int
foo = Int : a
```









TODO
W top levelu jaki jest dokładnie obszar scope'u definicji?
Czy obejmuje sygnaturę?

Czy może być wiele sygnatur do definicji?
Jak dać sygnaturę do czegoś co nie ma żadnej nazwy lub ma wiele nazw?

Różnica między pattern-matchingiem a typowaniem?
Różnica między `a = 5` oraz `5:a`. Co jest wartością, co jest typem?
Sygnatura bez definicji?