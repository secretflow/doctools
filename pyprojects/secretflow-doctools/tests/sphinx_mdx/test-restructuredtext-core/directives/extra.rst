Topics, Sidebars, and Rubrics
-----------------------------

.. sidebar:: Optional Sidebar Title
   :subtitle: Optional Subtitle

   This is a sidebar.  It is for text outside the flow of the main
   text.

   .. rubric:: This is a rubric inside a sidebar

   Sidebars often appears beside the main text with a border and
   background color.

.. topic:: Topic Title

   This is a topic.

.. rubric:: This is a rubric

Replacement Text
----------------

I recommend you try |Python|_.

.. |Python| replace:: Python, *the* best language around

.. _Python: https://www.python.org

Target Footnotes
------------------------

.. target-notes::

Compound Paragraph
------------------

.. compound::

   This paragraph contains a literal block::

       Connecting... OK
       Transmitting data... OK
       Disconnecting... OK

   and thus consists of a simple paragraph, a literal block, and
   another simple paragraph.  Nonetheless it is semantically *one*
   paragraph.

This construct is called a *compound paragraph* and can be produced
with the "compound" directive.

Comments
--------

Here's one:

.. Comments begin with two dots and a space. Anything may
   follow, except for the syntax of footnotes, hyperlink
   targets, directives, or substitution definitions.

   Double-dashes -- "--" -- must be escaped somehow in HTML output.

(View the HTML source to see the comment.)

Error Handling
==============

Any errors caught during processing will generate system messages.

|*** Expect 6 errors (including this one). ***|

There should be six messages in the following, auto-generated
section, "Docutils System Messages":

.. section should be added by Docutils automatically
