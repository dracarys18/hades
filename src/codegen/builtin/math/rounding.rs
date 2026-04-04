use super::intrinsic;

intrinsic!(Floor, "llvm.floor", [f64]);
intrinsic!(Ceil, "llvm.ceil", [f64]);
intrinsic!(Trunc, "llvm.trunc", [f64]);
intrinsic!(Round, "llvm.round", [f64]);
