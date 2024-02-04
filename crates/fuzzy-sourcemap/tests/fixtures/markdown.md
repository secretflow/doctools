<!--

Adapted from https://spec.commonmark.org/0.31.2/. John MacFarlane. CC BY-SA 4.0. Source at https://github.com/commonmark/commonmark-spec/blob/9103e341a973013013bb1a80e13567007c5cef6f/spec.txt

Adapted from https://github.com/rust-lang/book. The Rust Project Developers. MIT or Apache-2.0.

-->

# [CommonMark]

[CommonMark]: https://commonmark.org

Markdown is a plain text format for writing structured documents,
based on conventions for indicating formatting in email
and usenet posts. It was developed by John Gruber (with
help from Aaron Swartz) and released in 2004 in the form of a
[syntax description](https://daringfireball.net/projects/markdown/syntax)
and a Perl script (`Markdown.pl`) for converting Markdown to
HTML. In the next decade, dozens of implementations were
developed in many languages. Some extended the original
Markdown syntax with conventions for footnotes, tables, and
other document elements. Some allowed Markdown documents to be
rendered in formats other than HTML. Websites like Reddit,
StackOverflow, and GitHub had millions of people using Markdown.
And Markdown started to be used beyond the web, to author books,
articles, slide shows, letters, and lecture notes.

What distinguishes Markdown from many other lightweight markup
syntaxes, which are often easier to write, is its readability.
As Gruber writes:

> The overriding design goal for Markdown's formatting syntax is
> to make it as readable as possible. The idea is that a
> Markdown-formatted document should be publishable as-is, as
> plain text, without looking like it's been marked up with tags
> or formatting instructions.
> (<https://daringfireball.net/projects/markdown/>)

## 4.4 Indented code blocks

An indented code block is composed of one or more
[indented chunks] separated by blank lines.
An indented chunk is a sequence of non-blank lines,
each preceded by four or more spaces of indentation. The contents of the code
block are the literal contents of the lines, including trailing
[line endings], minus four spaces of indentation.
An indented code block has no [info string].

An indented code block cannot interrupt a paragraph, so there must be
a blank line between a paragraph and a following indented code block.
(A blank line is not needed, however, between a code block and a following
paragraph.)

    a simple
      indented code block

If there is any ambiguity between an interpretation of indentation
as a code block and as indicating that material belongs to a [list
item][list items], the list item interpretation takes precedence:

- foo

  bar

1. foo

   - bar

The contents of a code block are literal text, and do not get parsed
as Markdown:

    <a/>
    *hi*

    - one

Here we have three chunks separated by blank lines:

    chunk1

    chunk2



    chunk3

Any initial spaces or tabs beyond four spaces of indentation will be included in
the content, even in interior blank lines:

    chunk1

      chunk2

An indented code block cannot interrupt a paragraph. (This
allows hanging indents and the like.)

<!-- prettier-ignore-start -->

Foo
    bar

<!-- prettier-ignore-end -->

However, any non-blank line with fewer than four spaces of indentation ends
the code block immediately. So a paragraph may occur immediately
after indented code:

<!-- prettier-ignore-start -->

    foo
bar

<!-- prettier-ignore-end -->

And indented code can occur immediately before and after other kinds of blocks:

<!-- prettier-ignore-start -->

#### Heading
    foo
Heading
-------
    foo
----

<!-- prettier-ignore-end -->

## 4.5 Fenced code blocks

A code fence is a sequence
of at least three consecutive backtick characters (`` ` ``) or
tildes (`~`). (Tildes and backticks cannot be mixed.)
A fenced code block
begins with a code fence, preceded by up to three spaces of indentation.

```rust
fn main() {
    println!("Hello, world!");
}
```

### What is Ownership?

The variable `s` refers to a string literal, where the value of the string is
hardcoded into the text of our program. The variable is valid from the point at
which it’s declared until the end of the current _scope_. Listing 4-1 shows a
program with comments annotating where the variable `s` would be valid.

```rust
    {                      // s is not valid here, it’s not yet declared
        let s = "hello";   // s is valid from this point forward

        // do stuff with s
    }                      // this scope is now over, and s is no longer valid
```

The double colon `::` operator allows us to namespace this particular `from`
function under the `String` type rather than using some sort of name like
`string_from`. We’ll discuss this syntax more in the [“Method
Syntax”][method-syntax]<!-- ignore --> section of Chapter 5, and when we talk
about namespacing with modules in [“Paths for Referring to an Item in the
Module Tree”][paths-module-tree]<!-- ignore --> in Chapter 7.

This kind of string _can_ be mutated:

```rust
    let mut s = String::from("hello");

    s.push_str(", world!"); // push_str() appends a literal to a String

    println!("{}", s); // This will print `hello, world!`
```

### The `match` Control Flow Construct

Speaking of coins, let’s use them as an example using `match`! We can write a
function that takes an unknown US coin and, in a similar way as the counting
machine, determines which coin it is and returns its value in cents, as shown
in Listing 6-3.

```rust
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}
```

## 4.6 HTML blocks

HTML blocks continue until they are closed by their appropriate
[end condition], or the last line of the document or other [container
block](#container-blocks). This means any HTML **within an HTML
block** that might otherwise be recognised as a start condition will
be ignored by the parser and passed through as-is, without changing
the parser's state.

For instance, `<pre>` within an HTML block started by `<table>` will not affect
the parser state; as the HTML block was started in by start condition 6, it
will end at any blank line. This can be surprising:

<!-- prettier-ignore-start -->

<table><tr><td>
<pre>
**Hello**,

_world_.
</pre>
</td></tr></table>

<!-- prettier-ignore-end -->

## 5.1 Block quotes

The following rules define [block quotes]:

1.  **Basic case.** If a string of lines _Ls_ constitute a sequence
    of blocks _Bs_, then the result of prepending a [block quote
    marker] to the beginning of each line in _Ls_
    is a [block quote](#block-quotes) containing _Bs_.

2.  **Laziness.** If a string of lines _Ls_ constitute a [block
    quote](#block-quotes) with contents _Bs_, then the result of deleting
    the initial [block quote marker] from one or
    more lines in which the next character other than a space or tab after the
    [block quote marker] is [paragraph continuation
    text] is a block quote with _Bs_ as its content.
    Paragraph continuation text is text
    that will be parsed as part of the content of a paragraph, but does
    not occur at the beginning of the paragraph.

3.  **Consecutiveness.** A document cannot contain two [block
    quotes] in a row unless there is a [blank line] between them.

<!-- prettier-ignore-start -->

> ### Foo
> bar
> baz

The space or tab after the `>` characters can be omitted:

>### Bar
>bar
> baz

The `>` characters can be preceded by up to three spaces of indentation:

   > ### Baz
   > bar
 > baz

The Laziness clause allows us to omit the `>` before
[paragraph continuation text]:

> ### Quux
> bar
baz

A block quote can contain some lazy and some non-lazy
continuation lines:

> bar
baz
> foo

<!-- prettier-ignore-end -->

## 5.2 List items

A list marker is a
[bullet list marker] or an [ordered list marker].

A bullet list marker
is a `-`, `+`, or `*` character.

An ordered list marker
is a sequence of 1--9 arabic digits (`0-9`), followed by either a
`.` character or a `)` character. (The reason for the length
limit is that with 10 digits we start seeing integer overflows
in some browsers.)

The following rules define [list items]:

1.  **Basic case.** If a sequence of lines _Ls_ constitute a sequence of
    blocks _Bs_ starting with a character other than a space or tab, and _M_ is
    a list marker of width _W_ followed by 1 ≤ _N_ ≤ 4 spaces of indentation,
    then the result of prepending _M_ and the following spaces to the first line
    of _Ls_, and indenting subsequent lines of _Ls_ by _W + N_ spaces, is a
    list item with _Bs_ as its contents. The type of the list item
    (bullet or ordered) is determined by the type of its list marker.
    If the list item is ordered, then it is also assigned a start
    number, based on the ordered list marker.

    Exceptions:

    1. When the first list item in a [list] interrupts
       a paragraph---that is, when it starts on a line that would
       otherwise count as [paragraph continuation text]---then (a)
       the lines _Ls_ must not begin with a blank line, and (b) if
       the list item is ordered, the start number must be 1.
    2. If any line is a [thematic break][thematic breaks] then
       that line is not a list item.

<!-- prettier-ignore-start -->

1.  A paragraph
    with two lines.

        indented code

    > A block quote.

   > > 1.  one
>>
>>     two

A list item may contain blocks that are separated by more than
one blank line.

- foo


  bar

A list item may contain any kind of block:

1.  foo

    ```
    bar
    ```

    baz

    > bam

A list item that contains an indented code block will preserve
empty lines within the code block verbatim.

- Foo

      bar


      baz

<!-- prettier-ignore-end -->

3.  **Item starting with a blank line.** If a sequence of lines _Ls_
    starting with a single [blank line] constitute a (possibly empty)
    sequence of blocks _Bs_, and _M_ is a list marker of width _W_,
    then the result of prepending _M_ to the first line of _Ls_, and
    preceding subsequent lines of _Ls_ by _W + 1_ spaces of indentation, is a
    list item with _Bs_ as its contents.
    If a line is empty, then it need not be indented. The type of the
    list item (bullet or ordered) is determined by the type of its list
    marker. If the list item is ordered, then it is also assigned a
    start number, based on the ordered list marker.

<!-- prettier-ignore-start -->

-
  foo
-
  ```
  bar
  ```
-
      baz

<!-- prettier-ignore-end -->

4.  **Indentation.** If a sequence of lines _Ls_ constitutes a list item
    according to rule #1, #2, or #3, then the result of preceding each line
    of _Ls_ by up to three spaces of indentation (the same for each line) also
    constitutes a list item with the same contents and attributes. If a line is
    empty, then it need not be indented.

<!-- prettier-ignore-start -->

 1.  A paragraph
     with two lines.

         indented code

     > A block quote.

  1.  A paragraph
      with two lines.

          indented code

      > A block quote.

<!-- prettier-ignore-end -->
