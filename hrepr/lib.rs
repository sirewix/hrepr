//! Structural, type-parameter-agnostic representations for Rust types.
//!
//! The [`Generic`] trait (and its `#[derive(Generic)]` macro) gives every type
//! a structural representation built from heterogeneous lists and coproducts,
//! separating a type's *shape* from its *polymorphic parameters*. This lets you
//! write generic algorithms — schema generators, reflection layers, codecs —
//! that operate on the representation without knowing the concrete parameters.
//!
//! Product types (structs) use heterogeneous lists ([`HCons`] / [`HNil`]),
//! tagged by [`NamedStruct`], [`UnnamedStruct`] or [`UnitStruct`]. Sum types
//! (enums) use coproducts ([`HSum`] / [`HSumNil`]) wrapped in [`Enum`].
//! Polymorphic parameters are abstracted into [`TypeVar`]s numbered with the
//! Peano numerals [`Zero`] and [`Succ`].

#![no_std]
#![warn(missing_docs)]

pub use hrepr_derive::Generic;

/// A type together with a structural, type-parameter-agnostic representation
/// of itself.
///
/// The representation is built from heterogeneous lists ([`HCons`] / [`HNil`])
/// and coproducts ([`HSum`] / [`HSumNil`]); polymorphic type parameters become
/// [`TypeVar`]s in [`Mocked`](Generic::Mocked), while [`Repr`](Generic::Repr)
/// keeps the concrete types.
pub trait Generic {
    /// Type list of polymorphic (generic) parameters.
    type Params;
    /// Generic representation of the type.
    type Repr;
    /// Self but with mocked type parameters.
    type Mocked: Generic;

    /// Convert `self` into its generic representation.
    ///
    /// This is lossless: [`from_repr`](Generic::from_repr) of the result
    /// returns the original value.
    fn to_repr(self) -> Self::Repr;

    /// Reconstruct a value from its generic representation.
    fn from_repr(_: Self::Repr) -> Self;
}

/// A heterogeneous list cell: a head value `H` followed by the tail `T`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HCons<H, T>(pub H, pub T);

/// The empty heterogeneous list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HNil;

/// A coproduct cell: either the head `H` or the tail `T`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HSum<H, T> {
    /// The head alternative.
    L(H),
    /// The tail alternative.
    R(T),
}

/// The empty coproduct, i.e. an uninhabited sum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HSumNil {}

/// The successor of the Peano number `X`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Succ<X>(core::marker::PhantomData<X>);

/// The Peano number zero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zero;

/// A Peano-encoded natural number usable at the type level.
pub trait PeanoNumber {
    /// The value of this number as a `usize`.
    const NUMBER: usize;
}

impl PeanoNumber for Zero {
    const NUMBER: usize = 0;
}

impl<P: PeanoNumber> PeanoNumber for Succ<P> {
    const NUMBER: usize = P::NUMBER + 1;
}

/// A mocked polymorphic type parameter, indexed by a Peano numeral `X`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeVar<X>(pub X);

/// Representation of a struct with named fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NamedStruct<X>(pub X);

/// Representation of a struct or variant with unnamed (tuple) fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnnamedStruct<X>(pub X);

/// Representation of a unit struct or unit variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitStruct;

/// Representation of an enum as a coproduct of its variants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Enum<X>(pub X);

/// A type-level list whose length is available as a [`PeanoNumber`].
pub trait HLen {
    /// Length of the list.
    type Len: PeanoNumber;
}
impl<H, T: HLen> HLen for HCons<H, T> {
    type Len = Succ<T::Len>;
}
impl HLen for HNil {
    type Len = Zero;
}

impl<H, T: HLen> HLen for (H, T) {
    type Len = Succ<T::Len>;
}
impl HLen for () {
    type Len = Zero;
}
