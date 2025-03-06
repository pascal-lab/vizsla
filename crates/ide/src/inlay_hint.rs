use hir::{
    container::InFile,
    db::HirDb,
    file::HirFileId,
    hir_def::{
        expr::Expr,
        file::FileItem,
        module::{
            Module, ModuleId, ModuleSourceMap,
            instantiation::{Instantiation, ParamAssign, PortConn},
            port::Ports,
        },
    },
    scope::UnitEntry,
    source_map::IsSrc,
};
use ide_db::root_db::RootDb;
use syntax::{ast, match_ast_kind};
use utils::{
    get::{Get, GetRef},
    text_edit::{TextEdit, TextRange, TextSize},
};
use vfs::FileId;

use crate::markup::Markup;

#[derive(Debug, Copy, Clone, Hash)]
pub enum InlayKind {
    ParamAssign,
    Port,
}

#[derive(Debug)]
pub struct InlayHint {
    // The text range this inlay hint applies to.
    pub range: TextRange,

    pub label: String,
    pub tooltip: Option<Markup>,
    pub target_location: Option<InFile<TextRange>>,

    pub position: TextSize,
    pub kind: InlayKind,
    pub text_edit: Option<TextEdit>,
}

struct InlayHintCollector {
    hints: Vec<InlayHint>,
    range: TextRange,
}

impl InlayHintCollector {
    fn new(range: TextRange) -> Self {
        Self { hints: Vec::new(), range }
    }

    fn collect_hint(
        &mut self,
        src: impl IsSrc,
        target_src: Option<InFile<impl IsSrc>>,
        label: String,
        text_edit: Option<TextEdit>,
    ) {
        let range = src.range();

        if range.intersect(self.range).is_none() {
            return;
        }

        let kind = match_ast_kind! { src.kind(),
            ast::ParamAssignment => InlayKind::ParamAssign,
            ast::OrderedPortConnection => InlayKind::Port,
            _ => unimplemented!("{:?}", src.kind()),
        };

        let position = match_ast_kind! { src.kind(),
            _ => range.start(),
        };

        let (tooltip, target_location) = if let Some(InFile { value: src, file_id }) = target_src {
            let location = InFile::new(file_id, src.range());
            (Some(Markup::new()), Some(location))
        } else {
            (None, None)
        };

        self.hints.push(InlayHint {
            range,
            label,
            tooltip,
            target_location,
            position,
            kind,
            text_edit,
        });
    }

    fn collect_port_hint(
        &mut self,
        name: &str,
        conn_src: impl IsSrc,
        target_src: InFile<impl IsSrc>,
    ) {
        self.collect_hint(
            conn_src,
            Some(target_src),
            format!("{name}: "),
            edits_for_conn(name, conn_src),
        );
    }

    fn into_hints(self) -> Vec<InlayHint> {
        self.hints
    }
}

pub(crate) fn inlay_hint(db: &RootDb, file_id: FileId, range: TextRange) -> Vec<InlayHint> {
    let file_id = HirFileId(file_id);
    let (file, src_map) = db.hir_file_with_source_map(file_id);
    let (_file, src_map) = (file.as_ref(), src_map.as_ref());

    let mut collector = InlayHintCollector::new(range);

    for &item in src_map.items.iter() {
        match item {
            FileItem::LocalModuleId(idx) => {
                let module_id = ModuleId::new(file_id, idx);
                let module_src = src_map.get(idx);

                if module_src.range().intersect(range).is_some() {
                    collect_module_items(db, module_id, &mut collector);
                }
            }
            _ => {}
        }
    }

    collector.into_hints()
}

fn collect_module_items(db: &RootDb, module_id: ModuleId, collector: &mut InlayHintCollector) {
    let (module, src_map) = db.module_with_source_map(module_id);
    let (module, src_map) = (module.as_ref(), src_map.as_ref());

    module.instantiations.iter().for_each(|(_, instantiation)| {
        process_instantiation(db, module, src_map, instantiation, collector);
    });
}

fn process_instantiation(
    db: &RootDb,
    module: &Module,
    src_map: &ModuleSourceMap,
    instantiation: &Instantiation,
    collector: &mut InlayHintCollector,
) -> Option<()> {
    let unit_scope = db.unit_scope();
    let target_module_id = match unit_scope.get(instantiation.module_name.as_ref()?) {
        Some(UnitEntry::ModuleId(module_id)) => module_id,
        _ => return None,
    };

    let target_file = target_module_id.file_id;
    let (target_module, target_src_map) = db.module_with_source_map(target_module_id);

    // handle param assignments
    for (id, &assign_id) in instantiation.param_assigns.iter().enumerate() {
        let ParamAssign::Ordered(assign_expr) = module.get(assign_id) else {
            continue;
        };
        let assign_src = src_map.get(assign_id);

        let Some(param_id) = target_module.param_port_id_by_idx(id) else {
            continue;
        };
        let Some(param_name) = target_module.get(param_id).name.as_ref() else {
            continue;
        };

        if should_skip(module.get(*assign_expr), param_name) {
            continue;
        }

        let target_src = InFile::new(target_file, target_src_map.get(param_id));
        collector.collect_port_hint(param_name, assign_src, target_src);
    }

    // handle port connections
    for instance_id in instantiation.instances.iter() {
        let instance = module.get(*instance_id);

        for (id, &conn_id) in instance.connections.iter().enumerate() {
            let PortConn::Ordered(conn_expr) = module.get(conn_id) else {
                continue;
            };

            let conn_src = src_map.get(conn_id);

            match &target_module.ports {
                Ports::NonAnsi { .. } => {
                    let port_id = target_module.non_ansi_port_id_by_idx(id);
                    let Some(port_name) = target_module.get(port_id).label.as_ref() else {
                        continue;
                    };

                    if should_skip(module.get(*conn_expr), port_name) {
                        continue;
                    }

                    let target_src = InFile::new(target_file, target_src_map.get(port_id));
                    collector.collect_port_hint(port_name, conn_src, target_src);
                }
                Ports::Ansi(_) => {
                    let Some(port_id) = target_module.ansi_port_id_by_idx(id) else {
                        continue;
                    };
                    let Some(port_name) = target_module.get(port_id).name.as_ref() else {
                        continue;
                    };

                    if should_skip(module.get(*conn_expr), port_name) {
                        continue;
                    }

                    let target_src = InFile::new(target_file, target_src_map.get(port_id));
                    collector.collect_port_hint(port_name, conn_src, target_src);
                }
            }
        }
    }

    Some(())
}

fn edits_for_conn(param: &str, conn_src: impl IsSrc) -> Option<TextEdit> {
    let mut builder = TextEdit::builder();
    builder.insert(conn_src.range().start(), format!(".{}(", param));
    builder.insert(conn_src.range().end(), String::from(")"));
    Some(builder.finish())
}

fn should_skip(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Ident(ident) if ident == name => true,
        _ => false,
    }
}
