#![allow(clippy::std_instead_of_core)]

extern crate std;
use std::prelude::v1::*;
use super::*;


mod list;
mod binary_tree;
mod dyn_trait;


/// This results in tree depths that are enough to cause stack overflows when
/// `deep_safe_drop` is not used for a `Drop` impl.  You may increase this but
/// more RAM will be required.
const TREE_SIZE: usize = 2_usize.pow(20);


trait NewLink<Node> {
    fn new(node: Node) -> Self;
}


use list::List;
use binary_tree::BinaryTree;

#[allow(clippy::trait_duplication_in_bounds)]  // Shouldn't be needed, if it was smarter.
fn make_stretched_fan<L>(fan_degree: usize, stretch_len: usize) -> L
where
    L: NewLink<List<L>> + NewLink<BinaryTree<L>>
{
    let branch = || {
        let tail = Some(make_stretched_fan(fan_degree.saturating_sub(1), stretch_len));
        L::new(List::make(stretch_len, tail))
    };

    if fan_degree >= 1 {
        let left = Some(branch());
        let right = Some(branch());
        let branches = Some(L::new(BinaryTree { left, right }));
        L::new(List::make(stretch_len, branches))
    }
    else {
        L::new(BinaryTree { left: None, right: None })
    }
}
