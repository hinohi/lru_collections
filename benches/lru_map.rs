#![feature(test)]
extern crate test;
use test::Bencher;

use lru_collections::LruMap;


#[bench]
fn bench_get_from_e6(b: &mut Bencher) {
    let mut m = LruMap::new(1_000_000);
    for i in 0..1_000_000 {
        m.insert(i, i);
    }
    b.iter(move || {
        for i in (0..100).rev() {
            m.get(&i);
        }
        for i in 0..100 {
            m.get(&i);
        }
    });
}

#[bench]
fn bench_get_from_e4(b: &mut Bencher) {
    let mut m = LruMap::new(10000);
    for i in 0..10000 {
        m.insert(i, i);
    }
    b.iter(move || {
        for i in (0..100).rev() {
            m.get(&i);
        }
        for i in 0..100 {
            m.get(&i);
        }
    });
}

#[bench]
fn bench_get_from_e2(b: &mut Bencher) {
    let mut m = LruMap::new(100);
    for i in 0..100 {
        m.insert(i, i);
    }
    b.iter(move || {
        for i in (0..150).rev() {
            m.get(&i);
        }
        for i in 0..100 {
            m.get(&i);
        }
    });
}
