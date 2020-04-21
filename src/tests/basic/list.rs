use super::*;

struct List (Option<Box<Self>>);

impl DeepSafeDrop for Box<List>
{
    fn take_first_child(&mut self) -> Option<Self> {
        self.0.take()
    }

    fn replace_first_child_with_parent(&mut self, parent: Self)
        -> ReplacedFirstChild<Self>
    {
        if let Some(child) = self.0.take() {
            self.0 = Some(parent);
            ReplacedFirstChild::Yes { first_child: child }
        } else {
            ReplacedFirstChild::No { returned_parent: parent }
        }
    }

    fn take_next_child(&mut self) -> Option<Self> {
        None
    }
}

/// Comment this out to cause stack overflow.
impl Drop for List {
    fn drop(&mut self) {
        if let Some(box_list) = self.0.take() {
            deep_safe_drop(box_list);
        }
    }
}

const LIST_LEN: usize = TREE_SIZE;

fn make_list(size: usize) -> List {
    (0 .. size).fold(List(None), |acc, _| List(Some(Box::new(acc))))
}

#[test]
fn no_stack_overflow() {
    let _list = make_list(LIST_LEN);
}
