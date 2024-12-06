# ScLang (i forgot why I called it this)

## Syntax

very similar to stlc, minor changes

### atomics

set of all atomics is `a`

* `n`: natural numbers := {0, 1, 2, 3, ...}
* `b`: boolean := {true, false}
* `x`: identifier := string, cannot start with an number, may contain underscores
* `t`: types ::= {`num`, `bool`, `t->t`, `{(x : t)*}`}


### Expressions

set of all expressions is `e`

every `a` here should be an `e`, but im currently not smart enough to fix that.

* declaration: `let x = e; e`
* function: `fun(x : t) { e }`
* addition: `a + a`
* function application: `a(a)`
* record: `{(x = e,)*}`
* record access: `a.x`
* extension: `a extends x`
* with: `with a do a`

```

x -> id
b -> true | false
n -> num {0, 1, 2, ..}

a -> x | b | n

e -> term e'

term -> let x = e; e
    | fun(x : t) { e }
    | {(x = e,)*}
    | with e do e
    | a


e' -> (e) e'
    | + e' e'
    | .x e'
    | extends e e'
    | eof




```


#### parse tower

1. extension
2. function application
3. record access
4.