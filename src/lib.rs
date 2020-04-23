#![no_std]


#![forbid(unsafe_code)]
// TODO: All the lints...


use core::ops::DerefMut;


pub trait DeepSafeDrop<Link>
{
    /// Supply the first child and replace the link to it with a non-link, if
    /// the current state of `self` has at least one child.  (Many Self types
    /// should be able to compute this without needing any state other than
    /// their normal state.)
    fn take_first_child(&mut self) -> Option<Link>;

    /// Supply the first child and replace the link to it with a given
    /// replacement that links to the parent of `self`, if the current state of
    /// `self` has at least one child.  (Many Self types should be able to
    /// compute this without needing any state other than their normal state.)
    fn replace_first_child_with_parent(&mut self, parent: Link)
        -> ReplacedFirstChild<Link>;

    /// Supply the next child and replace the link to it with a non-link, if the
    /// current state of `self` has another child that has not been iterated
    /// yet.  (Many Self types should be able to compute this without needing
    /// any state other than their normal state.)
    fn take_next_child(&mut self) -> Option<Link>;
}

pub enum ReplacedFirstChild<L> {
    Yes { first_child: L },
    No { returned_parent: L },
}


/// Exists only to do the `debug_assert`.
fn take_first_child<T, L>(thing: &mut T) -> Option<L>
where
    T: DeepSafeDrop<L> + ?Sized,
{
    let first_child = thing.take_first_child();
    debug_assert!(matches!(thing.take_first_child(), None));
    first_child
}

/// A node's first-child link is reused as the parent link.
fn take_parent<L, N>(node: &mut N) -> Option<L>
where
    N: DeepSafeDrop<L> + ?Sized,
{
    let parent = take_first_child(node);
    parent
}

/// Return the nearest ancestor that has a next child if any, or the root
/// ancestor even when it does not have a next child.  Drop any ancestors in the
/// upwards path that do not have a child but that do have a parent.
fn take_ancestor_next_child<L>(parent: L) -> (L, Option<L>)
where
    L: DerefMut,
    L::Target: DeepSafeDrop<L>,
{
    let mut ancestor = parent;
    loop {
        if let Some(next_child) = ancestor.take_next_child() {
            break (ancestor, Some(next_child));
        }
        else if let Some(grandancestor) = take_parent(&mut *ancestor) {
            // `ancestor` is now a leaf node so drop it here.
            drop(ancestor);
            ancestor = grandancestor;
        }
        else {
            break (ancestor, None);
        }
    }
}


/// The main algorithm.
fn main_deep_safe_drop<L>(top: L)
where
    L: DerefMut,
    L::Target: DeepSafeDrop<L>,
{
    use ReplacedFirstChild::*;

    let mut parent = top;

    if let Some(mut cur) = take_first_child(&mut *parent) {
        loop {
            match cur.replace_first_child_with_parent(parent)
            {
                Yes { first_child } => {
                    parent = cur;
                    cur = first_child;
                }
                No { returned_parent } => {
                    parent = returned_parent;

                    // `cur` is a leaf node so drop it here.
                    drop(cur);

                    let (ancestor, ancestor_child) = take_ancestor_next_child(parent);
                    parent = ancestor;

                    if let Some(ancestor_child) = ancestor_child {
                        cur = ancestor_child;
                    }
                    else {
                        // Done. `parent` is now `top` which is now mutated to
                        // no longer have any children, so, when dropping it is
                        // completed by the compiler after this function
                        // returns, recursion into children cannot occur and so
                        // stack overflow cannot occur.
                        drop(parent);
                        break;
                    }
                }
            }
        }
    }
}

#[inline]
pub fn deep_safe_drop<T, L>(root: &mut T)
where
    T: DeepSafeDrop<L> + ?Sized,
    L: DerefMut,
    L::Target: DeepSafeDrop<L>,
{
    if let Some(child) = take_first_child(root) {
        main_deep_safe_drop(child);

        while let Some(child) = root.take_next_child() {
            main_deep_safe_drop(child);
        }
    }
}


#[cfg(test)]
mod tests;
