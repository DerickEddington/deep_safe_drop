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
    fn take_child_at_index_0(&mut self) -> Option<L> {
        self.0.take()
    }

    fn set_parent_at_index_0(&mut self, parent: L) -> SetParent<L> {
        if let Some(child) = self.0.take() {
            self.0 = Some(parent);
            SetParent::YesReplacedChild { child0: child }
        } else {
            SetParent::No { returned_parent: parent }
        }
    }

    fn take_next_child_at_pos_index(&mut self) -> Option<L> {
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
            #![allow(clippy::unreachable)]
            unreachable!()
        }
    }

    impl BorrowMut<List<Self>> for ListBox {
        fn borrow_mut(&mut self) -> &mut List<Self> {
            &mut self.0
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
    struct ListBox (#[allow(dead_code)] Box<List<Self>>);

    impl NewLink<List<Self>> for ListBox {
        fn new(list: List<Self>) -> Self {
            Self(Box::new(list))
        }
    }

    let list = List::<ListBox>::make(LIST_LEN, None);
    drop(list);
}
