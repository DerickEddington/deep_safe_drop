use super::{
    binary_tree::BinaryTree,
    list::List,
    *,
};


/// Used as both the `Link` and the `Node` types.
struct DynBox(Box<dyn DeepSafeDrop<Self>>);

/// Needed because it's used as the `Node` type.
impl DeepSafeDrop<Self> for DynBox
{
    fn take_child_at_index_0(&mut self) -> Option<Self>
    {
        self.0.take_child_at_index_0()
    }

    fn set_parent_at_index_0(
        &mut self,
        parent: Self,
    ) -> SetParent<Self>
    {
        self.0.set_parent_at_index_0(parent)
    }

    fn take_next_child_at_pos_index(&mut self) -> Option<Self>
    {
        self.0.take_next_child_at_pos_index()
    }
}

/// Needed because it's also used as the `Link` type.
impl Link<Self> for DynBox
{
    fn get_mut(&mut self) -> &mut Self
    {
        self
    }
}

/// Comment-out to cause stack overflow.
impl Drop for DynBox
{
    fn drop(&mut self)
    {
        deep_safe_drop(self);
    }
}

impl NewLink<List<Self>> for DynBox
{
    fn new(node: List<Self>) -> Self
    {
        Self(Box::new(node))
    }
}

impl NewLink<BinaryTree<Self>> for DynBox
{
    fn new(node: BinaryTree<Self>) -> Self
    {
        Self(Box::new(node))
    }
}


const FAN_DEGREE: usize = 2;

const STRETCH_LEN: usize = TREE_SIZE.div_euclid(7);


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
    struct DynBox(#[allow(dead_code)] Box<dyn DeepSafeDrop<Self>>);

    impl NewLink<List<Self>> for DynBox
    {
        fn new(node: List<Self>) -> Self
        {
            Self(Box::new(node))
        }
    }

    impl NewLink<BinaryTree<Self>> for DynBox
    {
        fn new(node: BinaryTree<Self>) -> Self
        {
            Self(Box::new(node))
        }
    }

    let fan: DynBox = make_stretched_fan(FAN_DEGREE, STRETCH_LEN);
    drop(fan);
}
