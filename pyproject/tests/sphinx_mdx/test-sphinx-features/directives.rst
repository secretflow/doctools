Directives
==========

.. versionadded:: 2.5
   The *spam* parameter.

.. deprecated:: 3.1
   Use :func:`spam` instead.

.. seealso::

   Module :py:mod:`zipfile`
      Documentation of the :py:mod:`zipfile` standard module.

   `GNU tar manual, Basic Tar Format <http://link>`_
      Documentation for tar archive files, including GNU tar extensions.

.. seealso:: modules :py:mod:`zipfile`, :py:mod:`tarfile`

.. hlist::
   :columns: 3

   * A list of
   * short items
   * that should be
   * displayed
   * horizontally

.. highlight:: c

.. code-block::

   int main(void)
   {
       printf("Hello, world!\n");
       return 0;
   }

.. code-block::

  #include <stdio.h>
  int main() {
    int dividend, divisor, quotient, remainder;
    printf("Enter dividend: ");
    scanf("%d", &dividend);
    printf("Enter divisor: ");
    scanf("%d", &divisor);
    // Computes quotient
    quotient = dividend / divisor;
    // Computes remainder
    remainder = dividend % divisor;
    printf("Quotient = %d\n", quotient);
    printf("Remainder = %d", remainder);
    return 0;
  }

.. highlight:: python

.. literalinclude:: conf.py

.. glossary::

   environment
      A structure where information about all documents under the root is
      saved, and used for cross-referencing.  The environment is pickled
      after the parsing stage, so that successive runs only need to read
      and parse new and changed documents.

   source directory
      The directory which, including its subdirectories, contains all
      source files for one Sphinx project.
