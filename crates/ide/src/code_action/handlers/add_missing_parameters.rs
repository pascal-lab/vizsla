use hir::{
    container::{InContainer, InModule},
    db::HirDb,
    hir_def::{declaration::Declaration, module::instantiation::ParamAssign},
};
use itertools::{Either, Itertools};
use rustc_hash::FxHashSet;
use smol_str::SmolStr;
use syntax::{ast, has_text_range::HasTextRange};
use utils::get::GetRef;

use crate::code_action::{CodeActionCollector, CodeActionCtx, CodeActionId, CodeActionKind};

const ID: CodeActionId =
    CodeActionId { name: "add_missing_parameters", kind: CodeActionKind::Generate };
const LABEL: &str = "Fill parameters";

pub(super) fn add_missing_parameters(
    collector: &mut CodeActionCollector,
    ctx: &CodeActionCtx,
) -> Option<()> {
    let sema = ctx.sema;
    let db = sema.db;

    let ast_instantiation = ctx.find_node_at_offset::<ast::HierarchyInstantiation>()?;
    let InModule { value: instantiation_id, module_id } =
        sema.resolve_instantiation(ast_instantiation);
    let module = db.module(module_id);
    let instantiation = module.get(instantiation_id);

    let params_node = ast_instantiation.parameters()?;
    let insert_offset = params_node.close_paren()?.text_range()?.start();

    let target_module_id = sema.nameres_instantiation(ast_instantiation)?;
    let target_module = db.module(target_module_id);

    let is_ordered = instantiation
        .param_assigns
        .first()
        .map(|id| matches!(module.get(*id), ParamAssign::Ordered(_)))
        .unwrap_or_default();

    let names = if is_ordered {
        let assigned = instantiation.param_assigns.len();

        let names = target_module
            .declarations
            .values()
            .take_while(|declaration| matches!(declaration, Declaration::ParamDecl(_)))
            .flat_map(|declaration| declaration.decls())
            .filter_map(|decl| target_module.get(decl).name.clone())
            .skip(assigned)
            .collect_vec();

        Either::Left(names)
    } else {
        let mut names = FxHashSet::default();

        for decl_id in target_module.declarations.values() {
            if let Declaration::ParamDecl(_) = decl_id {
                for decl in decl_id.decls() {
                    if let Some(name) = target_module.get(decl).name.clone() {
                        names.insert(name);
                    }
                }
            }
        }

        for param_id in instantiation.param_assigns.iter() {
            match module.get(*param_id) {
                ParamAssign::Named(Some(name), _) => {
                    names.remove(name);
                }
                ParamAssign::Ordered(_) => return None,
                _ => {}
            }
        }

        Either::Right(names)
    };

    collector.add(ID, LABEL, ctx.range, |builder| {
        let mut text = String::new();
        let cont_id = module_id.into();
        let mut add_to_text = |name: SmolStr| match (
            sema.name_to_def(InContainer::new(cont_id, name.clone())),
            is_ordered,
        ) {
            (None, true) => text.push_str(&format!("/* {name} */, ")),
            (None, false) => text.push_str(&format!(".{name}(), ")),
            (Some(_), true) => text.push_str(&format!("{name}, ")),
            (Some(_), false) => text.push_str(&format!(".{name}, ")),
        };

        match names {
            Either::Left(names) => {
                names.into_iter().for_each(&mut add_to_text);
            }
            Either::Right(names) => {
                names.into_iter().for_each(&mut add_to_text);
            }
        }

        if !text.is_empty() {
            assert!(text.pop() == Some(' '));
            assert!(text.pop() == Some(','));
        }

        builder.insert(insert_offset, text);
    });

    Some(())
}
