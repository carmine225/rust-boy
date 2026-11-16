//macro della cpu
#[macro_export]
macro_rules! get_u16register {
    ($cpu:ident, $high:expr, $low:expr) => {
        (($high as u16) << 8) | ($low as u16)
    };
}
#[macro_export]
macro_rules! set_u16register {
    ($cpu:ident, $high:expr, $low:expr, $val:expr) => {
        $high = (($val >> 8) & 0xFF) as u8;
        $low = ($val & 0xFF) as u8;
    };
}
