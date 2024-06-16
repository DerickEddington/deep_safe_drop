#![cfg_attr(not(windows), doc = include_str!("../README.md"))]
#![cfg_attr(windows, doc = include_str!("..\\README.md"))]
#![no_std]


/// Implement this for your tree node type, with `Link` as your tree link type that references or
/// is your node type.
///
/// The `Link` type may be the same as the `Self` type, when possible, which might be convenient.
/// Or, they can be different.
///
/// Many node types should be able to implement these methods without needing any extra state
/// beyond their normal state, e.g. because their link fields already support some unused state.
pub trait DeepSafeDrop<Link>
{
    /// Take the next child and replace the link to it with a non-link, if the current state of
    /// `self` has another child that has not been supplied yet.  This may return the child at
    /// index 0 when there is one.
    #[inline]
    fn take_next_child_at_any_index(&mut self) -> Option<Link>
    {
        self.take_child_at_index_0().or_else(|| self.take_next_child_at_pos_index())
    }

    /// Take the child at index 0 and replace the link to it with a given replacement that links
    /// to the parent of `self`.
    fn set_parent_at_index_0(
        &mut self,
        parent: Link,
    ) -> SetParent<Link>;

    /// Take the child at index 0 and replace the link to it with a non-link.
    fn take_child_at_index_0(&mut self) -> Option<Link>;

    /// Take the next child at an index greater than or equal to 1 and replace the link to it with
    /// a non-link, if the current state of `self` has another child at those indices that has not
    /// been supplied yet.  This must not return the child at index 0 when there is one, because
    /// that is reused to link to the parent.
    fn take_next_child_at_pos_index(&mut self) -> Option<Link>;
}


/// Result of [`DeepSafeDrop::set_parent_at_index_0`].
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum SetParent<Link>
{
    /// There was a child at index 0 and it was replaced by the parent.
    YesReplacedChild
    {
        /// The child at index 0 that was taken.
        child0: Link,
    },

    /// The parent was set at index 0 and no child was replaced.
    Yes,

    /// No setting could be done, because the node has no links, so the parent must be returned
    /// back.
    No
    {
        /// The same `parent` value that was given to the method call.
        returned_parent: Link,
    },
}

/// Implement this for your tree link type, with `Node` as your tree node type.
///
/// The `Node` type may be the same as the `Self` type, when possible, which might be convenient.
/// Or, they can be different.
pub trait Link<Node: ?Sized>
{
    /// Return a mutable reference to the node that `self` links to.
    fn get_mut(&mut self) -> &mut Node;
}


/// Exists to do these `debug_assert`s when a node can be immediately dropped because it's a leaf.
fn drop_leaf<L, N>(mut link: L)
where
    L: Link<N>,
    N: DeepSafeDrop<L> + ?Sized,
{
    let node = link.get_mut();
    debug_assert!(node.take_next_child_at_any_index().is_none(), "must be leaf");
    debug_assert!(node.take_child_at_index_0().is_none(), "must be leaf");
    debug_assert!(node.take_next_child_at_pos_index().is_none(), "must be leaf");
    drop(link);
}


/// A node's link at index 0 is reused as the parent link.
fn take_parent<L, N>(node: &mut N) -> Option<L>
where N: DeepSafeDrop<L> + ?Sized
{
    let child0 = node.take_child_at_index_0();
    debug_assert!(node.take_child_at_index_0().is_none(), "must be gone after take");
    child0
}


/// Return the nearest ancestor that has a next child if any, or the root ancestor even when it
/// does not have a next child.  Drop any ancestors in the upwards path that do not have a child
/// but that do have a parent.
fn take_ancestor_next_child<L, N>(parent: L) -> (L, Option<L>)
where
    L: Link<N>,
    N: DeepSafeDrop<L> + ?Sized,
{
    let mut ancestor = parent;
    loop {
        if let Some(next_child) = ancestor.get_mut().take_next_child_at_pos_index() {
            break (ancestor, Some(next_child));
        }
        else if let Some(grandancestor) = take_parent(ancestor.get_mut()) {
            drop_leaf(ancestor); // `ancestor` is now a leaf node so drop it here.
            ancestor = grandancestor;
        }
        else {
            break (ancestor, None);
        }
    }
}


/// The main algorithm.
fn main_deep_safe_drop<L, N>(top: L)
where
    L: Link<N>,
    N: DeepSafeDrop<L> + ?Sized,
{
    let mut parent = top;

    if let Some(mut cur) = parent.get_mut().take_next_child_at_any_index() {
        loop {
            match cur.get_mut().set_parent_at_index_0(parent) {
                SetParent::YesReplacedChild { child0 } => {
                    parent = cur;
                    cur = child0;
                    continue;
                },
                SetParent::Yes => {
                    let next = cur.get_mut().take_next_child_at_pos_index();
                    parent = cur;
                    if let Some(child) = next {
                        cur = child;
                        continue;
                    }
                },
                SetParent::No { returned_parent } => {
                    parent = returned_parent;
                    drop_leaf(cur); // `cur` is now a leaf node so drop it here.
                },
            }

            let (ancestor, ancestor_child) = take_ancestor_next_child(parent);
            parent = ancestor;

            if let Some(ancestor_child) = ancestor_child {
                cur = ancestor_child;
            }
            else {
                // Done. `parent` is now `top` which is now mutated to no longer have any
                // children, so, when dropping it is completed, by the implicit compiler-added
                // code, after this function returns, recursion into children cannot occur and so
                // stack overflow cannot occur.
                drop_leaf(parent);
                break;
            }
        }
    }
}


/// To be called from your [`Drop::drop`] implementations, to ensure that stack overflow is
/// avoided.
///
/// The `RootNode` type may be different than the primary `Node` type, when possible, which might
/// be convenient.  Or, they can be the same.
#[inline]
pub fn deep_safe_drop<RootNode, Link, Node>(root: &mut RootNode)
where
    RootNode: DeepSafeDrop<Link> + ?Sized,
    Link: crate::Link<Node>,
    Node: DeepSafeDrop<Link> + ?Sized,
{
    while let Some(next_child) = root.take_next_child_at_any_index() {
        main_deep_safe_drop(next_child);
    }
}
