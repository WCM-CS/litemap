use ph::{
    BuildDefaultSeededHasher,
    phast::{
        DefaultCompressedArray, Function2, Params, ShiftOnlyWrapped,
        bits_per_seed_to_100_bucket_size,
    },
    seeds::BitsFast,
};
use std::{hash::Hash, marker::PhantomData, sync::Arc};
use bumpalo::Bump;

pub struct FrozenMap<'w, K, V>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default,
{
    index: Mphf,
    keys: &'w [K],
    values: &'w [V],
    _arena: &'w Bump
}

type Mphf =
    Function2<BitsFast, ShiftOnlyWrapped<2>, DefaultCompressedArray, BuildDefaultSeededHasher>;


// only use if the key value pair indexes line up properly
impl<'w, K, V> FrozenMap<'w, K, V>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default,
{
    #[inline]
    pub fn init(keys: &[K], values: &[V], arena: &'w Bump) -> Self { // only use if the key value pair indexes line up properly
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

        let sorted_keys = arena.alloc_slice_clone(keys); // clone we cannot assume copy, val migth be a strig or key
        let sorted_values = arena.alloc_slice_clone(values);

        for (k, v) in keys.iter().zip(values.iter()) {
            let idx = index_map.get(&k); 

            sorted_keys[idx] = k.clone();
            sorted_values[idx] = v.clone();
        }

        Self {
            index: index_map,
            keys: sorted_keys,
            values: sorted_values,
            _arena: arena
        }
    }


    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        let idx = self.index.get(key);

        if &self.keys[idx] == key {
            Some(&self.values[idx])
        } else {
            None
        }
    }

 

    #[inline]
    pub fn contains(&self, key: &K) -> bool {
        let idx = self.index.get(key);

        &self.keys[idx] == key
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.keys.len()
    }
}
