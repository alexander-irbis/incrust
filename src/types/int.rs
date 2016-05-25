use super::abc::*;


impl Type for isize {
    fn iclone<'a>(&self) -> BType<'a> { Box::new(*self) }
    fn to_bool(&self) -> bool { *self != 0 }
}

impl AsReal for isize {
    fn as_real(&self) -> Option<f64> { Some(*self as f64) }
}

impl AsInt for isize {
    fn as_int(&self) -> Option<isize> { Some(*self) }
}


#[cfg_attr(feature = "clippy", allow(boxed_local))]
impl IArithm for isize {
    fn iadd(self: Box<Self>, other: BType) -> Option<BType> { other.as_int().map(|s| -> BType { Box::new(*self + s) }) }
    fn isub(self: Box<Self>, other: BType) -> Option<BType> { other.as_int().map(|s| -> BType { Box::new(*self - s) }) }
    fn imul(self: Box<Self>, other: BType) -> Option<BType> { other.as_int().map(|s| -> BType { Box::new(*self * s) }) }
    fn idiv(self: Box<Self>, other: BType) -> Option<BType> { other.as_int().map(|s| -> BType { Box::new(*self / s) }) }
}



impl <'a> Into<BType<'a>> for isize { fn into(self) -> BType<'a> { Box::new(self) } }
