//! Append-only concurrent vector-like datastructure

//! ```
//! # fn main() {
//! # use std::collections::HashSet;
//! # use std::sync::Arc;
//! # use std::thread;
//! use concurrent_vec::ConcVec;
//! let conc_vec = ConcVec::new(1, 1024);
//! let mut handles = vec![];
//!
//! const N: usize = 1_024;
//! const n_threads: usize = 12;
//!
//! for t in 0..n_threads {
//!     let mut vec = conc_vec.clone().get_appender();
//!     handles.push(thread::spawn(move || {
//!         for i in 0..N {
//!             if i % n_threads == t {
//!                 vec.push(i);
//!             }
//!         }
//!     }))
//! }
//!
//! for h in handles {
//!     h.join().unwrap();
//! }
//! let len = conc_vec.len();
//! assert_eq!(len, N);
//! # let mut set = HashSet::new();
//! for i in conc_vec.lock_iter() {
//!     println!("{}", i);
//! #     set.insert(i);
//! }
//! assert_eq!(len, set.len());
//! # }
//! ```

use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

/// A concurrent vector, only supporting push and indexed access
#[derive(Debug, Clone)]
pub struct ConcVec<T> {
    data: Arc<Mutex<Option<Vec<Vec<T>>>>>,
    buf_size: usize,
}

/// This is the representation used to push data to the vec.
/// Get this by calling `conc_vec.clone().get_appender();`
pub struct BufferVec<T> {
    data: Cell<Vec<T>>,
    vec: ConcVec<T>,
    buf_size: usize,
}

impl<T> Drop for BufferVec<T> {
    fn drop(&mut self) {
        let new_vec: Cell<Vec<T>> = Cell::new(Vec::new());
        self.data.swap(&new_vec);
        self.vec.push(new_vec.into_inner());
    }
}

impl<T> BufferVec<T> {
    pub fn push(&mut self, t: T) {
        self.data.get_mut().push(t);
        if self.data.get_mut().len() == self.buf_size {
            //let new_vec: Cell<Vec<T>> = Cell::new(Vec::with_capacity(self.buf_size));
            let new_vec: Cell<Vec<T>> = Cell::new(Vec::new());
            self.data.swap(&new_vec); // change to mem::swap
            self.vec.push(new_vec.into_inner());
        }
    }
}

impl<T> ConcVec<T> {
    /// Make a new Aoavec with a default buf size and initial capacity
    pub fn new(size: usize, buf_size: usize) -> Self {
        ConcVec {
            data: Arc::from(Mutex::from(Some(Vec::with_capacity(size)))),
            buf_size,
        }
    }

    /// Returns the length of self.
    /// # Performance
    /// ## O(n)
    /// This has to iterate over all vecs, so at minimum
    /// `self.len_estimate` / BUF_SIZE times
    pub fn len(&self) -> usize {
        let mut size = 0;
        for vec in self.data.lock().unwrap().as_ref().unwrap() {
            size += vec.len();
        }

        size
    }

    /// Returns an upper bound of the length of self.
    /// # Performance
    /// ## O(1)
    /// this only needs to take the Mutex
    pub fn len_estimate(&self) -> usize {
        self.data.lock().unwrap().as_ref().unwrap().len() * self.buf_size
    }

    /// Returns if self is empty.
    /// # Performance
    /// ## O(1)
    /// this only needs to take the Mutex
    pub fn is_empty(&self) -> bool {
        self.data.lock().unwrap().as_ref().unwrap().is_empty()
    }

    /// Use this function to get the struct to append elements to the vec
    pub fn get_appender(self) -> BufferVec<T> {
        BufferVec {
            data: Default::default(),
            buf_size: self.buf_size,
            vec: self,
        }
    }

    /// Use this function to get the struct to append elements to the vec with a size
    pub fn get_appender_with_size(self, buf_size: usize) -> BufferVec<T> {
        BufferVec {
            data: Default::default(),
            buf_size,
            vec: self,
        }
    }

    /// Push a vec to data
    pub(crate) fn push(&self, t: Vec<T>) {
        self.data.lock().unwrap().as_mut().unwrap().push(t)
    }

    /// Use this function to retrieve data from the vec.
    /// Takes All data out of the Vec
    pub fn lock_iter(&self) -> VecIter<T> {
        let data = self.data.lock().unwrap().replace(Vec::new()).unwrap();
        VecIter::<T> {
            data,
            outer: Vec::new(),
        }
    }
}

impl<T> Default for ConcVec<T> {
    fn default() -> Self {
        ConcVec::new(0, 32)
    }
}

/// Iterator over a the internal datastructure of a ConcVec
pub struct VecIter<T> {
    data: Vec<Vec<T>>,
    outer: Vec<T>,
}

impl<'a, T> std::iter::Iterator for VecIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.outer.is_empty() {
            self.outer = self.data.pop()?;
        }
        self.outer.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashSet, thread};

    #[test]
    fn base_one() {
        let conc_vec = ConcVec::new(1, 1);
        let mut vec = conc_vec.clone().get_appender();
        let n = 512;

        for i in 0..n {
            vec.push(i);
        }

        let mut set = HashSet::new();
        for i in conc_vec.lock_iter() {
            println!("{}", i);
            set.insert(i);
        }
    }

    #[test]
    fn base_thirtytwo() {
        let conc_vec = ConcVec::new(1, 32);
        let mut vec = conc_vec.clone().get_appender();
        let n = 1_000_000;

        for i in 0..n {
            vec.push(i);
        }

        let mut set = HashSet::new();
        for i in conc_vec.lock_iter() {
            println!("{}", i);
            set.insert(i);
        }
    }

    #[test]
    fn multithreading() {
        let conc_vec = ConcVec::new(1, 32);
        let n = 100_000;

        let n_threads = 16;

        let mut handles = vec![];

        for t in 0..n_threads {
            let mut conc_vec = conc_vec.clone().get_appender();
            handles.push(thread::spawn(move || {
                for i in 0..n {
                    if i % n_threads == t {
                        conc_vec.push(i);
                    }
                }
            }))
        }

        for h in handles {
            h.join().unwrap();
        }

        let mut set = HashSet::new();
        for i in conc_vec.lock_iter() {
            println!("{}", i);
            set.insert(i);
        }
    }
}
