use super::*;

struct BinaryTree {
    left: Option<Box<Self>>,
    right: Option<Box<Self>>
}

impl DeepSafeDrop for Box<BinaryTree>
{
    fn take_first_child(&mut self) -> Option<Self> {
        self.left.take()
    }

    fn replace_first_child_with_parent(&mut self, parent: Self)
        -> ReplacedFirstChild<Self>
    {
        if let Some(child) = self.left.take() {
            self.left = Some(parent);
            ReplacedFirstChild::Yes { first_child: child }
        } else {
            ReplacedFirstChild::No { returned_parent: parent }
        }
    }

    fn take_next_child(&mut self) -> Option<Self> {
        self.right.take()
    }
}

impl Drop for BinaryTree {
    fn drop(&mut self) {
        if let Some(box_tree) = self.left.take() {
            deep_safe_drop(box_tree);
        }
        if let Some(box_tree) = self.right.take() {
            deep_safe_drop(box_tree);
        }
    }
}

const fn fan_depth(size: usize) -> usize {
    use std::usize::MAX;
    // assert!(0 < size && size < MAX);
    const WIDTH: u32 = MAX.count_ones();
    (((WIDTH - 1) - (size + 1).leading_zeros()) - 1) as usize
}

const FAN_DEPTH: usize = fan_depth(TREE_SIZE);

fn make_fan(depth: usize) -> BinaryTree
{
    let mut fan = BinaryTree { left: None, right: None };

    if depth > 0 {
        fan.left = Some(Box::new(make_fan(depth - 1)));
        fan.right = Some(Box::new(make_fan(depth - 1)));
    }

    fan
}

#[test]
fn exercise() {
    let _fan = make_fan(FAN_DEPTH);
}
