// SAFETY:
// - ensure that `$t.$i` and `$proxy` are "basically" are the same if little endian (like `u16_le` and `u16`)
// - `NoUndef` trait shoudl be safe to implement
#[macro_export]
macro_rules! unsafe_resolve_as {
    (tuple: $($proxy:ty, $i:tt),*) => {
        pub struct Resolver (
            $( <$proxy as ::rkyv::Archive>::Resolver),*
        );
    };
    (struct: $($proxy:ty, $i:tt),*) => {
        pub struct Resolver {
            $($i: <$proxy as ::rkyv::Archive>::Resolver),*
        }
    };
    ($t:ty, $m:ident, $res_type:ident, $($proxy:ty, $i:tt),*) => { mod $m {
        use super::*;

        #[cfg(target_endian = "big")]
        compile_error!("Big endian machines are not supported. Code becomes a mess");

        // SAFETY: We do not support big endian machines
        unsafe impl ::rkyv::Portable for $t {}
        // SAFETY: Macro precondition
        unsafe impl ::rkyv::traits::NoUndef for $t {}

        $crate::unsafe_resolve_as!($res_type: $($proxy, $i),*);

        #[automatically_derived]
        impl ::rkyv::Archive for $t {
            type Archived = Self;

            type Resolver = Resolver;

            fn resolve(&self, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
                $(
                    let field_ptr = unsafe { &raw mut (*out.ptr()).$i }.cast::<$proxy>();
                    let field_out = unsafe { ::rkyv::Place::from_field_unchecked(out, field_ptr) };
                    {
                        let original = self.$i;
                        assert_eq!(size_of_val(&original), size_of::<$proxy>());
                        assert_eq!(align_of_val(&original), align_of::<$proxy>());
                    }
                    let v = <$proxy>::from(self.$i);
                    <$proxy as ::rkyv::Archive>::resolve(&v, resolver.$i, field_out);
                )*
            }
        }


        // this can be removed if we remove `repr(packed)` from TIs
        #[automatically_derived]
        impl<__S: ::rkyv::rancor::Fallible + ?Sized> ::rkyv::Serialize<__S> for $t
        where
            $($proxy: ::rkyv::Serialize<__S>,)*
        {
            fn serialize(
                &self,
                serializer: &mut __S,
            ) -> ::core::result::Result<
                <Self as ::rkyv::Archive>::Resolver,
                <__S as ::rkyv::rancor::Fallible>::Error,
            > {
                ::core::result::Result::Ok(Resolver {
                    $(
                        $i: {
                            let value = self.$i.into();
                            <$proxy as ::rkyv::Serialize< __S>>::serialize(&value, serializer)?
                        }
                    ),*
                })
            }
        }
    }}
}
