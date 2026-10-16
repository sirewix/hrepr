#![allow(clippy::disallowed_names)]

use hrepr::Generic;

#[derive(Generic, Debug, Clone, PartialEq)]
struct Boo<X>(u8, String, X);

#[derive(Generic, Debug, Clone, PartialEq)]
struct Foo<X, Y> {
    foo: String,
    boo: u8,
    xoo: Boo<X>,
    yoo: Y,
}

#[derive(Generic, Debug, Clone, PartialEq)]
enum Xoo<X, Y> {
    Lek(X),
    Pek,
    Kek { fes_o: String, les_i: Y },
}

#[derive(Generic, Debug, Clone, PartialEq)]
struct Yoo;

// helper trait to assert type equality
trait TypeEq<A, B> {
    const YES: bool = true;
}
impl<A> TypeEq<A, A> for () {}

#[test]
fn types_derivations() {
    #![allow(path_statements)]
    use hrepr::*;

    // Yoo
    <() as TypeEq<<Yoo as Generic>::Repr, UnitStruct>>::YES;
    <() as TypeEq<<Yoo as Generic>::Mocked, Yoo>>::YES;
    <() as TypeEq<<Yoo as Generic>::Params, ()>>::YES;

    // Boo
    <() as TypeEq<
        <Boo<bool> as Generic>::Repr,
        UnnamedStruct<HCons<u8, HCons<String, HCons<bool, HNil>>>>,
    >>::YES;
    <() as TypeEq<<Boo<bool> as Generic>::Mocked, Boo<TypeVar<Zero>>>>::YES;
    <() as TypeEq<<Boo<bool> as Generic>::Params, (bool, ())>>::YES;

    // Foo
    <() as TypeEq<
        <Foo<bool, u32> as Generic>::Repr,
        NamedStruct<HCons<String, HCons<u8, HCons<Boo<bool>, HCons<u32, HNil>>>>>,
    >>::YES;
    <() as TypeEq<
        <Foo<bool, u32> as Generic>::Mocked,
        Foo<TypeVar<Zero>, TypeVar<Succ<Zero>>>,
    >>::YES;
    <() as TypeEq<<Foo<bool, u32> as Generic>::Params, (bool, (u32, ()))>>::YES;

    // Xoo
    <() as TypeEq<
        <Xoo<bool, u32> as Generic>::Repr,
        Enum<
            HSum<
                UnnamedStruct<HCons<bool, HNil>>,
                HSum<UnitStruct, HSum<NamedStruct<HCons<String, HCons<u32, HNil>>>, HSumNil>>,
            >,
        >,
    >>::YES;
    <() as TypeEq<
        <Xoo<bool, u32> as Generic>::Mocked,
        Xoo<TypeVar<Zero>, TypeVar<Succ<Zero>>>,
    >>::YES;
    <() as TypeEq<<Xoo<bool, u32> as Generic>::Params, (bool, (u32, ()))>>::YES;

    // composite
    <() as TypeEq<
        <Boo<Boo<bool>> as Generic>::Repr,
        UnnamedStruct<HCons<u8, HCons<String, HCons<Boo<bool>, HNil>>>>,
    >>::YES;
    <() as TypeEq<<Boo<Boo<bool>> as Generic>::Mocked, Boo<TypeVar<Zero>>>>::YES;
    <() as TypeEq<<Boo<Boo<bool>> as Generic>::Params, (Boo<bool>, ())>>::YES;
}

#[test]
fn roundtrip() {
    fn roundtrip<T: Generic + Clone + PartialEq + std::fmt::Debug>(value: T) {
        assert_eq!(T::from_repr(value.clone().to_repr()), value);
    }

    roundtrip(Boo(1, "hi".into(), true));
    roundtrip(Foo { foo: "f".into(), boo: 2, xoo: Boo(3, "x".into(), 4u32), yoo: 5u32 });
    roundtrip(Xoo::<bool, u32>::Lek(true));
    roundtrip(Xoo::<bool, u32>::Pek);
    roundtrip(Xoo::<bool, u32>::Kek { fes_o: "k".into(), les_i: 9 });
    roundtrip(Yoo);
}
