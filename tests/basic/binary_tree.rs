use super::*;


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
    fn take_child_at_index_0(&mut self) -> Option<L> {
        self.left.take()
    }

    fn set_parent_at_index_0(&mut self, parent: L) -> SetParent<L>
    {
        if let Some(child) = self.left.take() {
            self.left = Some(parent);
            SetParent::YesReplacedChild { child0: child }
        } else {
            SetParent::No { returned_parent: parent }
        }
    }

    fn take_next_child_at_pos_index(&mut self) -> Option<L> {
        self.right.take()
    }
}


#[test]
fn exercise()
{
    use core::convert::TryInto;

    struct BinaryTreeBox (Box<BinaryTree<Self>>);

    impl NewLink<BinaryTree<Self>> for BinaryTreeBox {
        fn new(tree: BinaryTree<Self>) -> Self {
            Self(Box::new(tree))
        }
    }

    impl Borrow<BinaryTree<Self>> for BinaryTreeBox {
        fn borrow(&self) -> &BinaryTree<Self> {
            #![allow(clippy::unreachable)]
            unreachable!()
        }
    }

    impl BorrowMut<BinaryTree<Self>> for BinaryTreeBox {
        fn borrow_mut(&mut self) -> &mut BinaryTree<Self> {
            &mut self.0
        }
    }

    impl Drop for BinaryTreeBox {
        fn drop(&mut self) {
            deep_safe_drop::<_, Self, BinaryTree<Self>>(&mut *self.0);
        }
    }


    fn fan_depth(size: usize) -> usize {
        fn log2(x: usize) -> u32 {
            (usize::BITS - 1) - x.leading_zeros()
        }
        assert!(0 < size && size < usize::MAX);
        #[allow(clippy::expect_used)]
        (log2(size + 1) - 1).try_into().expect("impossible")
    }

    let fan = BinaryTree::<BinaryTreeBox>::make_fan(fan_depth(TREE_SIZE));
    drop(fan);
}
