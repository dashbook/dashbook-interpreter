use ahash::RandomState;
use js_sys::Error;
use std::collections::HashMap;
use swc::config::SourceMapsConfig;
use swc::Compiler;
use swc_atoms::Atom;
use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, SourceMap,
};
use swc_common::{BytePos, Globals, Span, SyntaxContext, GLOBALS};
use swc_ecma_ast::*;
use swc_ecma_codegen::Config;
use swc_ecma_visit::{noop_fold_type, Fold, FoldWith};

#[inline]
pub fn emit_js(body: Vec<Stmt>) -> Result<String, Error> {
    GLOBALS.set(&Globals::default(), || {
        let script = Script {
            span: Span::new(BytePos(0), BytePos(0), SyntaxContext::empty()),
            body: body,
            shebang: None,
        };
        let program = Program::Script(script);

        let cm: Lrc<SourceMap> = Default::default();
        let _fm = cm.new_source_file(FileName::Custom("jsfunction".into()), "".into());
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
        let pass = Folded;
        let compiler = Compiler::new(cm);

        let transformed_program = compiler.transform(&handler, program, false, pass);
        let hasher = HashMap::with_hasher(RandomState::new());

        compiler
            .print(
                &transformed_program,
                None,
                None,
                false,
                SourceMapsConfig::Bool(false),
                &hasher,
                None,
                None,
                true,
                "",
                Config::default().with_target(EsVersion::Es2015),
            )
            .map(|y| y.code)
            .map_err(|_| Error::new("Error"))
    })
}

struct Folded;

impl Fold for Folded {
    noop_fold_type!();

    fn fold_expr(&mut self, expr: Expr) -> Expr {
        let expr = match expr {
            Expr::Member(expr) => match &expr.prop {
                MemberProp::Ident(_) => Expr::Member(MemberExpr {
                    obj: expr.obj.fold_with(self),
                    ..expr
                }),
                MemberProp::PrivateName(_) => Expr::Member(MemberExpr {
                    obj: expr.obj.fold_with(self),
                    ..expr
                }),
                MemberProp::Computed(_) => Expr::Member(MemberExpr {
                    obj: expr.obj.fold_with(self),
                    prop: expr.prop.fold_with(self),
                    ..expr
                }),
            },
            _ => expr.fold_children_with(self),
        };

        match expr {
            Expr::Ident(ident) => {
                let mut string = String::from("weblab_");
                string.push_str(&ident.sym);
                // It's ok because we don't recurse into member expressions.
                return Expr::Ident(Ident {
                    sym: Atom::from(string),
                    span: ident.span,
                    optional: ident.optional,
                });
            }
            _ => {}
        };

        expr
    }

    fn fold_pat(&mut self, pat: Pat) -> Pat {
        let new_pat = pat.fold_children_with(self);

        match new_pat {
            Pat::Ident(ident) => {
                let mut string = String::from("weblab_");
                string.push_str(&ident.id.sym);
                // It's ok because we don't recurse into member expressions.
                return Pat::Ident(BindingIdent {
                    id: Ident {
                        sym: Atom::from(string),
                        span: ident.id.span,
                        optional: ident.id.optional,
                    },
                    type_ann: ident.type_ann,
                });
            }
            _ => {}
        }

        new_pat
    }
}
