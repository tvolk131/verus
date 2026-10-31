use super::super::prelude::*;
use core::clone::Clone;

verus! {

#[verifier::external_trait_specification]
pub trait ExClone: Sized {
    type ExternalTraitSpecificationFor: core::clone::Clone;

    fn clone(&self) -> Self;
}

#[verifier::external_trait_specification]
pub trait ExCopy: Clone {
    type ExternalTraitSpecificationFor: core::marker::Copy;
}

/*
#[verifier::external_fn_specification]
pub fn ex_clone_clone_from<T: Clone>(a: &mut T, b: &T)
{
    a.clone_from(b)
}
*/

pub assume_specification[ <bool as Clone>::clone ](b: &bool) -> (res: bool)
    returns
        b,
;

pub assume_specification[ <char as Clone>::clone ](c: &char) -> (res: char)
    returns
        c,
;

#[allow(suspicious_double_ref_op)]
pub assume_specification<'b, T: core::marker::PointeeSized, 'a>[ <&'b T as Clone>::clone ](
    b: &'a &'b T,
) -> (res: &'b T)
    ensures
        res == b,
;

pub assume_specification<T: Clone, const N: usize>[ <[T; N] as Clone>::clone ](a: &[T; N]) -> (res:
    [T; N])
    ensures
        forall|i| #![all_triggers] 0 <= i < N ==> cloned::<T>(a@[i], res@[i]),
        a@ =~= res@ ==> a@ == res@,
;

/*
#[verifier::external_fn_specification]
pub fn ex_bool_clone_from(dest: &mut bool, source: &bool)
    ensures *dest == source,
{
    dest.clone_from(source)
}
*/

// Cloning a Tracked copies the underlying ghost T
pub assume_specification<T: Copy>[ <Tracked<T> as Clone>::clone ](b: &Tracked<T>) -> (res: Tracked<
    T,
>)
    ensures
        res == b,
;

pub assume_specification<T>[ <Ghost<T> as Clone>::clone ](b: &Ghost<T>) -> (res: Ghost<T>)
    ensures
        res == b,
;

} // verus!
// Verified models for Rust's builtin tuple Clone shim. `cloned` permits the
// compiler's copy optimization without assuming arbitrary Clone is identity.
macro_rules! tuple_clone_model {
    ($diagnostic:literal, $name:ident, $($t:ident $i:tt),+) => {
        verus! {
            #[rustc_diagnostic_item = $diagnostic]
            pub fn $name<$($t: Clone),+>(value: &($($t,)+)) -> (result: ($($t,)+))
                ensures $(cloned(value.$i, result.$i),)+
            { ($(value.$i.clone(),)+) }
        }
    };
}
tuple_clone_model!("verus::vstd::std_specs::clone::tuple_clone_1", tuple_clone_1, A 0);
tuple_clone_model!("verus::vstd::std_specs::clone::tuple_clone_2", tuple_clone_2, A 0, B 1);
tuple_clone_model!("verus::vstd::std_specs::clone::tuple_clone_3", tuple_clone_3, A 0, B 1, C 2);
tuple_clone_model!("verus::vstd::std_specs::clone::tuple_clone_4", tuple_clone_4, A 0, B 1, C 2, D 3);

verus! {

/// An explicit, checked value law. `Copy` by itself does not supply this law.
/// Implementations prove it against the actual Clone contract, including the
/// reflexive closure used by standard container specifications.
pub trait CloneIdentity: Clone {
    proof fn lemma_clone_identity(a: Self, b: Self)
        requires
            cloned(a, b),
        ensures
            a == b,
    ;
}

pub broadcast proof fn lemma_cloned_identity<T: CloneIdentity>(a: T, b: T)
    requires
        #[trigger] cloned(a, b),
    ensures
        a == b,
{
    T::lemma_clone_identity(a, b);
}

/// A checked law about native `Clone`, expressed in the application's deep view.
/// Unlike `CloneIdentity`, this permits distinct allocated representations.
pub trait CloneView: Clone + DeepView {
    proof fn lemma_clone_view(a: Self, b: Self)
        requires
            cloned(a, b),
        ensures
            a.deep_view() == b.deep_view(),
    ;
}

pub broadcast proof fn lemma_cloned_view<T: CloneView>(a: T, b: T)
    requires
        #[trigger] cloned(a, b),
    ensures
        a.deep_view() == b.deep_view(),
{
    T::lemma_clone_view(a, b);
}

impl<T: CloneView> CloneView for Option<T> {
    proof fn lemma_clone_view(a: Self, b: Self)
        ensures
            a.deep_view() == b.deep_view(),
    {
        if a != b {
            match a {
                None => {},
                Some(x) => {
                    T::lemma_clone_view(x, b.unwrap());
                },
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: CloneView> CloneView for alloc::vec::Vec<T> {
    proof fn lemma_clone_view(a: Self, b: Self)
        ensures
            a.deep_view() == b.deep_view(),
    {
        broadcast use {crate::std_specs::vec::group_vec_axioms, crate::seq::group_seq_axioms};

        if a != b {
            assert forall|i: int| 0 <= i < a@.len() implies a@[i].deep_view()
                == b@[i].deep_view() by {
                T::lemma_clone_view(a@[i], b@[i]);
            }
            assert(a.deep_view() =~= b.deep_view());
            crate::std_specs::vec::vec_clone_deep_view_proof(a, b);
        }
    }
}

#[cfg(feature = "alloc")]
impl CloneView for alloc::string::String {
    proof fn lemma_clone_view(a: Self, b: Self)
        ensures
            a.deep_view() == b.deep_view(),
    {
    }
}

impl<T: CloneIdentity> CloneIdentity for Option<T> {
    proof fn lemma_clone_identity(a: Self, b: Self)
        ensures
            a == b,
    {
        if a != b {
            match a {
                None => {},
                Some(x) => {
                    T::lemma_clone_identity(x, b.unwrap());
                },
            }
        }
    }
}

impl<T: CloneIdentity, const N: usize> CloneIdentity for [T; N] {
    proof fn lemma_clone_identity(a: Self, b: Self)
        ensures
            a == b,
    {
        broadcast use crate::array::group_array_axioms;
        broadcast use crate::seq::axiom_seq_ext_equal;

        if a != b {
            assert forall|i: int| 0 <= i < N implies a@[i] == b@[i] by {
                T::lemma_clone_identity(a@[i], b@[i]);
            }
            assert(a@ =~= b@);
        }
    }
}

} // verus!
macro_rules! identity_scalar {
    ($($t:ty),*) => {
        verus!{$(
        impl CloneIdentity for $t {
            proof fn lemma_clone_identity(a: Self,b: Self)
                ensures a==b,
            {}
        }
        impl CloneView for $t {
            proof fn lemma_clone_view(a: Self,b: Self)
                ensures a.deep_view()==b.deep_view(),
            { Self::lemma_clone_identity(a,b); }
        }
    )*}
    };
}
identity_scalar!(bool, char, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

macro_rules! identity_tuple {
    ($($t:ident $i:tt),+) => {
        verus!{
        impl<$($t: CloneIdentity),+> CloneIdentity for ($($t,)+) {
            proof fn lemma_clone_identity(a: Self,b: Self)
                ensures a==b,
            {
                if a!=b { $($t::lemma_clone_identity(a.$i,b.$i);)+ }
            }
        }
    }
    };
}
identity_tuple!(A 0);
identity_tuple!(A 0, B 1);
identity_tuple!(A 0, B 1, C 2);
identity_tuple!(A 0, B 1, C 2, D 3);
