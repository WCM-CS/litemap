use ph::{
    BuildDefaultSeededHasher,
    phast::{
        DefaultCompressedArray, Function2, Params, ShiftOnlyWrapped,
        bits_per_seed_to_100_bucket_size,
    },
    seeds::BitsFast,
};
use std::{hash::Hash};

pub struct FrozenMap<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default,
{
    index: FrozenIndex<K>,
    store: Store<V>,
}

type Mphf =
    Function2<BitsFast, ShiftOnlyWrapped<2>, DefaultCompressedArray, BuildDefaultSeededHasher>;

pub struct FrozenIndex<K>
where
    K: Hash + Eq + Clone + Send + Sync + Default,
{
    pub mphf: Mphf,
    pub keys: Store<K>,
}


// only use if the key value pair indexes line up properly
impl<K, V> FrozenMap<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default,
{
    #[inline]
    pub fn init(keys: Vec<K>, values: Vec<V>) -> Self { // only use if the key value pair indexes line up properly
        assert_eq!(keys.len(), values.len());
        // NOTE: all keys must be unqiue!!!

        let index_map: Function2<
            BitsFast,
            ShiftOnlyWrapped<2>,
            DefaultCompressedArray,
            BuildDefaultSeededHasher,
        > = Function2::with_slice_p_threads_hash_sc(
            &keys,
            &Params::new(BitsFast(10), bits_per_seed_to_100_bucket_size(8)),
            std::thread::available_parallelism().map_or(1, |v| v.into()),
            BuildDefaultSeededHasher::default(),
            ShiftOnlyWrapped::<2>,
        );

        let mut sorted_keys: Vec<K> = vec![K::default(); keys.len()]; // initvec
        let mut sorted_values: Vec<V> = vec![V::default(); values.len()];
    

        for (i, (k, v)) in keys.into_iter().zip(values.into_iter()).enumerate() {
            sorted_keys[i] = k;
            sorted_values[i] = v;
        }

        let frozen_index = FrozenIndex {
            mphf: index_map,
            keys: Store::new(sorted_keys)
        };

        let store = Store::new(sorted_values);

        Self {
            index: frozen_index,
            store,
        }
    }


    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        let idx = self.index.mphf.get(key);

        match self.index.keys.get_value(idx) {
            Some(k) => {
                if k == key {
                    self.store.get_value(idx)
                } else {
                    None
                }
            },
            None => None,
        }
    }

    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let idx = self.index.mphf.get(key);

        match self.index.keys.get_value(idx) {
            Some(k) => {
                if k == key {
                    self.store.get_mut_value(idx)
                } else {
                    None
                }
            },
            None => None,
        }
    }

    #[inline]
    pub fn contains(&self, key: &K) -> bool {
        let idx = self.index.mphf.get(key);

        self.index.keys.get_value(idx) == Some(key)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.index.keys.len
    }
}




pub struct Store<V>
where
    V: Send + Sync + Clone,
{
    pub value_ptr: *mut V,
    pub len: usize
}

impl<V> Store<V>
where
    V: Send + Sync + Clone,
{
    #[inline]
    pub fn new(values: Vec<V>) -> Self {
        let mut values = values;
        let n = values.len();

        let value_ptr = values.as_mut_ptr();
        std::mem::forget(values); // so it doesnt drop from mem after var is descoped


        Self { value_ptr, len: n }
    }

    #[inline]
    pub fn get_value(&self, idx: usize) -> Option<&V> {
        if idx >= self.len || self.value_ptr.is_null() { // check index bounds and value
            None
        } else {
            unsafe { Some( &*self.value_ptr.add(idx) ) }
        }
    }

    #[inline]
    pub fn get_mut_value(&mut self, idx: usize) -> Option<&mut V> {
        if idx >= self.len || self.value_ptr.is_null() {
            None
        } else {
            unsafe { Some(&mut *self.value_ptr.add(idx) ) }
        }
    }
}

impl<V> Drop for Store<V>
where
    V: Send + Sync + Clone,
{
    fn drop(&mut self) {
        if !self.value_ptr.is_null() {
            // assumes data is full otherwise undefined behaviore occcurs, so use unsafe init and dont delete vals until changed to store capacity in store with current len
            unsafe { let _ = Vec::from_raw_parts(self.value_ptr, self.len, self.len); }
        }
    }
}



