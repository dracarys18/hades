use super::intrinsic;

intrinsic!(Sqrt, "llvm.sqrt", [f64]);
intrinsic!(Exp, "llvm.exp", [f64]);
intrinsic!(Exp2, "llvm.exp2", [f64]);
intrinsic!(Log, "llvm.log", [f64]);
intrinsic!(Log10, "llvm.log10", [f64]);
intrinsic!(Log2, "llvm.log2", [f64]);
