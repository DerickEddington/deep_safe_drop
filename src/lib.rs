//! Safe dropping of deep trees that otherwise could cause stack overflow.
//!
//! Does not require any allocation and reuses existing space of a tree to
//! enable working back up a tree branch, instead of the call-stack.
//!
//! No `unsafe` code.
//!
//! Is `no_std` and so can be used in constrained environments (e.g. without
//! heap allocation).
//!
//! Provides:
//!
//! - [`deep_safe_drop`] function to be called from your [`Drop::drop`]
//!   implementations.
//!
//! - [`DeepSafeDrop`] trait to be implemented by your types that use
//!   `deep_safe_drop`.
//!
//! Stack overflow is avoided by mutating a tree to become a leaf, i.e. no
//! longer have any children, doing the same mutation to children recursively
//! but iteratively, mutating children to become leafs, dropping leaf nodes as
//! they're enountered, before the implicit compiler-added dropping does its
//! automatic recursive dropping of fields.  Instead of using recursive function
//! calls (i.e. recording the continuations on the limited call-stack) to enable
//! working back up a tree branch (as the compiler's dropping does, which is
//! what could otherwise cause stack overflows), we reuse a link of each node to
//! record which parent node must be worked back "up" to.  Thus, we are
//! guaranteed to already have the amount of space needed for our
//! "continuations", no matter how extremely deep it may need to be, and it is
//! OK to reuse this space because the links it previously contained are already
//! being dropped anyway.
//!
//! A simple example of the mutation steps (nodes are dropped when removed as
//! leafs):
//! ```text
//! Initial:   Step 1:    Step 2:    Step 3:
//!   a          a          a          a    
//!    ⭨          ⭦          ⭦              
//!     b          b          b             
//!    ⭩ ⭨        ⭧ ⭨          ⭨            
//!   c   d      c   d          d           
//!  ⭩ ⭨          ⭨                         
//! e   f          f                        
//! ```
//! Note: Initially, `a` links to `b` and `b` links to `c`, but, at Step 1 and
//! after, `c` links to `b` and `b` links to `a`.  This is the reuse of a node's
//! link space to save the parent for later traversing back "up" to it, which
//! enables transitioning to Steps 2 & 3.  All steps are transitioned to via a
//! loop in the same single function call, by moving cursors down and "up" the
//! tree.
//!
//! See the tests for some examples of incorporating for different types and
//! different shapes.


#![no_std]


#![forbid(unsafe_code)]

// Warn about desired lints that would otherwise be allowed by default.
#![warn(
    // Groups
    future_incompatible,
    nonstandard_style,
    rust_2018_compatibility, // unsure if needed with edition="2018"
    rust_2018_idioms,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::cargo,
    // Individual lints not included in above groups and desired.
    macro_use_extern_crate,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    // missing_doc_code_examples, // maybe someday
    rustdoc::private_doc_tests,
    single_use_lifetimes, // annoying hits on invisible derived impls
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences,
)]

// Exclude (re-allow) undesired lints included in above groups.
#![allow(
    clippy::non_ascii_literal,
    clippy::blanket_clippy_restriction_lints,
    clippy::else_if_without_else,
    clippy::implicit_return,
    clippy::missing_docs_in_private_items,
    clippy::shadow_reuse,
    clippy::default_numeric_fallback,
    clippy::mod_module_files,
    clippy::separated_literal_suffix,
)]


use core::borrow::BorrowMut;


/// Implement this for your tree node type, with `Link` as your tree link type
/// that references or is your node type.
///
/// The `Link` type may be the same as the `Self` type, when possible, which
/// might be convenient.  Or, they can be different.
///
/// Many node types should be able to implement these methods without needing
/// any extra state beyond their normal state, e.g. because their link fields
/// already support some unused state.
pub trait DeepSafeDrop<Link>
{
    /// Take the next child and replace the link to it with a non-link, if the
    /// current state of `self` has another child that has not been supplied
    /// yet.  This may return the child at index 0 when there is one.
    #[inline]
    fn take_next_child_at_any_index(&mut self) -> Option<Link>
    {
        self.take_child_at_index_0().or_else(|| self.take_next_child_at_pos_index())
    }

    /// Take the child at index 0 and replace the link to it with a given
    /// replacement that links to the parent of `self`.
    fn set_parent_at_index_0(&mut self, parent: Link) -> SetParent<Link>;

    /// Take the child at index 0 and replace the link to it with a non-link.
    fn take_child_at_index_0(&mut self) -> Option<Link>;

    /// Take the next child at an index greater than or equal to 1 and replace
    /// the link to it with a non-link, if the current state of `self` has
    /// another child at those indices that has not been supplied yet.  This
    /// must not return the child at index 0 when there is one, because that is
    /// reused to link to the parent.
    fn take_next_child_at_pos_index(&mut self) -> Option<Link>;
}

/// Result of [`DeepSafeDrop::set_parent_at_index_0`].
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum SetParent<Link> {
    /// There was a child at index 0 and it was replaced by the parent.
    YesReplacedChild {
        /// The child at index 0 that was taken.
        child0: Link
    },
    /// The parent was set at index 0 and no child was replaced.
    Yes,
    /// No setting could be done, because the node has no links, so the parent
    /// must be returned back.
    No {
        /// The same `parent` value that was given to the method call.
        returned_parent: Link
     },
}


/// Exists to do these `debug_assert`s when a node can be immediately dropped
/// because it's a leaf.
fn drop_leaf<L, N>(mut link: L)
where
    L: BorrowMut<N>,
    N: DeepSafeDrop<L> + ?Sized,
{
    let node = link.borrow_mut();
    debug_assert!(node.take_next_child_at_any_index().is_none());
    debug_assert!(node.take_child_at_index_0().is_none());
    debug_assert!(node.take_next_child_at_pos_index().is_none());
    drop(link);
}


/// A node's link at index 0 is reused as the parent link.
fn take_parent<L, N>(node: &mut N) -> Option<L>
where
    N: DeepSafeDrop<L> + ?Sized,
{
    let child0 = node.take_child_at_index_0();
    debug_assert!(node.take_child_at_index_0().is_none());
    child0
}

/// Return the nearest ancestor that has a next child if any, or the root
/// ancestor even when it does not have a next child.  Drop any ancestors in the
/// upwards path that do not have a child but that do have a parent.
fn take_ancestor_next_child<L, N>(parent: L) -> (L, Option<L>)
where
    L: BorrowMut<N>,
    N: DeepSafeDrop<L> + ?Sized,
{
    let mut ancestor = parent;
    loop {
        if let Some(next_child) = ancestor.borrow_mut().take_next_child_at_pos_index() {
            break (ancestor, Some(next_child));
        }
        else if let Some(grandancestor) = take_parent(ancestor.borrow_mut()) {
            drop_leaf(ancestor);  // `ancestor` is now a leaf node so drop it here.
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
    L: BorrowMut<N>,
    N: DeepSafeDrop<L> + ?Sized,
{
    let mut parent = top;

    if let Some(mut cur) = parent.borrow_mut().take_next_child_at_any_index() {
        loop {
            match cur.borrow_mut().set_parent_at_index_0(parent)
            {
                SetParent::YesReplacedChild { child0 } => {
                    parent = cur;
                    cur = child0;
                    continue;
                }
                SetParent::Yes => {
                    if let Some(child) = cur.borrow_mut().take_next_child_at_pos_index() {
                        parent = cur;
                        cur = child;
                        continue;
                    }
                    else {
                        parent = cur;
                    }
                }
                SetParent::No { returned_parent } => {
                    parent = returned_parent;
                    drop_leaf(cur);  // `cur` is now a leaf node so drop it here.
                }
            }

            let (ancestor, ancestor_child) = take_ancestor_next_child(parent);
            parent = ancestor;

            if let Some(ancestor_child) = ancestor_child {
                cur = ancestor_child;
            }
            else {
                // Done. `parent` is now `top` which is now mutated to no longer
                // have any children, so, when dropping it is completed, by the
                // implicit compiler-added code, after this function returns,
                // recursion into children cannot occur and so stack overflow
                // cannot occur.
                drop_leaf(parent);
                break;
            }
        }
    }
}

/// To be called from your [`Drop::drop`] implementations, to ensure that stack
/// overflow is avoided.
///
/// The `RootNode` type may be different than the primary `Node` type, when
/// possible, which might be convenient.  Or, they can be the same.
#[inline]
pub fn deep_safe_drop<RootNode, Link, Node>(root: &mut RootNode)
where
    RootNode: DeepSafeDrop<Link> + ?Sized,
    Link: BorrowMut<Node>,
    Node: DeepSafeDrop<Link> + ?Sized,
{
        while let Some(next_child) = root.take_next_child_at_any_index() {
            main_deep_safe_drop(next_child);
        }
}


#[cfg(test)]
mod tests;
