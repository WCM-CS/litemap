pub struct Store<V>
where
    V: Send + Sync + Clone,
{
    value_ptr: *mut V,
    len: usize
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

