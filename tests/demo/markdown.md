# Markdown

- [Markdown](#markdown)
  - [Lorem ipsum](#lorem-ipsum)
  - [Haskell](#haskell)
  - [Python](#python)
  - [List of lists of lists](#list-of-lists-of-lists)
    - [General reference](#general-reference)
    - [Society and social sciences](#society-and-social-sciences)
  - [Grapefruit](#grapefruit)
  - [MyST-Parser](#myst-parser)
    - [Admonitions](#admonitions)
    - [Tables](#tables)
  - [Cross-references](#cross-references)

## Lorem ipsum

**Lorem ipsum** (/ˌlɔː.rəm ˈɪp.səm/ LOR-əm IP-səm) is a _dummy or placeholder_ text
commonly used in graphic design, publishing, and `web development`. Its purpose is to
permit a page layout to be designed, independently of the [copy] that will subsequently
populate it, or to demonstrate various fonts of a typeface ~~without meaningful text~~
that could be distracting.

Versions of the Lorem ipsum text have been used in typesetting since the 1960s, when
advertisements for Letraset transfer sheets popularized it. [^1]

[copy]: https://en.wikipedia.org/wiki/Copy_(publishing)

[^1]:
    Cibois, Philippe (2012-06-03). "Lorem ipsum: nouvel état de la question".
    L'intelligence du monde. L'Institut français. Retrieved 2017-04-07.

## Haskell

```hs
module Main (main) where          -- not needed in interpreter, is the default in a module file

main :: IO ()                     -- the compiler can infer this type definition
main = putStrLn "Hello, World!"
```

## Python

```py
n = int(input('Type a number, and its factorial will be printed: '))

if n < 0:
    raise ValueError('You must enter a non-negative integer')

factorial = 1
for i in range(2, n + 1):
    factorial *= i

print(factorial)
```

## List of lists of lists

This list of lists of lists is a list of articles that are lists of other list articles.
Each of the pages linked here is an index to multiple lists on a topic.

### General reference

- List of lists of lists – this article itself is a list of lists, so it contains itself
- Lists of academic journals
- Lists of encyclopedias
- Lists of important publications in science
- Lists of problems
  - Lists of unsolved problems

### Society and social sciences

1. Lists of abbreviations
2. Lists of dictionaries
3. Lists of English words

   1. Lists of collective nouns
   2. Lists of English words by country or language of origin

      1. Lists of English words of Celtic origin
      2. Lists of English words of Scottish origin

   3. Lists of Merriam-Webster's Words of the Year
   4. Lists of pejorative terms for people
   5. Lists of words having different meanings in American and British English
   6. Word lists by frequency

## Grapefruit

```{figure} grapefruit-slice.jpg
:scale: 50 %
:alt: grapefruit slice

Source: <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/img>
```

## MyST-Parser

### Admonitions

```{tip}
Let's give readers a helpful hint!
```

```{versionadded} 1.2.3
Explanation of the new feature.
```

```{versionchanged} 1.2.3
Explanation of the change.
```

```{deprecated} 1.2.3
Explanation of the deprecation.
```

### Tables

| foo | bar |
| --- | --- |
| baz | bim |

| left | center | right |
| :--- | :----: | ----: |
| a    |   b    |     c |

```{table} Table caption
:widths: auto
:align: center

| foo | bar |
| --- | --- |
| baz | bim |
```

```{list-table} Frozen Delights!
:widths: 15 10 30
:header-rows: 1

*   - Treat
    - Quantity
    - Description
*   - Albatross
    - 2.99
    - On a stick!
*   - Crunchy Frog
    - 1.49
    - If we took the bones out, it wouldn't be
 crunchy, now would it?
*   - Gannet Ripple
    - 1.99
    - On a stick!
```

## Cross-references

[Poem](./restructuredtext.rst#poem)

[Colors](./notebook.ipynb#colors-and-stderr)
