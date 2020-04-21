use super::*;


type Link = Box<dyn MyDeepSafeDrop>;

trait MyDeepSafeDrop
{
    fn take_first_child(&mut self) -> Option<Link>;

    fn replace_first_child_with_parent(&mut self, parent: Link)
        -> ReplacedFirstChild<Link>;

    fn take_next_child(&mut self) -> Option<Link>;
}

impl DeepSafeDrop for Link
{
    fn take_first_child(&mut self) -> Option<Self> {
        MyDeepSafeDrop::take_first_child(&mut **self)
    }

    fn replace_first_child_with_parent(&mut self, parent: Self)
        -> ReplacedFirstChild<Self>
    {
        MyDeepSafeDrop::replace_first_child_with_parent(&mut **self, parent)
    }

    fn take_next_child(&mut self) -> Option<Self> {
        MyDeepSafeDrop::take_next_child(&mut **self)
    }
}


struct List (Option<Link>);

impl List {
    fn new(size: usize, tail: Option<Link>) -> Link {
        let list = (0 .. size).fold(Self(tail), |acc, _| Self(Some(Box::new(acc))));
        Box::new(list)
    }
}

impl MyDeepSafeDrop for List
{
    fn take_first_child(&mut self) -> Option<Link> {
        self.0.take()
    }

    fn replace_first_child_with_parent(&mut self, parent: Link)
        -> ReplacedFirstChild<Link>
    {
        if let Some(child) = self.0.take() {
            self.0 = Some(parent);
            ReplacedFirstChild::Yes { first_child: child }
        } else {
            ReplacedFirstChild::No { returned_parent: parent }
        }
    }

    fn take_next_child(&mut self) -> Option<Link> {
        None
    }
}

/// Comment this out to cause stack overflow.
impl Drop for List {
    fn drop(&mut self) {
        if let Some(link) = self.0.take() {
            deep_safe_drop(link);
        }
    }
}


struct BinaryTree {
    left: Option<Link>,
    right: Option<Link>
}

impl BinaryTree {
    fn new(left: Option<Link>, right: Option<Link>) -> Link {
        let tree = Self { left, right };
        Box::new(tree)
    }
}

impl MyDeepSafeDrop for BinaryTree
{
    fn take_first_child(&mut self) -> Option<Link> {
        self.left.take()
    }

    fn replace_first_child_with_parent(&mut self, parent: Link)
        -> ReplacedFirstChild<Link>
    {
        if let Some(child) = self.left.take() {
            self.left = Some(parent);
            ReplacedFirstChild::Yes { first_child: child }
        } else {
            ReplacedFirstChild::No { returned_parent: parent }
        }
    }

    fn take_next_child(&mut self) -> Option<Link> {
        self.right.take()
    }
}

impl Drop for BinaryTree {
    fn drop(&mut self) {
        if let Some(link) = self.left.take() {
            deep_safe_drop(link);
        }
        if let Some(link) = self.right.take() {
            deep_safe_drop(link);
        }
    }
}


const FAN_DEGREE: usize = 2;
const STRETCH_LEN: usize = TREE_SIZE / 7;

fn make_stretched_fan(fan_degree: usize, stretch_len: usize) -> Link
{
    let branch = || {
        let tail = Some(make_stretched_fan(fan_degree - 1, stretch_len));
        List::new(stretch_len, tail)
    };

    if fan_degree >= 1 {
        let left = branch();
        let right = branch();
        let tree = BinaryTree::new(Some(left), Some(right));
        List::new(stretch_len, Some(tree))
    }
    else {
        BinaryTree::new(None, None)
    }
}

#[test]
fn no_stack_overflow() {
    let _fan = make_stretched_fan(FAN_DEGREE, STRETCH_LEN);
}
