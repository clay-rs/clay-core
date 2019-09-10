#[macro_export]
macro_rules! object_select {
    ( $Select:ident { $( $Enum:ident ( $Param:ident = $Object:ty ) ),+ $(,)? } ) => {
        $crate::instance_select!(
            $Select: $crate::object::Object: $crate::object::ObjectClass {
                $( $Enum($Param = $Object) ),+
            }
        );
        impl $crate::object::Object for $Select {}

        impl<
            B_: $crate::shape::Bound,
            $(
                $Param: 
                    $crate::object::Object +
                    $crate::shape::Bounded<B_>
            ),+
        > $crate::shape::Bounded<B_> for $Select<
            $( $Param ),+
        > {
            fn bound(&self) -> Option<B_> {
                match self {
                    $( $Select::$Enum(x) => x.bound(), )+
                }
            }
        }

        impl<
            T_: $crate::shape::Target,
            $(
                $Param: 
                    $crate::object::Object +
                    $crate::shape::Targeted<T_>
            ),+
        > $crate::shape::Targeted<T_> for $Select<
            $( $Param ),+
        > {
            fn target(&self) -> Option<(T_, f64)> {
                match self {
                    $( $Select::$Enum(x) => x.target(), )+
                }
            }
        }
    };
}

#[cfg(test)]
mod check {
    use crate::{
        shape::test::TestShape,
        material::test::TestMaterial,
        object::Covered,
        object_select,
    };

    object_select!(
        TestSelect {
            Object1(T1 = Covered<TestShape<i32>, TestMaterial<i32>>),
            Object2(T2 = Covered<TestShape<f32>, TestMaterial<f32>>),
        }
    );
}
