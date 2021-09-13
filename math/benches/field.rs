// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand_utils::{rand_value, rand_vector};
use std::{convert::TryInto, time::Duration};
use utils::AsBytes;
use winter_math::{
    batch_inversion,
    fields::{f128, f252, f62, QuadExtensionA},
    FieldElement, StarkField,
};

const SIZES: [usize; 3] = [262_144, 524_288, 1_048_576];

pub fn f128_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("f128");

    group.bench_function("add", |bench| {
        let x = rand_value::<f128::BaseElement>();
        let y = rand_value::<f128::BaseElement>();
        bench.iter(|| black_box(x) + black_box(y))
    });

    group.bench_function("sub", |bench| {
        let x = rand_value::<f128::BaseElement>();
        let y = rand_value::<f128::BaseElement>();
        bench.iter(|| black_box(x) - black_box(y))
    });

    group.bench_function("mul", |bench| {
        let x = rand_value::<f128::BaseElement>();
        let y = rand_value::<f128::BaseElement>();
        bench.iter(|| black_box(x) * black_box(y))
    });

    group.bench_function("exp", |bench| {
        let x = rand_value::<f128::BaseElement>();
        let y = u128::from_le_bytes(
            rand_value::<f128::BaseElement>()
                .as_bytes()
                .try_into()
                .unwrap(),
        );
        bench.iter(|| f128::BaseElement::exp(black_box(x), black_box(y)))
    });

    group.bench_function("inv", |bench| {
        let x = rand_value::<f128::BaseElement>();
        bench.iter(|| f128::BaseElement::inv(black_box(x)))
    });
}

pub fn f128_extension_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("f128_quad");

    group.bench_function("mul", |bench| {
        let x = rand_value::<QuadExtensionA<f128::BaseElement>>();
        let y = rand_value::<QuadExtensionA<f128::BaseElement>>();
        bench.iter(|| black_box(x) * black_box(y))
    });
}

pub fn f62_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("f62");

    group.bench_function("add", |bench| {
        let x = rand_value::<f62::BaseElement>();
        let y = rand_value::<f62::BaseElement>();
        bench.iter(|| black_box(x) + black_box(y))
    });

    group.bench_function("sub", |bench| {
        let x = rand_value::<f62::BaseElement>();
        let y = rand_value::<f62::BaseElement>();
        bench.iter(|| black_box(x) - black_box(y))
    });

    group.bench_function("mul", |bench| {
        let x = rand_value::<f62::BaseElement>();
        let y = rand_value::<f62::BaseElement>();
        bench.iter(|| black_box(x) * black_box(y))
    });

    group.bench_function("exp", |bench| {
        let x = rand_value::<f62::BaseElement>();
        let y = u64::from_le_bytes(
            rand_value::<f62::BaseElement>()
                .as_bytes()
                .try_into()
                .unwrap(),
        );
        bench.iter(|| f62::BaseElement::exp(black_box(x), black_box(y)))
    });

    group.bench_function("inv", |bench| {
        let x = rand_value::<f62::BaseElement>();
        bench.iter(|| f62::BaseElement::inv(black_box(x)))
    });
}

pub fn f62_extension_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("f62_quad");

    group.bench_function("mul", |bench| {
        let x = rand_value::<QuadExtensionA<f62::BaseElement>>();
        let y = rand_value::<QuadExtensionA<f62::BaseElement>>();
        bench.iter(|| black_box(x) * black_box(y))
    });
}

pub fn f252_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("f252");

    group.bench_function("add", |bench| {
        let x = rand_value::<f252::BaseElement>();
        let y = rand_value::<f252::BaseElement>();
        bench.iter(|| black_box(x) + black_box(y))
    });

    group.bench_function("sub", |bench| {
        let x = rand_value::<f252::BaseElement>();
        let y = rand_value::<f252::BaseElement>();
        bench.iter(|| black_box(x) - black_box(y))
    });

    group.bench_function("mul", |bench| {
        let x = rand_value::<f252::BaseElement>();
        let y = rand_value::<f252::BaseElement>();
        bench.iter(|| black_box(x) * black_box(y))
    });

    group.bench_function("square", |bench| {
        let x = rand_value::<f252::BaseElement>();
        bench.iter(|| black_box(x).square())
    });

    group.bench_function("square_from_mul", |bench| {
        let x = rand_value::<f252::BaseElement>();
        bench.iter(|| black_box(x) * black_box(x))
    });

    group.bench_function("exp", |bench| {
        let x = rand_value::<f252::BaseElement>();
        let y = rand_value::<f252::BaseElement>().to_repr();
        bench.iter(|| f252::BaseElement::exp(black_box(x), black_box(y)))
    });

    group.bench_function("inv", |bench| {
        let x = rand_value::<f252::BaseElement>();
        bench.iter(|| f252::BaseElement::inv(black_box(x)))
    });
}

pub fn batch_inv(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_inv");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    for &size in SIZES.iter() {
        let values = rand_vector::<f128::BaseElement>(size);

        group.bench_function(BenchmarkId::new("no_coeff", size), |bench| {
            bench.iter_with_large_drop(|| batch_inversion(&values));
        });
    }

    group.finish();
}

criterion_group!(
    field_group,
    batch_inv,
    f252_ops,
    f128_ops,
    f128_extension_ops,
    f62_ops,
    f62_extension_ops
);
criterion_main!(field_group);
