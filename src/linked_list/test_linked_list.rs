use super::*;
use std::rc::Rc;

fn mk_list_i32(items: &[i32]) -> List<i32> {
    let mut acc = List::Nil;
    for item in items.iter().rev() {
        acc = List::Cons(*item, Rc::new(acc));
    }
    acc
}

#[test]
fn reverse_reverses_items() {
    let list = mk_list_i32(&[1, 2, 3]);
    let reversed = list.reverse();
    let expected = mk_list_i32(&[3, 2, 1]);
    assert_eq!(reversed, expected);
}

#[test]
fn reverse_on_nil_returns_nil() {
    let list: List<i32> = List::Nil;
    assert_eq!(list.reverse(), List::Nil);
}

#[test]
fn display_formats_chain() {
    let list = mk_list_i32(&[1, 2]);
    assert_eq!(format!("{}", list), "1 -> 2 -> <Nil>");
}
