use ph::{
    BuildDefaultSeededHasher,
    phast::{
        DefaultCompressedArray, Function2, Params, ShiftOnlyWrapped,
        bits_per_seed_to_100_bucket_size,
    },
    seeds::BitsFast,
};
use std::{hash::Hash};

pub struct LitMa<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default,
{
    keys: Box<[K]>,
    values: Box<[V]>,
    index: Mphf, // Structure this so the Index is last/cold since its only used once per lokup
}

type Mphf =
    Function2<BitsFast, ShiftOnlyWrapped<2>, DefaultCompressedArray, BuildDefaultSeededHasher>;


// only use if the key value pair indexes line up properly
impl<K, V> LitMa<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default,
{
    //#[inline]
    pub fn init(keys: Vec<K>, values: Vec<V>) -> Self { // only use if the key value pair indexes line up properly
        let n = keys.len();
        assert_eq!(n, values.len());

        let (bits, bucket_bits): (BitsFast, u8) = match n {
            0..=1000 => (BitsFast(10), 8),
            1001..=10_000 => (BitsFast(12), 10),
            10_001..=100_000 => (BitsFast(14), 12),
            _ => (BitsFast(16), 14), // 100K+
        };

        // MPHF creation
        let index_map: Function2<
            BitsFast,
            ShiftOnlyWrapped<2>,
            DefaultCompressedArray,
            BuildDefaultSeededHasher,
        > = Function2::with_slice_p_threads_hash_sc(
            &keys,
            &Params::new(bits, bits_per_seed_to_100_bucket_size(bucket_bits)),
            std::thread::available_parallelism().map_or(1, |v| v.into()),
            BuildDefaultSeededHasher::default(),
            ShiftOnlyWrapped::<2>,
        );

        let mut sorted_keys: Vec<K> = vec![K::default(); n];
        let mut sorted_values: Vec<V> = vec![V::default(); n];

        for (k, v) in keys.into_iter().zip(values.into_iter()) {
            let idx = index_map.get(&k); 

            sorted_keys[idx] = k;
            sorted_values[idx] = v;
        }

        Self {
            index: index_map,
            keys: sorted_keys.into_boxed_slice(),
            values: sorted_values.into_boxed_slice()
        }
    }

    #[inline(always)]
    pub fn get(&self, key: &K) -> Option<&V> {
        let idx = self.index.get(key);

        if &self.keys[idx] == key {
            Some(&self.values[idx])
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn contains(&self, key: &K) -> bool {
        let idx = self.index.get(key);

        &self.keys[idx] == key
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.keys.len()
    }
}