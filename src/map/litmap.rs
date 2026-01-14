use ph::{
    BuildDefaultSeededHasher,
    phast::{
        DefaultCompressedArray, Function2, Params, ShiftOnlyWrapped,
        bits_per_seed_to_100_bucket_size,
    },
    seeds::BitsFast,
};
use std::{hash::Hash};

use crate::index::prelude::*;
use crate::store::prelude::*;
use init_vec::*;

//  SyncVerifiedFrozenMap    // higher overhead // no thread safe // key verification

pub struct FrozenMap<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default,
{
    index: VerifiedIndex<K>,
    store: Store<V>,
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

        let frozen_index = VerifiedIndex {
            mphf: index_map,
            keys: WithKeys::new(sorted_keys)
        };

        let store = Store::new(sorted_values);

        Self {
            index: frozen_index,
            store,
        }
    }


    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        let idx = self.index.get_index(key);


        if self.index.keys.get(idx) != key {
            return None;
        }

        self.store.get_value(idx)
    }

    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let idx = self.index.get_index(key);



        if self.index.keys.get(idx) != key {
            return None;
        }

        self.store.get_mut_value(idx)
    }


    #[inline]
    pub fn contains(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }

    #[inline]
    pub fn contains_value(&self, key: &K) -> bool {
        let idx = self.index.get_index(key);
        !self.store.get_value(idx).is_none()
    }


    #[inline]
    pub fn len(&self) -> usize {
        self.index.keys.len()
    }

    

    #[inline]
    pub fn iter_keys(&self) -> impl Iterator<Item = K> {
        self.index.keys.get_keys().into_iter()
    }


   
}



// we probably shouldnt use this
pub mod init_vec {
    use std::mem::MaybeUninit;

    pub struct InitVec<T> {
        vec: Vec<MaybeUninit<T>>,
    }
    
    impl<T> InitVec<T> {
        pub fn new(len: usize) -> Self {
            let mut vec = Vec::with_capacity(len);
            unsafe { vec.set_len(len); }
            Self { vec }
        }


        pub fn push(&mut self, idx: usize, el: T) {
            self.vec[idx].write(el);
        }

        pub fn into_vec(self) -> Vec<T> {
            unsafe { std::mem::transmute(self.vec) }
        }


    }

}

