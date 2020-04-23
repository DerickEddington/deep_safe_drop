use super::{*, list::List, binary_tree::BinaryTree};
use std::ops::{Deref, DerefMut};


/// Used as both the `Link` and the `Node` types.
pub struct DynBox (pub Box<dyn DeepSafeDrop<Self>>);

impl Deref for DynBox {
    /// Dereferences to itself!
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl DerefMut for DynBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

/// Needed because it's used as the `Node` type.
impl DeepSafeDrop<Self> for DynBox
{
    fn take_first_child(&mut self) -> Option<Self> {
        self.0.take_first_child()
    }

    fn replace_first_child_with_parent(&mut self, parent: Self)
        -> ReplacedFirstChild<Self>
    {
        self.0.replace_first_child_with_parent(parent)
    }

    fn take_next_child(&mut self) -> Option<Self> {
        self.0.take_next_child()
    }
}

/// Comment-out to cause stack overflow.
impl Drop for DynBox {
    fn drop(&mut self) {
        deep_safe_drop(self)
    }
}

impl NewLink<List<Self>> for DynBox {
    fn new(node: List<Self>) -> Self {
        Self(Box::new(node))
    }
}

impl NewLink<BinaryTree<Self>> for DynBox {
    fn new(node: BinaryTree<Self>) -> Self {
        Self(Box::new(node))
    }
}


const FAN_DEGREE: usize = 2;

#[allow(clippy::integer_division)]
const STRETCH_LEN: usize = TREE_SIZE / 7;


#[test]
fn no_stack_overflow()
{
    let fan: DynBox = make_stretched_fan(FAN_DEGREE, STRETCH_LEN);
    drop(fan);
}


#[test]
#[ignore]
fn stack_overflow()
{
    struct DynBox (Box<dyn DeepSafeDrop<Self>>);

    impl NewLink<List<Self>> for DynBox {
        fn new(node: List<Self>) -> Self {
            Self(Box::new(node))
        }
    }

    impl NewLink<BinaryTree<Self>> for DynBox {
        fn new(node: BinaryTree<Self>) -> Self {
            Self(Box::new(node))
        }
    }

    let fan: DynBox = make_stretched_fan(FAN_DEGREE, STRETCH_LEN);
    drop(fan);
}
