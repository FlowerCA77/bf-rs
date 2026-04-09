use std::{fmt, rc::Rc};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum List<T> {
    Cons(T, Rc<List<T>>),
    Nil,
}

impl<T: Clone> List<T> {
    pub fn reverse(&self) -> List<T> {
        fn go<T: Clone>(acc: List<T>, rest: &List<T>) -> List<T> {
            match rest {
                List::Nil => acc,
                List::Cons(head, tail) => go(List::Cons(head.clone(), Rc::new(acc)), tail),
            }
        }
        go(List::Nil, self)
    }
}

impl<T: fmt::Display> List<T> {
    fn to_str(&self) -> String {
        match self {
            List::Cons(x, cons) => fmt::format(format_args!("{} -> {}", x, cons.to_str())),
            List::Nil => "<Nil>".to_string(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

pub use List::Cons;
pub use List::Nil;
