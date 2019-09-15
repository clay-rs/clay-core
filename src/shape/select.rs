/// The macro for making a union of shapes.
///
/// You can read more about the technique [here](https://clay-rs.github.io/knowledge/#objects).
#[macro_export]
macro_rules! shape_select {
    ( $Select:ident { $( $Enum:ident ( $Param:ident = $Shape:ty ) ),+ $(,)? } ) => {
        $crate::instance_select!(
            $Select: $crate::shape::Shape: $crate::shape::ShapeClass {
                $( $Enum($Param = $Shape) ),+
            }
        );
        impl $crate::shape::Shape for $Select {}
        
        impl<
            B_: $crate::shape::Bound,
            $(
                $Param: 
                    $crate::shape::Shape +
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
    };
}

#[cfg(test)]
mod check {
    use crate::{
        shape::test::TestShape,
        shape_select,
    };

    shape_select!(
        TestShapeSelect {
            Shape1(T1 = TestShape<i32>),
            Shape2(T2 = TestShape<f32>),
        }
    );
}
