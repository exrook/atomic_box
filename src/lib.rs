// Copyright 2017 Jacob Hughes
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use std::fmt;

#[cfg(test)]
mod test;

/// A struct which allows multiple threads to safely update and read from a shared non-nullable pointer to heap data
pub struct AtomicBox<T> {
    ptr: AtomicPtr<Arc<T>>,
}

impl<T> AtomicBox<T> {
    /// Create a new atomic box from `data`
    pub fn new(data: T) -> AtomicBox<T> {
        let arc = Box::new(Arc::new(data));
        let ptr = AtomicPtr::new(Box::into_raw(arc));
        AtomicBox { ptr: ptr }
    }
    /// Get an `Arc<T>` pointing to the value currently stored in the box
    pub fn load(&self) -> Arc<T> {
        loop {
            // Poison the ptr so that it won't be dropped before we call Clone()
            let ptr = self.ptr.swap(ptr::null_mut(), Ordering::Acquire);
            if ptr == ptr::null_mut() {
                continue;
            }
            let arc = unsafe { &mut *ptr }.clone();
            // Unpoison the ptr
            self.ptr.swap(ptr, Ordering::AcqRel);
            return arc;
        }
    }
    /// Update the value stored in the box to point to `data`, the previous data will be reclaimed
    /// once the last reader has dropped their reference.
    pub fn store(&self, data: T) {
        self.swap(data);
    }
    /// Swap the current value with `data`, returning a `boxed` `Arc<T>` pointing to the old value
    pub fn swap(&self, data: T) -> Box<Arc<T>> {
        let arc = Box::new(Arc::new(data));
        let ptr = Box::into_raw(arc);
        loop {
            // Poison the ptr so that it won't be dropped before being cloned
            let old_ptr = self.ptr.swap(ptr::null_mut(), Ordering::AcqRel);
            if old_ptr == ptr::null_mut() {
                continue;
            }
            let old_arc = unsafe { Box::from_raw(old_ptr) };
            // Unpoison the ptr
            self.ptr.swap(ptr, Ordering::AcqRel);
            return old_arc;
        }
    }
}

impl<T> Drop for AtomicBox<T> {
    fn drop(&mut self) {
        // let bound ensures that we are unboxing the right ptr
        let _: Box<Arc<T>> = unsafe { Box::from_raw(*self.ptr.get_mut()) };
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
