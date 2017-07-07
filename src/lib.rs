// Copyright 2017 Jacob Hughes
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::fmt;

/// A struct which allows multiple threads to safely update and read from a shared non-nullable pointer to heap data
pub struct AtomicBox<T> {
    ptr: AtomicPtr<Arc<T>>,
}

impl<T> AtomicBox<T> {
    /// Create a new atomic box from `data`
    pub fn new(data: T) -> AtomicBox<T> {
        let arc = Box::new(Arc::new(data));
        let ptr = AtomicPtr::new(Box::into_raw(arc));
        return AtomicBox { ptr: ptr };
    }
    /// Get an `Arc<T>` pointing to the value currently stored in the box
    pub fn load(&self) -> Arc<T> {
        let arc = unsafe { &mut *self.ptr.load(Ordering::Acquire) };
        return arc.clone();
    }
    /// Update the value stored in the box to point to `data`, the previous data will be reclaimed
    /// once the last reader has dropped their reference.
    pub fn store(&self, data: T) {
        let arc = Box::new(Arc::new(data));
        let ptr = Box::into_raw(arc);
        let old_ptr = self.ptr.swap(ptr, Ordering::AcqRel);
        unsafe { Box::from_raw(old_ptr) };
    }
    /// Swap the current value with `data`, returning a `boxed` `Arc<T>` pointing to the old value
    pub fn swap(&self, data: T) -> Box<Arc<T>> {
        let arc = Box::new(Arc::new(data));
        let ptr = Box::into_raw(arc);
        let old_ptr = self.ptr.swap(ptr, Ordering::AcqRel);
        return unsafe { Box::from_raw(old_ptr) };
    }
}

impl<T> Drop for AtomicBox<T> {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.ptr.load(Ordering::Acquire)) };
    }
}

impl<T> fmt::Debug for AtomicBox<T>
    where T: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AtomicBox")
            .field("ptr", &self.load())
            .finish()
    }
}
