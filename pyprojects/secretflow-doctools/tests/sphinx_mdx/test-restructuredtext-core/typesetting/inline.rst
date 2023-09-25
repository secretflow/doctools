Inline
======

The default role for interpreted text is `Title Reference`.  Here are
some explicit interpreted text roles: a PEP reference (:PEP:`287`); an
RFC reference (:RFC:`2822`); a :sub:`subscript`; a :sup:`superscript`;
and explicit roles for :emphasis:`standard` :strong:`inline`
:literal:`markup`.

.. DO NOT RE-WRAP THE FOLLOWING PARAGRAPH!

Let's test wrapping and whitespace significance in inline literals:
``This is an example of --inline-literal --text, --including some--
strangely--hyphenated-words.  Adjust-the-width-of-your-browser-window
to see how the text is wrapped.  -- ---- --------  Now note    the
spacing    between the    words of    this sentence    (words
should    be grouped    in pairs).``

If the ``--pep-references`` option was supplied, there should be a
live link to PEP 258 here.

reStructuredText Interpreted Text Roles
---------------------------------------

This is `interpreted text` using the default role.

This is :title:`interpreted text` using an explicit role.

An abbreviation used in the document. An example of an abbreviation is :abbr:`St.` being used instead of ‘Street’.

.. role:: latex(code)
   :language: latex

:latex:`A_\text{c} = (\pi/4) d^2`

:math:`A_\text{c} = (\pi/4) d^2`

.. role:: raw-html(raw)
   :format: html

If there just *has* to be a line break here,
:raw-html:`<br />`
it can be accomplished with a "raw"-derived role.
But the line block syntax should be considered first.
