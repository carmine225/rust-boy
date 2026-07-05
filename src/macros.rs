//macro della cpu
#[macro_export]
macro_rules! get_u16register {
    ($cpu:ident, $high:expr, $low:expr) =>{
        
        (($high as u16) << 8) | ($low as u16)
    };
}

