# hrepr

Structural, type-parameter-agnostic representations for Rust types.

`hrepr` derives an **HList-based representation** of your structs and enums,
separating the *shape* of a type from its *polymorphic parameters*. This is the
building block for writing generic algorithms — schema generators, reflection
layers, codecs — that don't care about concrete type parameters.

## The `Generic` trait

```rust
pub trait Generic {
    /// Type list of polymorphic (generic) parameters.
    type Params;
    /// Generic representation of the type.
    type Repr;
    /// Self but with mocked type parameters.
    type Mocked: Generic;

    fn to_repr(self) -> Self::Repr;
    fn from_repr(_: Self::Repr) -> Self;
}
```

- `Repr` — the structural form of the type, with concrete field types.
- `Mocked` — the same type with every generic parameter replaced by a
  [`TypeVar`] numbered with Peano numerals (`Zero`, `Succ<Zero>`, ...).
- `Params` — a tuple type list of the polymorphic parameters.

Products use heterogeneous lists (`HCons` / `HNil`); sums use coproducts
(`HSum` / `HSumNil`). Struct kinds are tagged with `NamedStruct`,
`UnnamedStruct`, `UnitStruct` and enums with `Enum`.

## Usage

```rust
use hrepr::Generic;

#[derive(Generic)]
struct Foo<X, Y> {
    foo: String,
    boo: u8,
    xoo: X,
    yoo: Y,
}
```

The derive generates:

```rust
impl<X, Y> Generic for Foo<X, Y> {
    type Params = (X, (Y, ()));
    type Mocked = Foo<TypeVar<Zero>, TypeVar<Succ<Zero>>>;
    type Repr = NamedStruct<
        HCons<String, HCons<u8, HCons<X, HCons<Y, HNil>>>>,
    >;

    fn to_repr(self) -> Self::Repr { /* ... */ }
    fn from_repr(repr: Self::Repr) -> Self { /* ... */ }
}
```

`to_repr` and `from_repr` are lossless: `from_repr(x.to_repr()) == x`.

## Layout

- `hrepr` — the trait, HList/coproduct machinery, and re-exported derive.
- `hrepr-derive` — the `#[derive(Generic)]` procedural macro.

## Development

A [Nix](https://nixos.org) flake provides a dev shell with the Rust toolchain:

```sh
nix develop
cargo test
```

## License

MIT — see [LICENSE](LICENSE).