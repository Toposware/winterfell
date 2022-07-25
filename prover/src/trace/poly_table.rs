// Copyright (c) Facebook, Inc. and its affiliates.
// Copyright (c) 2021-2022 Toposware, Inc.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::{
    matrix::{ColumnIter, MultiColumnIter},
    Matrix,
};
use math::{log2, FieldElement, StarkField, get_power_series_with_offset};
use utils::collections::Vec;

// TRACE POLYNOMIAL TABLE
// ================================================================================================

/// Trace polynomials in coefficient from for all segments of the execution trace.
///
/// Coefficients of the polynomials for the main trace segment are always in the base field.
/// However, coefficients of the polynomials for the auxiliary trace segments may be either in the
/// base field, or in the extension field, depending on whether extension field is being used.
pub struct TracePolyTable<E: FieldElement> {
    main_segment_polys: Matrix<E::BaseField>,
    aux_segment_polys: Vec<Matrix<E>>,
}

impl<E: FieldElement> TracePolyTable<E> {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Creates a new table of trace polynomials from the provided main trace segment polynomials.
    pub fn new(main_trace_polys: Matrix<E::BaseField>) -> Self {
        Self {
            main_segment_polys: main_trace_polys,
            aux_segment_polys: Vec::new(),
        }
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Adds the provided auxiliary segment polynomials to this polynomial table.
    pub fn add_aux_segment(&mut self, aux_segment_polys: Matrix<E>) {
        assert_eq!(
            self.main_segment_polys.num_rows(),
            aux_segment_polys.num_rows(),
            "polynomials in auxiliary segment must be of the same size as in the main segment"
        );
        self.aux_segment_polys.push(aux_segment_polys);
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the size of each polynomial - i.e. size of a vector needed to hold a polynomial.
    pub fn poly_size(&self) -> usize {
        self.main_segment_polys.num_rows()
    }

    /// Evaluates all trace polynomials (across all trace segments) at the specified point `x`.
    pub fn evaluate_at(&self, x: E) -> Vec<E> {
        let mut result = self.main_segment_polys.evaluate_columns_at(x);
        for aux_polys in self.aux_segment_polys.iter() {
            result.append(&mut aux_polys.evaluate_columns_at(x));
        }
        result
    }

    /// Returns an out-of-domain evaluation frame constructed by evaluating trace polynomials
    /// for all columns at points z and z * g, .., z*g^{max_pow-1}, where g is the generator of the trace domain.
    pub fn get_ood_frame(&self, z: E, max_pow: usize, ratio: usize) -> Vec<Vec<E>> {
        let g = E::from(E::BaseField::get_root_of_unity(log2(self.poly_size())));
        let current  = get_power_series_with_offset(g, z, max_pow)
        .into_iter()
        .flat_map(|i| self.evaluate_at(i))
        .collect();
        let next = self.evaluate_at(z * g.exp((ratio as u64).into()));
        vec![
            current,
            next
        ]
    }

    /// Returns an iterator over the polynomials of the main trace segment.
    pub fn main_trace_polys(&self) -> ColumnIter<E::BaseField> {
        self.main_segment_polys.columns()
    }

    /// Returns an iterator over the polynomials of all auxiliary trace segments.
    pub fn aux_trace_polys(&self) -> MultiColumnIter<E> {
        MultiColumnIter::new(self.aux_segment_polys.as_slice())
    }

    // TEST HELPERS
    // --------------------------------------------------------------------------------------------

    /// Returns the number of polynomials in the main segment of the trace.
    #[cfg(test)]
    pub fn num_main_trace_polys(&self) -> usize {
        self.main_segment_polys.num_cols()
    }

    /// Returns a polynomial from the main segment of the trace at the specified index.
    #[cfg(test)]
    pub fn get_main_trace_poly(&self, idx: usize) -> &[E::BaseField] {
        &self.main_segment_polys.get_column(idx)
    }
}
