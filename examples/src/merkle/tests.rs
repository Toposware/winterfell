// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winterfell::{FieldExtension, HashFunction, ProofOptions};

#[test]
fn merkle_test_basic_proof_verification() {
    let merkle = Box::new(super::MerkleExample::new(7, build_options(false)));
    crate::tests::test_basic_proof_verification(merkle);
}

#[test]
fn merkle_test_basic_proof_verification_extension() {
    let merkle = Box::new(super::MerkleExample::new(7, build_options(true)));
    crate::tests::test_basic_proof_verification(merkle);
}

#[test]
fn merkle_test_basic_proof_verification_fail() {
    let merkle = Box::new(super::MerkleExample::new(7, build_options(false)));
    crate::tests::test_basic_proof_verification_fail(merkle);
}

fn build_options(use_extension_field: bool) -> ProofOptions {
    let extension = if use_extension_field {
        FieldExtension::Quadratic
    } else {
        FieldExtension::None
    };
    ProofOptions::new(28, 8, 0, HashFunction::Blake3_256, extension, 4, 256)
}
