# Rich content

## Math

Here is an equation: {math}`X_{0:5} = (X_0, X_1, X_2, X_3, X_4)`.

Here is another:

```{math}
:label: This is a label

\nabla^2 f =
\frac{1}{r^2} \frac{\partial}{\partial r}
\left( r^2 \frac{\partial f}{\partial r} \right) +
\frac{1}{r^2 \sin \theta} \frac{\partial f}{\partial \theta}
\left( \sin \theta \, \frac{\partial f}{\partial \theta} \right) +
\frac{1}{r^2 \sin^2\theta} \frac{\partial^2 f}{\partial \phi^2}
```

You can add a link to equations like the one above {eq}`This is a label` by using
`{eq}`.

Since Pythagoras, we know that {math}`a^2 + b^2 = c^2`.

```{math}
:label: mymath
(a + b)^2 = a^2 + 2ab + b^2

\begin{align}
(a + b)^2  &=  (a + b)(a + b) \\
           &=  a^2 + 2ab + b^2
\end{align}
```

The equation {eq}`mymath` is a quadratic equation.

```{warning}
[Implicitly aligned equations using `\\` and `&`][myst] are not supported. You must use
`\begin{align}` and `\end{align}` explicitly.

[myst]: https://myst-parser.readthedocs.io/en/latest/syntax/math.html#math-role-and-directive
```

## Graphviz

```{graphviz}
digraph mygraph {
  fontname="Helvetica Neue, sans-serif"
  node [fontname="Helvetica Neue, sans-serif"]
  edge [fontname="Helvetica Neue, sans-serif"]
  node [shape=box];
  "//absl/random:random"
  "//absl/random:random" -> "//absl/random:distributions"
  "//absl/random:random" -> "//absl/random:seed_sequences"
  "//absl/random:random" -> "//absl/random/internal:pool_urbg"
  "//absl/random:random" -> "//absl/random/internal:nonsecure_base"
  "//absl/random:distributions"
  "//absl/random:distributions" -> "//absl/strings:strings"
  "//absl/random:seed_sequences"
  "//absl/random:seed_sequences" -> "//absl/random/internal:seed_material"
  "//absl/random:seed_sequences" -> "//absl/random/internal:salted_seed_seq"
  "//absl/random:seed_sequences" -> "//absl/random/internal:pool_urbg"
  "//absl/random:seed_sequences" -> "//absl/random/internal:nonsecure_base"
  "//absl/random/internal:nonsecure_base"
  "//absl/random/internal:nonsecure_base" -> "//absl/random/internal:pool_urbg"
  "//absl/random/internal:nonsecure_base" -> "//absl/random/internal:salted_seed_seq"
  "//absl/random/internal:nonsecure_base" -> "//absl/random/internal:seed_material"
  "//absl/random/internal:pool_urbg"
  "//absl/random/internal:pool_urbg" -> "//absl/random/internal:seed_material"
  "//absl/random/internal:salted_seed_seq"
  "//absl/random/internal:salted_seed_seq" -> "//absl/random/internal:seed_material"
  "//absl/random/internal:seed_material"
  "//absl/random/internal:seed_material" -> "//absl/strings:strings"
  "//absl/strings:strings"
}
```

## Mermaid

```{mermaid}
sequenceDiagram
  participant Alice
  participant Bob
  Alice->John: Hello John, how are you?
  loop Healthcheck
      John->John: Fight against hypochondria
  end
  Note right of John: Rational thoughts <br/>prevail...
  John-->Alice: Great!
  John->Bob: How about you?
  Bob-->John: Jolly good!
```
