use hades_ast::{FuncKind, FunctionSignature, TypedFuncDef, Types};
use hades_error::Span;
use hades_tokens::Name;

use crate::{
    ToMir,
    builder::MirBuilder,
    mir::{
        cfg::Cfg,
        local::LocalDecl,
        terminator::{Terminator, TerminatorKind},
    },
};

/// A fully lowered MIR function.
#[derive(Debug, Clone)]
pub struct MirFunction {
    /// Fully-qualified name (e.g. `"MyStruct::method"` or `"foo"`).
    pub name: Name,

    /// Receiver type if this is a method.
    pub receiver: Option<hades_ast::TypedReceiver>,

    /// Return type (same as `locals[0].ty`).
    pub return_type: Types,

    /// All local variable slots.
    /// `locals[0]`           — return slot `_0`
    /// `locals[1..=arg_count]` — parameter slots `_1.._N`
    /// `locals[arg_count+1..]` — temporaries
    pub locals: Vec<LocalDecl>,

    /// Number of parameter locals (not counting `_0`).
    pub arg_count: usize,

    /// The control-flow graph.
    pub cfg: Cfg,

    /// The original function signature (kept for codegen convenience).
    pub signature: FunctionSignature,

    pub span: Span,
}

impl MirFunction {
    pub fn return_local(&self) -> &LocalDecl {
        &self.locals[0]
    }

    pub fn param_locals(&self) -> &[LocalDecl] {
        &self.locals[1..=self.arg_count]
    }

    pub fn temp_locals(&self) -> &[LocalDecl] {
        &self.locals[self.arg_count + 1..]
    }
}

// ── ToMir impl ────────────────────────────────────────────────────────────────

impl ToMir for TypedFuncDef {
    type Output = Option<MirFunction>;

    fn to_mir(&self, _builder: &mut MirBuilder) -> Option<MirFunction> {
        // Extern / Intrinsic functions have no body to lower.
        match self.signature.kind {
            FuncKind::Extern { .. } | FuncKind::Intrinsic(_) => return None,
            FuncKind::Normal => {}
        }

        let body = match &self.body {
            Some(b) => b,
            None => return None, // forward declaration
        };

        let return_type = self.signature.return_type.clone();
        let span = self.span.clone();
        let mut builder = MirBuilder::new(return_type.clone(), span.clone());

        // Allocate parameter locals `_1.._N`.
        let mut arg_count = 0usize;
        for (param_kind, param_ty) in self.signature.params().into_iter() {
            use hades_tokens::ParamKind;
            match &param_kind {
                ParamKind::Self_(_) => {
                    // `self` param — allocate as a named local "self".
                    let self_ident = hades_tokens::Ident::new(
                        "self".to_string(),
                        span.clone(),
                    );
                    builder.new_named_local(self_ident, param_ty, span.clone());
                    arg_count += 1;
                }
                ParamKind::Ident(ident) => {
                    builder.new_named_local(ident.clone(), param_ty, span.clone());
                    arg_count += 1;
                }
            }
        }

        // Lower the function body into the CFG.
        body.to_mir(&mut builder);

        // If the last block has no terminator (void function), emit implicit return.
        if !builder.is_terminated() {
            // Inline any pending defers.
            let deferred = builder.deferred_stmts();
            for stmt in deferred {
                builder.emit(stmt);
            }
            builder.terminate(Terminator::new(TerminatorKind::Return, span.clone()));
        }

        let (cfg, locals) = builder.finish();

        Some(MirFunction {
            name: self.name.clone(),
            receiver: self.signature.receiver.clone(),
            return_type,
            locals,
            arg_count,
            cfg,
            signature: self.signature.clone(),
            span,
        })
    }
}
