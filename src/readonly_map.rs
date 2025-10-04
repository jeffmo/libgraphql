use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::RandomState;
use std::ops::Index;

type TFilterFn<K, V> = fn(&(&K, &V)) -> bool;

#[derive(Clone, Debug)]
pub struct ReadOnlyMap<
    'a,
    K,
    V,
    S = RandomState,
> {
    map: &'a HashMap<K, V, S>,
    filter_fn: TFilterFn<K, V>,
}

impl<'a, K, V> ReadOnlyMap<'a, K, V, RandomState> {
    #[inline]
    pub fn new(
        map: &'a HashMap<K, V>,
        filter_fn: Option<TFilterFn<K, V>>,
    ) -> ReadOnlyMap<'a, K, V, RandomState> {
        Self {
            map,
            filter_fn: filter_fn.unwrap_or(|_| true),
        }
    }
}

impl<'a, K, V, S> ReadOnlyMap<'a, K, V, S> {
    #[inline]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(key, _val)| key)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_key, val)| val)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter().filter(&self.filter_fn)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[inline]
    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }
}

impl<
    'a,
    K: Eq + Hash,
    V,
    S: BuildHasher,
> ReadOnlyMap<'a, K, V, S> {
    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let filter_fn = &self.filter_fn;
        self.map.get_key_value(k).and_then(|(k, v)| {
            if filter_fn(&(k, v)) {
                Some(v)
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn get_key_value<Q>(&self, k: &Q) -> Option<(&'a K, &'a V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let filter_fn = &self.filter_fn;
        self.map.get_key_value(k).filter(|(k, v)| filter_fn(&(k, v)))
    }

    #[inline]
    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(k).is_some()
    }
}

impl<
    'a,
    K: Eq + Hash,
    V: PartialEq,
    S: BuildHasher,
> PartialEq for ReadOnlyMap<'a, K, V, S> {
    fn eq(&self, other: &ReadOnlyMap<'_, K, V, S>) -> bool {
        self.iter().all(|(key, value)| other.get(key).is_some_and(|v| *value == *v))
    }
}

impl<
    'a,
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
> Eq for ReadOnlyMap<'a, K, V, S> {}

impl<
    'a,
    K,
    V: Eq,
    Q,
    S: BuildHasher,
> Index<&Q> for ReadOnlyMap<'a, K, V, S>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Hash + Eq + ?Sized,
{
    type Output = V;

    #[inline]
    fn index(&self, key: &Q) -> &'a V {
        self.get(key).expect("no entry found for key")
    }
}
