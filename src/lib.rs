#![no_std]


#![forbid(unsafe_code)]
// TODO: All the lints...


pub trait DeepSafeDrop: Sized
{
    /// Supply the first child and replace the link to it with a non-link, if
    /// the current state of `self` has at least one child.  (Many Self types
    /// should be able to compute this without needing any state other than
    /// their normal state.)
    fn take_first_child(&mut self) -> Option<Self>;

    /// Supply the first child and replace the link to it with a given
    /// replacement that links to the parent of `self`, if the current state of
    /// `self` has at least one child.  (Many Self types should be able to
    /// compute this without needing any state other than their normal state.)
    fn replace_first_child_with_parent(&mut self, parent: Self)
        -> ReplacedFirstChild<Self>;

    /// Supply the next child and replace the link to it with a non-link, if the
    /// current state of `self` has another child that has not been iterated
    /// yet.  (Many Self types should be able to compute this without needing
    /// any state other than their normal state.)
    fn take_next_child(&mut self) -> Option<Self>;
}


pub enum ReplacedFirstChild<T> {
    Yes { first_child: T },
    No { returned_parent: T }
}


/// A node's first-child link is reused as the parent link.
fn take_parent<T>(node: &mut T) -> Option<T>
where
    T: DeepSafeDrop
{
    let parent = node.take_first_child();
    debug_assert!(matches!(node.take_first_child(), None));
    parent
}


pub fn deep_safe_drop<T>(tree: T)
where
    T: DeepSafeDrop
{
    use ReplacedFirstChild::*;

    /// Return the nearest ancestor that has a next child if any, or the root
    /// ancestor even when it does not have a next child.  Drop any ancestors in
    /// the upwards path that do not have a child but that do have a parent.
    fn take_ancestor_next_child<T>(parent: T) -> (T, Option<T>)
    where
        T: DeepSafeDrop
    {
        let mut ancestor = parent;
        loop {
            if let Some(next_child) = ancestor.take_next_child() {
                break (ancestor, Some(next_child));
            }
            else if let Some(grandancestor) = take_parent(&mut ancestor) {
                // `ancestor` is now a leaf node so drop it here.
                drop(ancestor);
                ancestor = grandancestor;
            }
            else {
                break (ancestor, None);
            }
        }
    }

    let mut parent = tree;

    if let Some(mut cur) = parent.take_first_child()
    {
        debug_assert!(matches!(parent.take_first_child(), None));

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
                        // Done. `parent` is now `tree` which is now mutated to
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


#[cfg(test)]
mod tests {
    extern crate std;
    use std::prelude::v1::*;
    use super::*;

    /// This results in tree depths that are enough to cause stack overflows
    /// when `deep_safe_drop` is not used for a `Drop` impl.  You may increase
    /// this but more RAM will be required.
    const TREE_SIZE: usize = 1 << 20;

    mod basic {
        use super::*;

        mod list;
        mod binary_tree;
        mod dyn_trait;
    }
}
