/// Extract unsigned bits from a value.
/// `ubits!(0xFF, 7:4)` → `0xF` (bits 7 down to 4)
#[macro_export]
macro_rules! ubits {
    ($val:expr, $hi:literal : $lo:literal) => {
        ($val >> $lo) & ((1 << ($hi - $lo + 1)) - 1)
    };
}

/// Sign-extend a value from a given bit width to i32.
/// `sext!(0xFFF, 12)` → `-1` (12-bit all-ones become -1 in i32)
#[macro_export]
macro_rules! sext {
    ($val:expr, $bits:expr) => {{
        let shift = 32 - $bits;
        (($val as i32) << shift) >> shift
    }};
}

/// Extract a contiguous bit field and sign-extend it.
/// `ibits!(0xFFFF_F800, 31:20)` → `-1` (bits 31:20 = 0xFFF, sign-extended)
#[macro_export]
macro_rules! ibits {
    ($val:expr, $hi:literal : $lo:literal) => {
        $crate::sext!($crate::ubits!($val, $hi:$lo), $hi - $lo + 1)
    };
}

/// Gather scattered bit fields into a single u32.
/// Each arm maps source bits to a destination position.
/// `gather!(inst, 31:25 => 5, 11:7 => 0)` → inst[31:25] placed at imm[11:5],
///                                            inst[11:7] placed at imm[4:0]
#[macro_export]
macro_rules! gather {
    ($val:expr, $( $hi:literal : $lo:literal => $dst:literal ),+ $(,)?) => {
        ( 0u32 $( | (ubits!($val, $hi:$lo) << $dst) )+ )
    };
}
