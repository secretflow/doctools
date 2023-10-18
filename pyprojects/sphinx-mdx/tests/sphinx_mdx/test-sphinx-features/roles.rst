.. default-role:: any

Signal Handling
===============

.. module:: signal

.. function:: install()

   This function installs a `handler` for every signal known by the
   `signal` module.  See the section `list-of-signals` for more information.

.. function:: handler(signal, frame)

   This function is called when a signal is received.  The ``signal`` argument
   is the signal number, and the ``frame`` argument is the current stack frame
   (or `None` if the signal is not associated with a stack frame).

   The default handler prints a message to the standard error stream and
   raises a :exc:`SystemExit` exception with the signal number as the exit
   code.  See the section `list-of-signals` for more information.

.. _list-of-signals:

List of signals
---------------

The `signal` module defines a number of signals, which are listed in the
following table.  The table also shows the default action for each signal.

+--------------------+-------------------+------------------------------------+
| Signal             | Default Action    | Description                        |
+====================+===================+====================================+
| ``signal.SIGABRT`` | Abnormal termination of the program, such as a call to |
|                    | ``abort()``.                                           |
+--------------------+-------------------+------------------------------------+
| ``signal.SIGALRM`` | Termination of the program on alarm clock timeout.     |
+--------------------+-------------------+------------------------------------+
| ``signal.SIGBUS``  | Abnormal termination of the program, such as a         |
|                    | ``SIGBUS``.                                            |
+--------------------+-------------------+------------------------------------+

:doc:`./cpp`

See :download:`this example script <./conf.py>`.

.. role:: python(code)
   :language: python

In Python, :python:`1 + 2` is equal to :python:`3`.

:command:`rm -rf /`

:kbd:`Control-x Control-f`

... is installed in :file:`/usr/lib/python3.{x}/site-packages` ...

:menuselection:`Start --> Programs`
