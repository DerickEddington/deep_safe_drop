use super::*;
use std::ops::{Deref, DerefMut};


pub(super) struct BinaryTree<L> {
    pub(super) left: Option<L>,
    pub(super) right: Option<L>
}

impl<L> BinaryTree<L>
{
    fn make_fan(depth: usize) -> Self
    where
        L: NewLink<Self>
    {
        let mut fan = Self { left: None, right: None };

        if depth > 0 {
            fan.left = Some(L::new(Self::make_fan(depth.saturating_sub(1))));
            fan.right = Some(L::new(Self::make_fan(depth.saturating_sub(1))));
        }

        fan
    }
}

impl<L> DeepSafeDrop<L> for BinaryTree<L>
{
    fn take_first_child(&mut self) -> Option<L> {
        self.left.take()
    }

    fn replace_first_child_with_parent(&mut self, parent: L)
        -> ReplacedFirstChild<L>
    {
        if let Some(child) = self.left.take() {
            self.left = Some(parent);
            ReplacedFirstChild::Yes { first_child: child }
        } else {
            ReplacedFirstChild::No { returned_parent: parent }
        }
    }

    fn take_next_child(&mut self) -> Option<L> {
        self.right.take()
    }
}


#[test]
fn exercise() -> Result<(), std::num::TryFromIntError>
{
    use std::convert::TryInto;

    struct BinaryTreeBox (Box<BinaryTree<Self>>);

    impl NewLink<BinaryTree<Self>> for BinaryTreeBox {
        fn new(tree: BinaryTree<Self>) -> Self {
            Self(Box::new(tree))
        }
    }

    impl Deref for BinaryTreeBox {
        type Target = BinaryTree<Self>;

        fn deref(&self) -> &Self::Target {
            &*self.0
        }
    }

    impl DerefMut for BinaryTreeBox {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut *self.0
        }
    }

    impl Drop for BinaryTreeBox {
        fn drop(&mut self) {
            deep_safe_drop(&mut **self);
        }
    }


    const fn fan_depth(size: usize) -> u32 {
        // assert!(0 < size && size < MAX);
        const WIDTH: u32 = usize::max_value().count_ones();
        ((WIDTH - 1) - (size + 1).leading_zeros()) - 1
    }

    const FAN_DEPTH: u32 = fan_depth(TREE_SIZE);

    let fan = BinaryTree::<BinaryTreeBox>::make_fan(FAN_DEPTH.try_into()?);
    drop(fan);

    Ok(())
}
