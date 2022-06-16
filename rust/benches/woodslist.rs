use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::read_to_string;
use wood::parse_multiline_woodslist;

fn criterion_benchmark(c: &mut Criterion){
    let big_curvy = read_to_string("big curvy.sli").unwrap();
    c.bench_function("parsing big curvy", |b| b.iter(||{
        parse_multiline_woodslist(black_box(big_curvy.as_str())).unwrap()
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);