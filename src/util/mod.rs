pub mod improved_app_handler;

#[allow(unused_macros)]
macro_rules! structural_eq {
    ($val:expr; $($struct:tt)*) => {
        {
            match $val {
                $($struct)* => true,
                _ => false
            }
        }
    };
}
#[allow(unused_imports)]
pub(crate) use structural_eq;