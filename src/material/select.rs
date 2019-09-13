/// The macro for making a union of materials.
///
/// You can read more about the technique [here](https://clay-rs.github.io/knowledge/#objects).
#[macro_export]
macro_rules! material_select {
    ( $Select:ident { $( $Enum:ident ( $Param:ident = $Material:ty ) ),+ $(,)? } ) => {
        $crate::instance_select!(
            $Select: $crate::material::Material: $crate::material::MaterialClass {
                $( $Enum($Param = $Material) ),+
            }
        );
        impl Material for $Select {
            fn brightness(&self) -> f64 {
                match self {
                    $( $Select::$Enum(m) => m.brightness(), )+
                }
            }
        }
    };
}

#[cfg(test)]
mod check {
    use crate::{
        material::{
            Material,
            test::TestMaterial,
        },
        material_select,
    };

    material_select!(
        TestSelect {
            Material1(T1 = TestMaterial<i32>),
            Material2(T2 = TestMaterial<f32>),
        }
    );
}
