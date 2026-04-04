use super::intrinsic;

intrinsic!(Pow, "llvm.pow", [f64, f64]);
intrinsic!(Powi, "llvm.powi", [f64, i32]);
