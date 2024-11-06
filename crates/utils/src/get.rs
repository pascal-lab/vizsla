use la_arena::{Arena, Idx};

pub trait Get<A> {
    type Output;

    fn get(&self, a: A) -> Self::Output;
}

pub trait GetRef<A> {
    type Output;

    fn get(&self, a: A) -> &Self::Output;
}

impl<T> GetRef<Idx<T>> for Arena<T> {
    type Output = T;

    fn get(&self, id: Idx<T>) -> &Self::Output {
        &self[id]
    }
}
