use super::*;


pub(super) struct List<L> (Option<L>); 

impl<L> List<L>
{
    pub(super) fn make(size: usize, tail: Option<L>) -> Self
    where
        L: NewLink<Self>
    {
        (0 .. size).fold(Self(tail), |acc, _| Self(Some(L::new(acc))))
    }
}

impl<L> DeepSafeDrop<L> for List<L>
{
    fn take_first_child(&mut self) -> Option<L> {
        self.0.take()
    }

    fn replace_first_child_with_parent(&mut self, parent: L)
        -> ReplacedFirstChild<L>
    {
        if let Some(child) = self.0.take() {
            self.0 = Some(parent);
            ReplacedFirstChild::Yes { first_child: child }
        } else {
            ReplacedFirstChild::No { returned_parent: parent }
        }
    }

    fn take_next_child(&mut self) -> Option<L> {
        None
    }
}


const LIST_LEN: usize = TREE_SIZE;


#[test]
fn no_stack_overflow()
{
    struct ListBox (Box<List<Self>>);

    impl NewLink<List<Self>> for ListBox {
        fn new(list: List<Self>) -> Self {
            Self(Box::new(list))
        }
    }

    impl Borrow<List<Self>> for ListBox {
        fn borrow(&self) -> &List<Self> {
            unreachable!()
        }
    }

    impl BorrowMut<List<Self>> for ListBox {
        fn borrow_mut(&mut self) -> &mut List<Self> {
            &mut *self.0
        }
    }

    /// Comment-out to cause stack overflow.
    impl Drop for ListBox {
        fn drop(&mut self) {
            deep_safe_drop::<List<Self>, Self, List<Self>>(&mut *self.0);
        }
    }


    let list = List::<ListBox>::make(LIST_LEN, None);
    drop(list);
}


#[test]
#[ignore]
fn stack_overflow()
{
    struct ListBox (Box<List<Self>>);

    impl NewLink<List<Self>> for ListBox {
        fn new(list: List<Self>) -> Self {
            Self(Box::new(list))
        }
    }

    let list = List::<ListBox>::make(LIST_LEN, None);
    drop(list);
}
