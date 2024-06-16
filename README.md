# `deep_safe_drop`

Safe dropping of deep trees that otherwise could cause stack overflow.

Does not require any allocation and reuses existing space of a tree to enable working back up a
tree branch, instead of the call-stack.

No `unsafe` code.

Is `no_std` and so can be used in constrained environments (e.g. without heap allocation).

Provides:

- [`deep_safe_drop`] function to be called from your [`Drop::drop`] implementations.

- [`DeepSafeDrop`] trait to be implemented by your types that use `deep_safe_drop`.

Stack overflow is avoided by mutating a tree to become a leaf, i.e. no longer have any children,
doing the same mutation to children recursively but iteratively, dropping leaf nodes as they're
encountered, mutating children to become leafs, before the implicit compiler-added dropping does
its automatic final dropping of fields via recursive calls, so that all nodes, including the root,
have become a leaf by the time it does for each, and thereby recursive calls to that final
dropping are not done.

Instead of using recursive function calls (i.e. recording the continuations on the limited
call-stack) to enable working back up a tree branch to traverse to other branches (as the
compiler-added final dropping does, which is what could otherwise cause stack overflows), we reuse
a link of each node to record which parent node must be worked back "up" to.  Thus, we are
guaranteed to already have the amount of space needed for our "continuations", no matter how
extremely deep it may need to be, and it is OK to reuse this space because the links it previously
contained are already being dropped anyway.

A simple example of the mutation steps (nodes are dropped when removed as leafs):
```text
Initial:   Step 1:    Step 2:    Step 3:
  a          a          a          a    
   ⭨          ⭦          ⭦              
    b           b          b             
   ⭩ ⭨        ⭧ ⭨          ⭨            
  c   d       c   d          d           
 ⭩ ⭨          ⭨                         
e   f          f                        
```
Note: Initially, `a` links to `b` and `b` links to `c`, but, at Step 1 and after, `c` links to `b`
and `b` links to `a`.  This is the reuse of a node's link space to save the parent for later
traversing back "up" to it, which enables transitioning to Steps 2 & 3.  All steps are
transitioned to via a loop in the same single function call, by moving cursors down and "up" a
tree.

See the tests for some examples of incorporating for different types and different shapes.
