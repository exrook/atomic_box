use ::AtomicBox;
use std::mem;
use std::thread;
use std::sync::{Barrier,Arc};
use std::sync::atomic::{AtomicBool,Ordering};
use std::time::Duration;

#[test]
fn test_new() {
    let b = AtomicBox::new(vec![0; 5]);
    mem::forget(b); // forget b so we don't run the destructor
}

#[test]
fn test_load() {
    let b = AtomicBox::new(vec![0; 5]);
    assert!(*b.load() == vec![0; 5]);
    mem::forget(b); // forget b so we don't run the destructor
}

#[test]
fn test_swap() {
    let b = AtomicBox::new(vec![0; 5]);
    assert!(**b.swap(vec![1; 5]) == vec![0; 5]);
    assert!(*b.load() == vec![1; 5]);
    mem::forget(b); // forget b so we don't run the destructor
}

#[test]
fn test_destructor() {
    let b = AtomicBox::new(vec![0; 5]);
    mem::drop(b);
}

#[test]
fn test_swap_contention() {
    let b = Arc::new(AtomicBox::new(vec![0; 0]));
    let barrier = Arc::new(Barrier::new(3));
    let b1 = b.clone();
    let barrier1 = barrier.clone();
    let t1 = thread::spawn(move || {
        barrier1.wait();
        thread::sleep(Duration::from_millis(4));
        for i in 0..32 {
            b1.swap(vec![i; 5]);
            thread::sleep(Duration::from_millis(10));
        }
        barrier1.wait();
    });
    let b2 = b.clone();
    let barrier2 = barrier.clone();
    let t2 = thread::spawn(move || {
        barrier2.wait();
        for i in 0..32 {
            b2.swap(vec![i; 10]);
            thread::sleep(Duration::from_millis(10));
        }
        barrier2.wait();
    });
    thread::spawn(move || {
        barrier.wait();
        thread::spawn(move || {
            barrier.wait();
        });
        loop {
            println!("{:?}", b.load());
            thread::sleep(Duration::from_millis(5));
        }
    });
    t1.join().unwrap();
    t2.join().unwrap();
}

#[test]
fn test_contention2() {
    let num_threads = 100;
    let mut v = vec![42usize; 1000];
    v[0] = 0;
    let b = Arc::new(AtomicBox::new(v));
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut t = vec!();
    for i in 0..num_threads {
        let b = b.clone();
        let barrier = barrier.clone();
        t.push(thread::spawn(move || {
            barrier.wait();
            let mut v = (*(b.load())).clone();
            for i in 0..1024 {
                for (n,x) in v.iter_mut().enumerate() {
                    *x = (*x).wrapping_add(n+1)
                }
                v = (**(b.swap(v))).clone();
            }
            v
        }));
    }
    let finished = Arc::new(AtomicBool::new(false));
    let print_thread = {
        let finished = finished.clone();
        thread::spawn(move || {
            while !finished.load(Ordering::Acquire) {
                println!("b: {:?}", b.load()[0]);
                thread::sleep(Duration::from_millis(16));
            }
        })
    };
    let mut v: Vec<_> = t.into_iter().map(|t|t.join().unwrap()).enumerate().collect();
    finished.store(true, Ordering::Release);
    v.sort_by_key(|&(_, ref v)| { v[0] });
    for (n,v) in v {
        println!("Thread #{}: {:?}", n,v[0]);
    }
}
