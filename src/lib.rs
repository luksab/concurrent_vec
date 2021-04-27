#![feature(min_const_generics)]

//! Append-only concurrent vector-like datastructure

use std::{cell::Cell, sync::{Arc, Mutex, MutexGuard}};

/// A concurrent vector, only supporting push and indexed access
pub struct ConcVec<T, const BUF_SIZE: usize> {
    data: Mutex<Option<Vec<Vec<T>>>>,
}

unsafe impl<T, const BUF_SIZE: usize> Sync for ConcVec<T, BUF_SIZE> {}
unsafe impl<T, const BUF_SIZE: usize> Send for ConcVec<T, BUF_SIZE> {}

pub struct BufferVec<T, const BUF_SIZE: usize> {
    data: Cell<Vec<T>>,
    vec: Arc<ConcVec<T, BUF_SIZE>>,
}

impl<T, const BUF_SIZE: usize> Drop for BufferVec<T, BUF_SIZE> {
    fn drop(&mut self) {
        let new_vec: Cell<Vec<T>> = Cell::new(Vec::new());
        self.data.swap(&new_vec);
        self.vec.push(new_vec.into_inner());
    }
}

impl<T, const BUF_SIZE: usize> BufferVec<T, BUF_SIZE> {
    pub fn push(&mut self, t: T) {
        self.data.get_mut().push(t);
        if self.data.get_mut().len() == BUF_SIZE {
            let new_vec: Cell<Vec<T>> = Cell::new(Vec::new());
            self.data.swap(&new_vec);
            self.vec.push(new_vec.into_inner());
        }
    }
}

impl<'a, T, const BUF_SIZE: usize> ConcVec<T, BUF_SIZE> {
    /// Make a new Aoavec
    pub fn new() -> Self {
        ConcVec {
            data: Default::default(),
        }
    }

    /// Make a new Aoavec with an initial capacity
    pub fn with_capacity(size: usize) -> Self {
        ConcVec {
            data: Mutex::from(Some(Vec::with_capacity(size))),
        }
    }

    /// Returns the length of the `Aoavec`.
    pub fn len(&self) -> usize {
        self.data.lock().unwrap().as_ref().unwrap().len() * BUF_SIZE
    }

    pub fn get_appender(vec: Arc<Self>) -> BufferVec<T, BUF_SIZE> {
        BufferVec {
            data: Default::default(),
            vec,
        }
    }

    /// Push a vec to data
    pub(crate) fn push(&self, t: Vec<T>) {
        self.data.lock().unwrap().as_mut().unwrap().push(t)
    }

    pub fn into_iter(&self) -> VecIter<T, BUF_SIZE>{//impl Iterator<Item = T> + 'static {
        //self.data.lock().unwrap().into_iter().flatten()
        VecIter::<T, BUF_SIZE> {
            data: self.data.lock().unwrap().take().unwrap(),
            outer: Vec::new(),
        }
    }
}

pub struct VecIter<T, const BUF_SIZE: usize> {
    data: Vec<Vec<T>>,
    outer: Vec<T>,
}

impl<'a, T, const BUF_SIZE: usize> std::iter::Iterator for VecIter<T, BUF_SIZE> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.outer.is_empty(){
            self.outer = self.data.pop()?;
        }
        self.outer.pop()
        // match self.outer.(self.outer_index){
        //     Some(outer) => {
        //         match outer.get(self.inner_index) {
        //             Some(inner) => {

        //             }
        //             None => {None}
        //         }
        //     }
        //     None => {None}
        // }
    }
}

// impl<'a, T, const BUF_SIZE: usize> IntoIterator for VecIter<'a, T, BUF_SIZE> {
//     type Item = T;

//     type IntoIter = std::iter::Flatten<std::vec::IntoIter<std::vec::Vec<T>>>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.guard.into_iter().flatten()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::{collections::HashSet, thread};

    #[test]
    fn base_one() {
        let conc_vec = Arc::new(ConcVec::<_, 1>::with_capacity(1));
        let mut vec = ConcVec::get_appender(conc_vec.clone());
        let n = 1024;

        for i in 0..n {
            vec.push(i);
        }

        // for i in 0..n {
        //     assert_eq!(ConcVec[i], i);
        // }
    }

    #[test]
    fn base_thirtytwo() {
        let conc_vec = Arc::new(ConcVec::<_, 32>::with_capacity(32));
        let mut vec = ConcVec::get_appender(conc_vec.clone());
        let n = 1_000_000;

        for i in 0..n {
            vec.push(i);
        }

        // for i in 0..n {
        //     assert_eq!(ConcVec[i], i);
        //     assert_eq!(ConcVec.get(i), Some(&i));
        //     unsafe {
        //         assert_eq!(ConcVec.get_unchecked(i), &i);
        //     }
        // }
    }

    #[test]
    fn multithreading() {
        let conc_vec = Arc::new(ConcVec::<_, 32>::with_capacity(32));
        let n = 100_000;

        let n_threads = 16;

        let mut handles = vec![];

        for t in 0..n_threads {
            let mut conc_vec = ConcVec::get_appender(conc_vec.clone());
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

        // let mut set = HashSet::new();

        // for i in 0..n {
        //     set.insert(ConcVec[i]);
        // }

        // assert_eq!(set.len(), n);
    }
}
