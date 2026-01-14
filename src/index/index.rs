use ph::{
    BuildDefaultSeededHasher,
    phast::{DefaultCompressedArray, Function2, ShiftOnlyWrapped},
    seeds::BitsFast,
};
use std::{hash::Hash, marker::PhantomData, mem::MaybeUninit};

type Mphf =
    Function2<BitsFast, ShiftOnlyWrapped<2>, DefaultCompressedArray, BuildDefaultSeededHasher>;

pub type VerifiedIndex<K> = FrozenIndex<WithKeys<K>>;
pub type UnverifiedIndex<K> = FrozenIndex<NoKeys<K>>;

pub struct FrozenIndex<S>
where
    S: KeyStorage,
    S::Key: Hash + Eq + Clone + Send + Sync + Default,
{
    pub mphf: Mphf,
    pub keys: S,
}

impl<S> FrozenIndex<S>
where
    S: KeyStorage,
    S::Key: Hash + Eq + Clone + Send + Sync + Default,
{
    #[inline]
    pub fn get_index(&self, key: &S::Key) -> usize {
        self.mphf.get(key)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.keys.len()
    }
}

impl<K> FrozenIndex<WithKeys<K>>
where
    K: Hash + Eq + Clone + Send + Sync + Default,
{
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        let idx = self.get_index(key);

        self.keys.get(idx) == key
    }
}

pub trait KeyStorage {
    type Key;

    fn get(&self, idx: usize) -> &Self::Key;
    fn len(&self) -> usize;
}

pub struct WithKeys<K> {
    keys: Vec<K>,
    len: usize,
}

impl<K> WithKeys<K>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
{
    pub fn new(keys: Vec<K>) -> Self {
        let n = keys.len();

        Self {
            keys,
            len: n,
        }
    }

    pub fn get_keys(&self) -> Vec<K> {
        self.keys.to_vec()
    }
}


pub struct NoKeys<K> {
    _ghost: PhantomData<K>,
    len: usize,
}

impl<K> NoKeys<K> {
    pub fn new(len: usize) -> Self {

        Self {
            _ghost: PhantomData,
            len,
        }
    }
}

impl<K> KeyStorage for WithKeys<K> {
    type Key = K;

    #[inline]
    fn get(&self, idx: usize) -> &K {
        &self.keys[idx]
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }


}

impl<K> KeyStorage for NoKeys<K> {
    type Key = K;

    #[inline]
    fn get(&self, _: usize) -> &K {
        unreachable!("unverified index does not store keys")
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

}
