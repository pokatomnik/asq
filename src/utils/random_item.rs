use rand::seq::IndexedRandom;

pub(crate) trait RandomItem<'a, T> {
    fn random(&'a self) -> Option<&'a T>;
}

impl<'a, T> RandomItem<'a, T> for Vec<T> {
    fn random(&'a self) -> Option<&'a T> {
        let mut rng = rand::rng();
        self.choose(&mut rng)
    }
}
