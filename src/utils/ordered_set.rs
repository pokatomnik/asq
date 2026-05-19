use std::{collections::HashSet, hash::Hash, rc::Rc};

pub(crate) struct OrderedSet<V> {
    set: std::collections::HashSet<Rc<V>>,
    values: Vec<Rc<V>>,
}

impl<V> OrderedSet<V>
where
    V: Eq + Hash,
{
    pub fn with_capacity(cap: usize) -> Self {
        OrderedSet {
            set: HashSet::with_capacity(cap),
            values: Vec::with_capacity(cap),
        }
    }

    pub fn insert(&mut self, value: V) -> bool {
        let val = Rc::new(value);

        let inserted = self.set.insert(val.clone());

        if inserted {
            self.values.push(val.clone());
        }

        inserted
    }

    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.values.iter().map(|v| v.as_ref())
    }
}
