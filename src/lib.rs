//! Safe dropping of deep trees that otherwise could cause stack overflow.
//!
//! Does not require any allocation and reuses existing space of a tree to
//! enable working back up a tree branch, instead of the stack.
//!
//! Is `no_std` and so can be used in constrained environments (e.g. without
//! heap allocation).
//!
//! Provides:
//!
//! - `deep_safe_drop` function to be called from your `Drop::drop`
//!   implementations.
//!
//! - `DeepSafeDrop` trait to be implemented by your types that use
//!   `deep_safe_drop`.
//!
//! Stack overflow is avoided by mutating a tree to become a leaf, i.e. no
//! longer have any children, before the compiler does its automatic recursive
//! dropping of fields.  Instead of using recursive function calls
//! (i.e. recording the continuations on the limited stack) to enable working
//! back up a tree branch (as the compiler's dropping does, which is what could
//! otherwise cause stack overflows), we reuse a link of each node to record
//! which parent node must be worked back up to.  Thus, we are guaranteed to
//! already have the amount of space needed for our "continuations", no matter
//! how extremely deep it may need to be, and it is OK to reuse this space
//! because the links it previously contained are already being dropped anyway.
//!
//! See the included tests for some examples.


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
)]


use core::ops::DerefMut;


/// Implement this for your tree node type, with `Link` as your tree link type
/// that dereferences to your node type.
///
/// The `Link` type may be the same as the `Self` type, when possible, which
/// might be convenient.  Or, they can be different.
///
/// Many `Self` types should be able to implement these methods without needing
/// any extra state beyond their normal state.
pub trait DeepSafeDrop<Link>
{
    /// Supply the first child and replace the link to it with a non-link, if
    /// the current state of `self` has at least one child.
    fn take_first_child(&mut self) -> Option<Link>;

    /// Supply the first child and replace the link to it with a given
    /// replacement that links to the parent of `self`, if the current state of
    /// `self` has at least one child.
    fn replace_first_child_with_parent(&mut self, parent: Link)
        -> ReplacedFirstChild<Link>;

    /// Supply the next child and replace the link to it with a non-link, if the
    /// current state of `self` has another child that has not been supplied
    /// yet.
    fn take_next_child(&mut self) -> Option<Link>;
}

/// Result of `DeepSafeDrop::replace_first_child_with_parent`.
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
pub enum ReplacedFirstChild<Link> {
    /// There was a first child and it was replaced by the parent.
    Yes {
        /// The first child that was taken.
        first_child: Link
    },
    /// There was not any child, so no replacement was done, so the parent must
    /// be returned back.
    No {
        /// The same parent that was given to the function call.
        returned_parent: Link
    },
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
    take_first_child(node)
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
    use ReplacedFirstChild::{No, Yes};

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

/// To be called from your `Drop::drop` implementations, to ensure that stack
/// overflow is avoided.
///
/// The `RootNode` type may be different than the primary `Link::Target` node
/// type, when possible, which might be convenient.  Or, they can be the same.
#[inline]
pub fn deep_safe_drop<RootNode, Link>(root: &mut RootNode)
where
    RootNode: DeepSafeDrop<Link> + ?Sized,
    Link: DerefMut,
    Link::Target: DeepSafeDrop<Link>,
{
    if let Some(first_child) = take_first_child(root) {
        main_deep_safe_drop(first_child);

        while let Some(next_child) = root.take_next_child() {
            main_deep_safe_drop(next_child);
        }
    }
}


#[cfg(test)]
mod tests;
