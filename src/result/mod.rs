pub use self::Result::{None, Value, Start, Finish};

type NonZeroUsize = &'static u8;

#[derive(Clone, Copy)]
pub struct Id(NonZeroUsize);

#[derive(Clone)]
pub enum Result<T> {
    None,
    Start { id: Id, parent: Option<Id> },
    Value {
        id: Id,
        parent: Option<Id>,
        value: T,
    },
    Finish { id: Id, parent: Option<Id> },
}

impl<T> Copy for Result<T>
where
    T: Clone + Copy,
{
}

impl<T> Result<T> {
    pub fn value(self) -> Option<T> {
        match self {
            Value { value, .. } => Option::Some(value),
            _ => Option::None,
        }
    }

    pub fn id(&self) -> Option<Id> {
        match *self {
            None => Option::None,
            Start { id, .. } => Option::Some(id),
            Value { id, .. } => Option::Some(id),
            Finish { id, .. } => Option::Some(id),
        }
    }

    pub fn parent(&self) -> Option<Option<Id>> {
        match *self {
            None => Option::None,
            Start { parent, .. } => Option::Some(parent),
            Value { parent, .. } => Option::Some(parent),
            Finish { parent, .. } => Option::Some(parent),
        }
    }

    pub fn as_ref(&self) -> Result<&T> {
        match *self {
            None => None,
            Start { id, parent } => Start { id, parent },
            Value {
                id,
                parent,
                ref value,
            } => Value { id, parent, value },
            Finish { id, parent } => Finish { id, parent },
        }
    }

    pub fn as_mut(&mut self) -> Result<&mut T> {
        match *self {
            None => None,
            Start { id, parent } => Start { id, parent },
            Value {
                id,
                parent,
                ref mut value,
            } => Value { id, parent, value },
            Finish { id, parent } => Finish { id, parent },
        }
    }
}
