// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::{Example, ExampleOptions};
use log::debug;
use std::time::Instant;
use winterfell::{
    math::{fields::f63::BaseElement, log2, FieldElement},
    ProofOptions, Prover, StarkProof, Trace, TraceTable, VerifierError,
};

#[allow(clippy::module_inception)]
pub mod rescue;

mod air;
use air::{PublicInputs, RescueAir};

mod prover;
use prover::RescueProver;

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

const CYCLE_LENGTH: usize = 8;
const NUM_HASH_ROUNDS: usize = 7;

// RESCUE HASH CHAIN EXAMPLE
// ================================================================================================

pub fn get_example(options: ExampleOptions, chain_length: usize) -> Box<dyn Example> {
    Box::new(RescueExample::new(
        chain_length,
        options.to_proof_options(56, 4),
    ))
}

pub struct RescueExample {
    options: ProofOptions,
    chain_length: usize,
    seed: [BaseElement; 7],
    result: [BaseElement; 7],
}

impl RescueExample {
    pub fn new(chain_length: usize, options: ProofOptions) -> RescueExample {
        assert!(
            chain_length.is_power_of_two(),
            "chain length must a power of 2"
        );
        let seed = [
            BaseElement::from(42u8),
            BaseElement::from(43u8),
            BaseElement::from(44u8),
            BaseElement::from(45u8),
            BaseElement::from(46u8),
            BaseElement::from(47u8),
            BaseElement::from(48u8),
        ];

        // compute the sequence of hashes using external implementation of Rescue hash
        let now = Instant::now();
        let result = compute_hash_chain(seed, chain_length);
        debug!(
            "Computed a chain of {} Rescue hashes in {} ms",
            chain_length,
            now.elapsed().as_millis(),
        );

        RescueExample {
            options,
            chain_length,
            seed,
            result,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for RescueExample {
    fn prove(&self) -> StarkProof {
        // generate the execution trace
        debug!(
            "Generating proof for computing a chain of {} Rescue hashes\n\
            ---------------------",
            self.chain_length
        );

        let prover = RescueProver::new(self.options.clone());

        // generate the execution trace
        let now = Instant::now();
        let trace = prover.build_trace(self.seed, self.chain_length);
        let trace_length = trace.length();
        debug!(
            "Generated execution trace of {} registers and 2^{} steps in {} ms",
            trace.width(),
            log2(trace_length),
            now.elapsed().as_millis()
        );

        // generate the proof
        prover.prove(trace).unwrap()
    }

    fn verify(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let pub_inputs = PublicInputs {
            seed: self.seed,
            result: self.result,
        };
        winterfell::verify::<RescueAir>(proof, pub_inputs)
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let pub_inputs = PublicInputs {
            seed: self.seed,
            result: [
                self.result[0],
                self.result[1],
                self.result[2],
                self.result[3],
                self.result[4],
                self.result[5],
                self.result[6] + BaseElement::ONE,
            ],
        };
        winterfell::verify::<RescueAir>(proof, pub_inputs)
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn compute_hash_chain(seed: [BaseElement; 7], length: usize) -> [BaseElement; 7] {
    let mut values = seed;
    let mut result = [BaseElement::ZERO; 7];
    for _ in 0..length {
        rescue::hash(values, &mut result);
        values.copy_from_slice(&result);
    }
    result
}
