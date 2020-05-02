# deep_safe_drop

Safe dropping of deep trees that otherwise could cause stack overflow.

Does not require any allocation and reuses existing space of a tree to
enable working back up a tree branch, instead of the stack.

Is `no_std` and so can be used in constrained environments (e.g. without
heap allocation).

Provides:

- `deep_safe_drop` function to be called from your `Drop::drop`
  implementations.

- `DeepSafeDrop` trait to be implemented by your types that use
  `deep_safe_drop`.

Stack overflow is avoided by mutating a tree to become a leaf, i.e. no
longer have any children, before the compiler does its automatic recursive
dropping of fields.  Instead of using recursive function calls
(i.e. recording the continuations on the limited stack) to enable working
back up a tree branch (as the compiler's dropping does, which is what could
otherwise cause stack overflows), we reuse a link of each node to record
which parent node must be worked back up to.  Thus, we are guaranteed to
already have the amount of space needed for our "continuations", no matter
how extremely deep it may need to be, and it is OK to reuse this space
because the links it previously contained are already being dropped anyway.

See the included tests for some examples.

License: Unlicense
