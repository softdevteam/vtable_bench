// Copyright (c) 2019 King's College London created by the Software Development Team
// <http://soft-dev.org/>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, or the UPL-1.0 license <http://opensource.org/licenses/UPL>
// at your option. This file may not be copied, modified, or distributed except according to those
// terms.

#![feature(alloc_layout_extra)]
#![feature(allocator_api)]

use std::{
    alloc::{alloc, Layout},
    env,
    mem::{size_of, transmute},
    time::Instant
};

use lazy_static::lazy_static;

lazy_static! {
    static ref ITERS: usize = env::args().nth(1).unwrap().parse().unwrap();
    static ref VEC_SIZE: usize = env::args().nth(2).unwrap().parse().unwrap();
}
const VAL_NOREAD: usize = 0xdeadbeef;
const VAL_WITHREAD: usize = 0xFEEDC0DE;

#[inline(never)]
fn time<F>(mut f: F)
where
    F: FnMut()
{
    let before = Instant::now();
    for _ in 0..*ITERS {
        f();
    }
    let d = Instant::now() - before;
    println!("{:?}", d.as_secs() as f64 + d.subsec_nanos() as f64 * 1e-9);
}

// The trait whose function we'll dynamically look up.
trait GetVal {
    fn val(&self) -> usize;
}

// This is a simple helper trait that lets us cut a lot of our code in half below: it allows us to
// instantiate SNoRead or SWithRead via a trait. We can't put this in GetVal because GetVal then
// wouldn't be a valid trait object.
trait New {
    fn new() -> Self;
}

// SNoRead and SWithRead are the same size, but SNoRead.val() doesn't actually read the "_i"
// attribute (hence the leading underscore) whereas SWithRead does read from its "i" attribute
// (hence the lack of a leading underscore).
struct SNoRead {
    _i: usize
}

impl GetVal for SNoRead {
    fn val(&self) -> usize {
        VAL_NOREAD
    }
}

impl New for SNoRead {
    fn new() -> Self {
        SNoRead { _i: VAL_NOREAD }
    }
}

struct SWithRead {
    i: usize
}

impl GetVal for SWithRead {
    fn val(&self) -> usize {
        self.i
    }
}

impl New for SWithRead {
    fn new() -> Self {
        SWithRead { i: VAL_WITHREAD }
    }
}

fn vec_normal<S: 'static + New + GetVal>() -> Vec<Box<dyn GetVal>> {
    let mut v = Vec::<Box<dyn GetVal>>::with_capacity(*VEC_SIZE);
    for _ in 0..*VEC_SIZE {
        v.push(Box::new(S::new()));
    }
    v
}

pub fn bench_fat_no_read() {
    assert_eq!(size_of::<Box<()>>(), size_of::<usize>());
    assert_eq!(size_of::<Box<dyn GetVal>>(), size_of::<usize>() * 2);
    let v = vec_normal::<SNoRead>();
    time(|| {
        for e in &v {
            assert_eq!(e.val(), VAL_NOREAD);
        }
    });
}

pub fn bench_fat_with_read() {
    assert_eq!(size_of::<Box<()>>(), size_of::<usize>());
    assert_eq!(size_of::<Box<dyn GetVal>>(), size_of::<usize>() * 2);
    let v = vec_normal::<SWithRead>();
    time(|| {
        for e in &v {
            assert_eq!(e.val(), VAL_WITHREAD);
        }
    });
}

fn vec_multiref<S: 'static + New + GetVal>() -> Vec<*mut dyn GetVal> {
    vec![Box::into_raw(Box::new(S::new())); *VEC_SIZE]
}

fn clean_vec_multiref(v: Vec<*mut dyn GetVal>) {
    unsafe {
        Box::from_raw(v[0]);
    }
}

pub fn bench_fat_multiref_no_read() {
    assert_eq!(size_of::<Box<()>>(), size_of::<usize>());
    assert_eq!(size_of::<Box<dyn GetVal>>(), size_of::<usize>() * 2);
    let v = vec_multiref::<SNoRead>();
    time(|| {
        for &e in &v {
            assert_eq!(unsafe { (&*e).val() }, VAL_NOREAD);
        }
    });
    clean_vec_multiref(v);
}

pub fn bench_fat_multiref_with_read() {
    assert_eq!(size_of::<Box<()>>(), size_of::<usize>());
    assert_eq!(size_of::<Box<dyn GetVal>>(), size_of::<usize>() * 2);
    let v = vec_multiref::<SWithRead>();
    time(|| {
        for &e in &v {
            assert_eq!(unsafe { (&*e).val() }, VAL_WITHREAD);
        }
    });
    clean_vec_multiref(v);
}

fn vec_vtable<S: 'static + New + GetVal>() -> Vec<*mut ()> {
    assert_eq!(size_of::<Box<()>>(), size_of::<usize>());
    assert_eq!(size_of::<Box<dyn GetVal>>(), size_of::<usize>() * 2);
    let mut v = Vec::with_capacity(*VEC_SIZE);
    // Since every instance of S will share the same vtable, it's OK for us to pull it out and
    // reuse it. With the coerce_unsized feature turned on, we can do this in a marginally cleverer
    // way, but the outcome is the same.
    let vtable = {
        let b: *const dyn GetVal = Box::into_raw(Box::new(S::new()));
        let (_, vtable) = unsafe { transmute::<_, (usize, usize)>(b) };
        vtable
    };
    // We're going to lay out memory (on a 64-bit machine) as:
    //   offset 0: vtable
    //          8: S
    let (layout, _) = Layout::new::<usize>().extend(Layout::new::<S>()).unwrap();
    // The following assert ensure that the layout really is as we expect.
    assert_eq!(layout.size(), size_of::<usize>() + size_of::<S>());
    for _ in 0..*VEC_SIZE {
        let b = unsafe {
            let b: *mut usize = alloc(layout) as *mut usize;
            b.copy_from(&vtable, 1);
            (b.add(1) as *mut S).copy_from(&S::new(), 1);
            b as *mut ()
        };
        v.push(b);
    }
    v
}

fn clean_vec_vtable(v: Vec<*mut ()>) {
    for e in v {
        unsafe {
            Box::from_raw(e);
        }
    }
}

pub fn bench_innervtable_no_read() {
    let v = vec_vtable::<SNoRead>();
    time(|| {
        for &e in &v {
            let vtable = unsafe { *(e as *const usize) };
            let t_ptr = unsafe { (e as *const usize).add(1) };
            let b: *const dyn GetVal = unsafe { transmute((t_ptr, vtable)) };
            assert_eq!(unsafe { (&*b).val() }, VAL_NOREAD);
        }
    });
    clean_vec_vtable(v);
}

pub fn bench_innervtable_with_read() {
    let v = vec_vtable::<SWithRead>();
    time(|| {
        for &e in &v {
            let vtable = unsafe { *(e as *const usize) };
            let t_ptr = unsafe { (e as *const usize).add(1) };
            let b: *const dyn GetVal = unsafe { transmute((t_ptr, vtable)) };
            assert_eq!(unsafe { (&*b).val() }, VAL_WITHREAD);
        }
    });
    clean_vec_vtable(v);
}

fn vec_multiref_vtable<S: 'static + New + GetVal>() -> Vec<*mut ()> {
    let mut v = Vec::with_capacity(*VEC_SIZE);
    let vtable = {
        let b: *const dyn GetVal = Box::into_raw(Box::new(S::new()));
        let (_, vtable) = unsafe { transmute::<_, (usize, usize)>(b) };
        vtable
    };
    let ptr = {
        let (layout, _) = Layout::new::<usize>().extend(Layout::new::<S>()).unwrap();
        assert_eq!(layout.size(), size_of::<usize>() + size_of::<S>());
        let b = unsafe {
            let b: *mut usize = alloc(layout) as *mut usize;
            b.copy_from(&vtable, 1);
            (b.add(1) as *mut S).copy_from(&S::new(), 1);
            b as *mut S as *mut dyn GetVal
        };
        let (ptr, _) = unsafe { transmute::<_, (*mut (), usize)>(b) };
        ptr
    };
    for _ in 0..*VEC_SIZE {
        v.push(ptr);
    }
    v
}

fn clean_multiref_table(v: Vec<*mut ()>) {
    unsafe {
        Box::from_raw(v[0]);
    }
}

pub fn bench_innervtable_multiref_no_read() {
    let v = vec_multiref_vtable::<SNoRead>();
    time(|| {
        for &e in &v {
            let vtable = unsafe { *(e as *const usize) };
            let t_ptr = unsafe { (e as *const usize).add(1) };
            let b: *const dyn GetVal = unsafe { transmute((t_ptr, vtable)) };
            assert_eq!(unsafe { (&*b).val() }, VAL_NOREAD);
        }
    });
    clean_multiref_table(v);
}

pub fn bench_innervtable_multiref_with_read() {
    let v = vec_multiref_vtable::<SWithRead>();
    time(|| {
        for &e in &v {
            let vtable = unsafe { *(e as *const usize) };
            let t_ptr = unsafe { (e as *const usize).add(1) };
            let b: *const dyn GetVal = unsafe { transmute((t_ptr, vtable)) };
            assert_eq!(unsafe { (&*b).val() }, VAL_WITHREAD);
        }
    });
    clean_multiref_table(v);
}
