#![feature(test)]

extern crate test;
use test::Bencher;

#[bench]
fn bench_xor_1000_ints(b: &mut Bencher) {
    b.iter(|| {
        let n = test::black_box(10000);

        (0..n).fold(0, |a, b| a ^ b)
    })
}
