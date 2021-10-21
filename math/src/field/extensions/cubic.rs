// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use crate::fields::{
    f128::BaseElement as BaseElement128, f62::BaseElement as BaseElement62,
    f64::BaseElement as BaseElement64,
};

use super::{ExtensibleField, FieldElement};
use core::{
    convert::TryFrom,
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    slice,
};
use utils::{
    collections::Vec, string::ToString, AsBytes, ByteReader, ByteWriter, Deserializable,
    DeserializationError, Randomizable, Serializable, SliceReader,
};

// QUADRATIC EXTENSION FIELD
// ================================================================================================

/// Represents an element in a cubic extension of a [StarkField](crate::StarkField).
///
/// The extension element is defined as α + β * φ + γ * φ^2, where φ is a root of in irreducible
/// polynomial defined by the implementation of the [ExtensibleField] trait, and α, β, γ are base
/// field elements.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct CubeExtension<B: ExtensibleField<3>>(B, B, B);

impl<B: ExtensibleField<3>> CubeExtension<B> {
    /// Returns a new extension element instantiated from the provided base elements.
    pub fn new(a: B, b: B, c: B) -> Self {
        Self(a, b, c)
    }

    /// Converts a vector of base elements into a vector of elements in a cubic extension field
    /// by fusing three adjacent base elements together. The output vector is half the length of
    /// the source vector.
    fn base_to_quad_vector(source: Vec<B>) -> Vec<Self> {
        debug_assert!(
            source.len() % 3 == 0,
            "source vector length must be divisible by three, but was {}",
            source.len()
        );
        let mut v = core::mem::ManuallyDrop::new(source);
        let p = v.as_mut_ptr();
        let len = v.len() / 3;
        let cap = v.capacity() / 3;
        unsafe { Vec::from_raw_parts(p as *mut Self, len, cap) }
    }
}

impl FieldElement for CubeExtension<BaseElement62> {
    type Representation = <BaseElement62 as FieldElement>::Representation;
    type BaseField = BaseElement62;

    const ELEMENT_BYTES: usize = BaseElement62::ELEMENT_BYTES * 3;
    const IS_CANONICAL: bool = BaseElement62::IS_CANONICAL;
    const ZERO: Self = Self(
        BaseElement62::ZERO,
        BaseElement62::ZERO,
        BaseElement62::ZERO,
    );
    const ONE: Self = Self(BaseElement62::ONE, BaseElement62::ZERO, BaseElement62::ZERO);

    fn exp(self, power: Self::Representation) -> Self {
        let mut r = Self::ONE;
        let mut b = self;
        let mut p = power;

        let int_zero = Self::Representation::from(0u32);
        let int_one = Self::Representation::from(1u32);

        if p == int_zero {
            return Self::ONE;
        } else if b == Self::ZERO {
            return Self::ZERO;
        }

        while p > int_zero {
            if p & int_one == int_one {
                r *= b;
            }
            p >>= int_one;
            b = b.square();
        }

        r
    }

    #[inline]
    fn inv(self) -> Self {
        if self == Self::ZERO {
            return self;
        }

        let x = [self.0, self.1, self.2];
        let c1 = <BaseElement62 as ExtensibleField<3>>::frobenius(x);
        let c2 = <BaseElement62 as ExtensibleField<3>>::frobenius(c1);
        let numerator = <BaseElement62 as ExtensibleField<3>>::mul(c1, c2);

        let norm = <BaseElement62 as ExtensibleField<3>>::mul(x, numerator);
        debug_assert_eq!(
            norm[1],
            BaseElement62::ZERO,
            "norm must be in the base field"
        );
        debug_assert_eq!(
            norm[2],
            BaseElement62::ZERO,
            "norm must be in the base field"
        );
        let denom_inv = norm[0].inv();

        Self(
            numerator[0] * denom_inv,
            numerator[1] * denom_inv,
            numerator[2] * denom_inv,
        )
    }

    #[inline]
    fn conjugate(&self) -> Self {
        let result = <BaseElement62 as ExtensibleField<3>>::frobenius([self.0, self.1, self.2]);
        Self(result[0], result[1], result[2])
    }

    fn elements_as_bytes(elements: &[Self]) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                elements.as_ptr() as *const u8,
                elements.len() * Self::ELEMENT_BYTES,
            )
        }
    }

    unsafe fn bytes_as_elements(bytes: &[u8]) -> Result<&[Self], DeserializationError> {
        if bytes.len() % Self::ELEMENT_BYTES != 0 {
            return Err(DeserializationError::InvalidValue(format!(
                "number of bytes ({}) does not divide into whole number of field elements",
                bytes.len(),
            )));
        }

        let p = bytes.as_ptr();
        let len = bytes.len() / Self::ELEMENT_BYTES;

        // make sure the bytes are aligned on the boundary consistent with base element alignment
        if (p as usize) % Self::BaseField::ELEMENT_BYTES != 0 {
            return Err(DeserializationError::InvalidValue(
                "slice memory alignment is not valid for this field element type".to_string(),
            ));
        }

        Ok(slice::from_raw_parts(p as *const Self, len))
    }

    fn zeroed_vector(n: usize) -> Vec<Self> {
        // get twice the number of base elements, and re-interpret them as quad field elements
        let result = BaseElement62::zeroed_vector(n * 2);
        Self::base_to_quad_vector(result)
    }

    fn as_base_elements(elements: &[Self]) -> &[Self::BaseField] {
        let ptr = elements.as_ptr();
        let len = elements.len() * 3;
        unsafe { slice::from_raw_parts(ptr as *const Self::BaseField, len) }
    }

    fn normalize(&mut self) {
        self.0.normalize();
        self.1.normalize();
        self.2.normalize();
    }
}

impl FieldElement for CubeExtension<BaseElement64> {
    type Representation = <BaseElement64 as FieldElement>::Representation;
    type BaseField = BaseElement64;

    const ELEMENT_BYTES: usize = BaseElement64::ELEMENT_BYTES * 3;
    const IS_CANONICAL: bool = BaseElement64::IS_CANONICAL;
    const ZERO: Self = Self(
        BaseElement64::ZERO,
        BaseElement64::ZERO,
        BaseElement64::ZERO,
    );
    const ONE: Self = Self(BaseElement64::ONE, BaseElement64::ZERO, BaseElement64::ZERO);

    fn exp(self, power: Self::Representation) -> Self {
        let mut r = Self::ONE;
        let mut b = self;
        let mut p = power;

        let int_zero = Self::Representation::from(0u32);
        let int_one = Self::Representation::from(1u32);

        if p == int_zero {
            return Self::ONE;
        } else if b == Self::ZERO {
            return Self::ZERO;
        }

        while p > int_zero {
            if p & int_one == int_one {
                r *= b;
            }
            p >>= int_one;
            b = b.square();
        }

        r
    }

    #[inline]
    fn inv(self) -> Self {
        if self == Self::ZERO {
            return self;
        }

        let x = [self.0, self.1, self.2];
        let c1 = <BaseElement64 as ExtensibleField<3>>::frobenius(x);
        let c2 = <BaseElement64 as ExtensibleField<3>>::frobenius(c1);
        let numerator = <BaseElement64 as ExtensibleField<3>>::mul(c1, c2);

        let norm = <BaseElement64 as ExtensibleField<3>>::mul(x, numerator);
        debug_assert_eq!(
            norm[1],
            BaseElement64::ZERO,
            "norm must be in the base field"
        );
        debug_assert_eq!(
            norm[2],
            BaseElement64::ZERO,
            "norm must be in the base field"
        );
        let denom_inv = norm[0].inv();

        Self(
            numerator[0] * denom_inv,
            numerator[1] * denom_inv,
            numerator[2] * denom_inv,
        )
    }

    #[inline]
    fn conjugate(&self) -> Self {
        let result = <BaseElement64 as ExtensibleField<3>>::frobenius([self.0, self.1, self.2]);
        Self(result[0], result[1], result[2])
    }

    fn elements_as_bytes(elements: &[Self]) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                elements.as_ptr() as *const u8,
                elements.len() * Self::ELEMENT_BYTES,
            )
        }
    }

    unsafe fn bytes_as_elements(bytes: &[u8]) -> Result<&[Self], DeserializationError> {
        if bytes.len() % Self::ELEMENT_BYTES != 0 {
            return Err(DeserializationError::InvalidValue(format!(
                "number of bytes ({}) does not divide into whole number of field elements",
                bytes.len(),
            )));
        }

        let p = bytes.as_ptr();
        let len = bytes.len() / Self::ELEMENT_BYTES;

        // make sure the bytes are aligned on the boundary consistent with base element alignment
        if (p as usize) % Self::BaseField::ELEMENT_BYTES != 0 {
            return Err(DeserializationError::InvalidValue(
                "slice memory alignment is not valid for this field element type".to_string(),
            ));
        }

        Ok(slice::from_raw_parts(p as *const Self, len))
    }

    fn zeroed_vector(n: usize) -> Vec<Self> {
        // get twice the number of base elements, and re-interpret them as quad field elements
        let result = BaseElement64::zeroed_vector(n * 2);
        Self::base_to_quad_vector(result)
    }

    fn as_base_elements(elements: &[Self]) -> &[Self::BaseField] {
        let ptr = elements.as_ptr();
        let len = elements.len() * 3;
        unsafe { slice::from_raw_parts(ptr as *const Self::BaseField, len) }
    }

    fn normalize(&mut self) {
        self.0.normalize();
        self.1.normalize();
        self.2.normalize();
    }
}

impl FieldElement for CubeExtension<BaseElement128> {
    type Representation = <BaseElement128 as FieldElement>::Representation;
    type BaseField = BaseElement128;

    const ELEMENT_BYTES: usize = BaseElement128::ELEMENT_BYTES * 3;
    const IS_CANONICAL: bool = BaseElement128::IS_CANONICAL;
    const ZERO: Self = Self(
        BaseElement128::ZERO,
        BaseElement128::ZERO,
        BaseElement128::ZERO,
    );
    const ONE: Self = Self(
        BaseElement128::ONE,
        BaseElement128::ZERO,
        BaseElement128::ZERO,
    );

    fn exp(self, power: Self::Representation) -> Self {
        let mut r = Self::ONE;
        let mut b = self;
        let mut p = power;

        let int_zero = Self::Representation::from(0u32);
        let int_one = Self::Representation::from(1u32);

        if p == int_zero {
            return Self::ONE;
        } else if b == Self::ZERO {
            return Self::ZERO;
        }

        while p > int_zero {
            if p & int_one == int_one {
                r *= b;
            }
            p >>= int_one;
            b = b.square();
        }

        r
    }

    #[inline]
    fn inv(self) -> Self {
        if self == Self::ZERO {
            return self;
        }

        let x = [self.0, self.1, self.2];
        let c1 = <BaseElement128 as ExtensibleField<3>>::frobenius(x);
        let c2 = <BaseElement128 as ExtensibleField<3>>::frobenius(c1);
        let numerator = <BaseElement128 as ExtensibleField<3>>::mul(c1, c2);

        let norm = <BaseElement128 as ExtensibleField<3>>::mul(x, numerator);
        debug_assert_eq!(
            norm[1],
            BaseElement128::ZERO,
            "norm must be in the base field"
        );
        debug_assert_eq!(
            norm[2],
            BaseElement128::ZERO,
            "norm must be in the base field"
        );
        let denom_inv = norm[0].inv();

        Self(
            numerator[0] * denom_inv,
            numerator[1] * denom_inv,
            numerator[2] * denom_inv,
        )
    }

    #[inline]
    fn conjugate(&self) -> Self {
        let result = <BaseElement128 as ExtensibleField<3>>::frobenius([self.0, self.1, self.2]);
        Self(result[0], result[1], result[2])
    }

    fn elements_as_bytes(elements: &[Self]) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                elements.as_ptr() as *const u8,
                elements.len() * Self::ELEMENT_BYTES,
            )
        }
    }

    unsafe fn bytes_as_elements(bytes: &[u8]) -> Result<&[Self], DeserializationError> {
        if bytes.len() % Self::ELEMENT_BYTES != 0 {
            return Err(DeserializationError::InvalidValue(format!(
                "number of bytes ({}) does not divide into whole number of field elements",
                bytes.len(),
            )));
        }

        let p = bytes.as_ptr();
        let len = bytes.len() / Self::ELEMENT_BYTES;

        // make sure the bytes are aligned on the boundary consistent with base element alignment
        if (p as usize) % Self::BaseField::ELEMENT_BYTES != 0 {
            return Err(DeserializationError::InvalidValue(
                "slice memory alignment is not valid for this field element type".to_string(),
            ));
        }

        Ok(slice::from_raw_parts(p as *const Self, len))
    }

    fn zeroed_vector(n: usize) -> Vec<Self> {
        // get twice the number of base elements, and re-interpret them as quad field elements
        let result = BaseElement128::zeroed_vector(n * 2);
        Self::base_to_quad_vector(result)
    }

    fn as_base_elements(elements: &[Self]) -> &[Self::BaseField] {
        let ptr = elements.as_ptr();
        let len = elements.len() * 3;
        unsafe { slice::from_raw_parts(ptr as *const Self::BaseField, len) }
    }

    fn normalize(&mut self) {
        self.0.normalize();
        self.1.normalize();
        self.2.normalize();
    }
}

impl<B: ExtensibleField<3>> Randomizable for CubeExtension<B> {
    const VALUE_SIZE: usize = B::ELEMENT_BYTES * 3;

    fn from_random_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(bytes).ok()
    }
}

impl<B: ExtensibleField<3>> fmt::Display for CubeExtension<B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.0, self.1, self.2)
    }
}

// OVERLOADED OPERATORS
// ------------------------------------------------------------------------------------------------

impl<B: ExtensibleField<3>> Add for CubeExtension<B> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<B: ExtensibleField<3>> AddAssign for CubeExtension<B> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<B: ExtensibleField<3>> Sub for CubeExtension<B> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<B: ExtensibleField<3>> SubAssign for CubeExtension<B> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<B: ExtensibleField<3>> Mul for CubeExtension<B> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let result =
            <B as ExtensibleField<3>>::mul([self.0, self.1, self.2], [rhs.0, rhs.1, rhs.2]);
        Self(result[0], result[1], result[2])
    }
}

impl<B: ExtensibleField<3>> MulAssign for CubeExtension<B> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl Div for CubeExtension<BaseElement62> {
    type Output = Self;

    #[inline]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self {
        self * rhs.inv()
    }
}

impl DivAssign for CubeExtension<BaseElement62> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl Div for CubeExtension<BaseElement64> {
    type Output = Self;

    #[inline]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self {
        self * rhs.inv()
    }
}

impl DivAssign for CubeExtension<BaseElement64> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl Div for CubeExtension<BaseElement128> {
    type Output = Self;

    #[inline]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self {
        self * rhs.inv()
    }
}

impl DivAssign for CubeExtension<BaseElement128> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl<B: ExtensibleField<3>> Neg for CubeExtension<B> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self(-self.0, -self.1, -self.2)
    }
}

// TYPE CONVERSIONS
// ------------------------------------------------------------------------------------------------

impl<B: ExtensibleField<3>> From<B> for CubeExtension<B> {
    fn from(value: B) -> Self {
        Self(value, B::ZERO, B::ZERO)
    }
}

impl<B: ExtensibleField<3>> From<u128> for CubeExtension<B> {
    fn from(value: u128) -> Self {
        Self(B::from(value), B::ZERO, B::ZERO)
    }
}

impl<B: ExtensibleField<3>> From<u64> for CubeExtension<B> {
    fn from(value: u64) -> Self {
        Self(B::from(value), B::ZERO, B::ZERO)
    }
}

impl<B: ExtensibleField<3>> From<u32> for CubeExtension<B> {
    fn from(value: u32) -> Self {
        Self(B::from(value), B::ZERO, B::ZERO)
    }
}

impl<B: ExtensibleField<3>> From<u16> for CubeExtension<B> {
    fn from(value: u16) -> Self {
        Self(B::from(value), B::ZERO, B::ZERO)
    }
}

impl<B: ExtensibleField<3>> From<u8> for CubeExtension<B> {
    fn from(value: u8) -> Self {
        Self(B::from(value), B::ZERO, B::ZERO)
    }
}

impl<'a, B: ExtensibleField<3>> TryFrom<&'a [u8]> for CubeExtension<B> {
    type Error = DeserializationError;

    /// Converts a slice of bytes into a field element; returns error if the value encoded in bytes
    /// is not a valid field element. The bytes are assumed to be in little-endian byte order.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < B::ELEMENT_BYTES * 3 {
            return Err(DeserializationError::InvalidValue(format!(
                "not enough bytes for a full field element; expected {} bytes, but was {} bytes",
                B::ELEMENT_BYTES * 3,
                bytes.len(),
            )));
        }
        if bytes.len() > B::ELEMENT_BYTES * 3 {
            return Err(DeserializationError::InvalidValue(format!(
                "too many bytes for a field element; expected {} bytes, but was {} bytes",
                B::ELEMENT_BYTES * 3,
                bytes.len(),
            )));
        }
        let mut reader = SliceReader::new(bytes);
        Self::read_from(&mut reader)
    }
}

impl<B: ExtensibleField<3>> AsBytes for CubeExtension<B> {
    fn as_bytes(&self) -> &[u8] {
        // TODO: take endianness into account
        let self_ptr: *const Self = self;
        unsafe { slice::from_raw_parts(self_ptr as *const u8, B::ELEMENT_BYTES * 3) }
    }
}

// SERIALIZATION / DESERIALIZATION
// ------------------------------------------------------------------------------------------------

impl<B: ExtensibleField<3>> Serializable for CubeExtension<B> {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.0.write_into(target);
        self.1.write_into(target);
        self.2.write_into(target);
    }
}

impl<B: ExtensibleField<3>> Deserializable for CubeExtension<B> {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let value0 = B::read_from(source)?;
        let value1 = B::read_from(source)?;
        let value2 = B::read_from(source)?;
        Ok(Self(value0, value1, value2))
    }
}

/*
TODO: enable

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{DeserializationError, FieldElement, CubeExtension, Vec};
    use crate::field::f128::BaseElement;
    use rand_utils::rand_value;

    // BASIC ALGEBRA
    // --------------------------------------------------------------------------------------------

    #[test]
    fn add() {
        // identity
        let r: CubeExtension<BaseElement> = rand_value();
        assert_eq!(r, r + CubeExtension::<BaseElement>::ZERO);

        // test random values
        let r1: CubeExtension<BaseElement> = rand_value();
        let r2: CubeExtension<BaseElement> = rand_value();

        let expected = CubeExtension(r1.0 + r2.0, r1.1 + r2.1);
        assert_eq!(expected, r1 + r2);
    }

    #[test]
    fn sub() {
        // identity
        let r: CubeExtension<BaseElement> = rand_value();
        assert_eq!(r, r - CubeExtension::<BaseElement>::ZERO);

        // test random values
        let r1: CubeExtension<BaseElement> = rand_value();
        let r2: CubeExtension<BaseElement> = rand_value();

        let expected = CubeExtension(r1.0 - r2.0, r1.1 - r2.1);
        assert_eq!(expected, r1 - r2);
    }

    // INITIALIZATION
    // --------------------------------------------------------------------------------------------

    #[test]
    fn zeroed_vector() {
        let result = CubeExtension::<BaseElement>::zeroed_vector(4);
        assert_eq!(4, result.len());
        for element in result.into_iter() {
            assert_eq!(CubeExtension::<BaseElement>::ZERO, element);
        }
    }

    // SERIALIZATION / DESERIALIZATION
    // --------------------------------------------------------------------------------------------

    #[test]
    fn elements_as_bytes() {
        let source = vec![
            CubeExtension(BaseElement::new(1), BaseElement::new(2)),
            CubeExtension(BaseElement::new(3), BaseElement::new(4)),
        ];

        let expected: Vec<u8> = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];

        assert_eq!(
            expected,
            CubeExtension::<BaseElement>::elements_as_bytes(&source)
        );
    }

    #[test]
    fn bytes_as_elements() {
        let bytes: Vec<u8> = vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 5,
        ];

        let expected = vec![
            CubeExtension(BaseElement::new(1), BaseElement::new(2)),
            CubeExtension(BaseElement::new(3), BaseElement::new(4)),
        ];

        let result = unsafe { CubeExtension::<BaseElement>::bytes_as_elements(&bytes[..64]) };
        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());

        let result = unsafe { CubeExtension::<BaseElement>::bytes_as_elements(&bytes) };
        assert!(matches!(result, Err(DeserializationError::InvalidValue(_))));

        let result = unsafe { CubeExtension::<BaseElement>::bytes_as_elements(&bytes[1..]) };
        assert!(matches!(result, Err(DeserializationError::InvalidValue(_))));
    }

    // UTILITIES
    // --------------------------------------------------------------------------------------------

    #[test]
    fn as_base_elements() {
        let elements = vec![
            CubeExtension(BaseElement::new(1), BaseElement::new(2)),
            CubeExtension(BaseElement::new(3), BaseElement::new(4)),
        ];

        let expected = vec![
            BaseElement::new(1),
            BaseElement::new(2),
            BaseElement::new(3),
            BaseElement::new(4),
        ];

        assert_eq!(
            expected,
            CubeExtension::<BaseElement>::as_base_elements(&elements)
        );
    }
}
*/