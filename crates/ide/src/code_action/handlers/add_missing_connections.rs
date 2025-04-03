use hir::{
    container::{InContainer, InModule},
    db::HirDb,
    hir_def::module::{instantiation::PortConn, port::Ports},
};
use itertools::Either;
use rustc_hash::FxHashSet;
use smol_str::SmolStr;
use syntax::{
    ast::{self, AstNode},
    has_text_range::HasTextRange,
};
use utils::get::GetRef;

use crate::code_action::{CodeActionCollector, CodeActionCtx, CodeActionId, CodeActionKind};

const ID: CodeActionId =
    CodeActionId { name: "add_missing_connections", kind: CodeActionKind::Generate };
const LABEL: &str = "Fill connections";

pub(super) fn add_missing_connections(
    collector: &mut CodeActionCollector,
    ctx: &CodeActionCtx,
) -> Option<()> {
    let sema = ctx.sema;
    let db = sema.db;

    let ast_instance = ctx.find_node_at_offset::<ast::HierarchicalInstance>()?;
    let InModule { value: instance_id, module_id } = sema.resolve_instance(ast_instance);
    let module = db.module(module_id);
    let instance = module.get(instance_id);
    let insert_offset = ast_instance.close_paren()?.text_range()?.start();

    let instantiation = ast::HierarchyInstantiation::cast(ast_instance.syntax().parent()?)?;
    let target_module_id = sema.nameres_instantiation(instantiation)?;
    let target_module = db.module(target_module_id);

    let is_ordered = instance
        .connections
        .first()
        .map(|id| matches!(module.get(*id), PortConn::Ordered(_) | PortConn::Empty))
        .unwrap_or_default();

    let names = if is_ordered {
        let mut names = Vec::default();
        let connected = instance.connections.len();
        match &target_module.ports {
            Ports::NonAnsi { ports, .. } => {
                ports.values().skip(connected).filter_map(|port| port.label.clone()).for_each(
                    |label| {
                        names.push(label);
                    },
                );
            }
            Ports::Ansi(ports) => {
                ports
                    .values()
                    .flat_map(|port| port.decls.clone())
                    .skip(connected)
                    .filter_map(|decl| target_module.get(decl).name.clone())
                    .for_each(|name| {
                        names.push(name);
                    });
            }
        }
        Either::Left(names)
    } else {
        let mut names = FxHashSet::default();
        match &target_module.ports {
            Ports::NonAnsi { ports, .. } => {
                ports.values().filter_map(|port| port.label.clone()).for_each(|label| {
                    names.insert(label);
                });
            }
            Ports::Ansi(ports) => {
                ports
                    .values()
                    .flat_map(|port| port.decls.clone())
                    .filter_map(|decl| target_module.get(decl).name.clone())
                    .for_each(|name| {
                        names.insert(name);
                    });
            }
        }

        for conn_id in instance.connections.iter() {
            match module.get(*conn_id) {
                PortConn::Named(Some(name), _) => {
                    names.remove(name);
                }
                PortConn::Ordered(_) => return None,
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
                names.into_iter().for_each(|name| add_to_text(name));
            }
            Either::Right(names) => {
                names.into_iter().for_each(|name| add_to_text(name));
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
