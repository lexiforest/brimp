use std::collections::HashSet;

use swc_common::{DUMMY_SP, FileName, GLOBALS, Globals, Mark, SourceMap, sync::Lrc};
use swc_ecma_ast::{
    CallExpr, Callee, Expr, IdentName, KeyValueProp, Lit, MetaPropExpr, MetaPropKind, ModuleDecl,
    ModuleItem, ObjectLit, Pass, Program, Prop, PropName, PropOrSpread, Str,
};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax, lexer::Lexer};
use swc_ecma_transforms_base::{
    fixer::fixer,
    helpers::{HELPERS, Helpers, inject_helpers},
    hygiene::hygiene,
    resolver,
};
use swc_ecma_transforms_module::{
    common_js::{FeatureFlag, common_js},
    path::Resolver,
    util::Config as ModuleConfig,
};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith};

pub(crate) struct CompiledModule {
    pub(crate) dependencies: Vec<String>,
    pub(crate) code: String,
}

pub(crate) fn compile(source: &str, url: &str) -> Result<CompiledModule, String> {
    GLOBALS.set(&Globals::new(), || {
        HELPERS.set(&Helpers::new(false), || compile_with_globals(source, url))
    })
}

fn compile_with_globals(source: &str, url: &str) -> Result<CompiledModule, String> {
    let source_map: Lrc<SourceMap> = Default::default();
    let source_file =
        source_map.new_source_file(FileName::Custom(url.to_owned()).into(), source.to_owned());
    let lexer = Lexer::new(
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|error| format!("could not parse module {url}: {error:?}"))?;
    if let Some(error) = parser.take_errors().into_iter().next() {
        return Err(format!("could not parse module {url}: {error:?}"));
    }

    let mut collector = DependencyCollector::default();
    module.visit_with(&mut collector);

    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();
    let mut program = Program::Module(module);
    program.visit_mut_with(&mut ImportMetaRewriter { url });
    resolver(unresolved_mark, top_level_mark, false).process(&mut program);
    common_js(
        Resolver::Default,
        unresolved_mark,
        ModuleConfig::default(),
        FeatureFlag {
            support_block_scoping: true,
            support_arrow: true,
        },
    )
    .process(&mut program);
    inject_helpers(unresolved_mark).process(&mut program);
    hygiene().process(&mut program);
    fixer(None).process(&mut program);

    Ok(CompiledModule {
        dependencies: collector.dependencies,
        code: swc_ecma_codegen::to_code_default(source_map, None, &program),
    })
}

struct ImportMetaRewriter<'a> {
    url: &'a str,
}

impl VisitMut for ImportMetaRewriter<'_> {
    fn visit_mut_expr(&mut self, expression: &mut Expr) {
        if matches!(
            expression,
            Expr::MetaProp(MetaPropExpr {
                kind: MetaPropKind::ImportMeta,
                ..
            })
        ) {
            *expression = Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(IdentName::new("url".into(), DUMMY_SP)),
                    value: Box::new(Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: self.url.into(),
                        raw: None,
                    }))),
                })))],
            });
            return;
        }
        expression.visit_mut_children_with(self);
    }
}

#[derive(Default)]
struct DependencyCollector {
    dependencies: Vec<String>,
    seen: HashSet<String>,
}

impl DependencyCollector {
    fn push(&mut self, value: &swc_ecma_ast::Str) {
        let value = value.value.to_string_lossy().into_owned();
        if self.seen.insert(value.clone()) {
            self.dependencies.push(value);
        }
    }
}

impl Visit for DependencyCollector {
    fn visit_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::ModuleDecl(ModuleDecl::Import(declaration)) => self.push(&declaration.src),
            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(declaration)) => {
                if let Some(source) = &declaration.src {
                    self.push(source);
                }
            }
            ModuleItem::ModuleDecl(ModuleDecl::ExportAll(declaration)) => {
                self.push(&declaration.src)
            }
            _ => {}
        }
        item.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, call: &CallExpr) {
        if matches!(call.callee, Callee::Import(_))
            && let Some(argument) = call.args.first()
            && argument.spread.is_none()
            && let Expr::Lit(Lit::Str(source)) = &*argument.expr
        {
            self.push(source);
        }
        call.visit_children_with(self);
    }
}

#[cfg(test)]
mod tests {
    use super::compile;

    #[test]
    fn compiles_static_and_dynamic_imports_to_common_js() {
        let compiled = compile(
            r#"import { value } from './dependency.js';
               export const result = value + 1;
               globalThis.metaUrl = import.meta.url;
               globalThis.lazy = () => import('./lazy.js');"#,
            "https://example.test/main.js",
        )
        .unwrap();

        assert_eq!(compiled.dependencies, ["./dependency.js", "./lazy.js"]);
        assert!(compiled.code.contains("require(\"./dependency.js\")"));
        assert!(compiled.code.contains("require(\"./lazy.js\")"));
        assert!(!compiled.code.contains("export const"));
        assert!(!compiled.code.contains("import.meta"));
        assert!(compiled.code.contains("https://example.test/main.js"));
    }
}
