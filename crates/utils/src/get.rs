pub trait Get<A> {
    type Output;
    fn get(&self, a: A) -> Self::Output {
        self.get_opt(a).unwrap()
    }
    fn get_opt(&self, a: A) -> Option<Self::Output>;
}

pub trait GetRef<A> {
    type Output;

    fn get(&self, a: A) -> &Self::Output {
        self.get_opt(a).unwrap()
    }

    fn get_opt(&self, a: A) -> Option<&Self::Output>;
}
