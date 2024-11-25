use hir::{
    db::HirDb,
    file::HirFileId,
    hir_def::{
        block::{BlockId, BlockInfo, BlockItem, BlockSrc, LocalBlockId},
        declaration::{Declaration, DeclarationId, DeclarationSrc},
        expr::declarator::{DeclId, Declarator, DeclaratorSrc, DeclsRange},
        file::FileItem,
        module::{ModuleId, ModuleItem, ModuleSrc, port::Ports},
        stmt::{CaseItem, ForInit, Stmt, StmtId, StmtKind, StmtSrc},
    },
    source_map::{IsNamedSrc, IsSrc},
};
use ide_db::root_db::RootDb;
use la_arena::Idx;
use line_index::TextRange;
use smol_str::SmolStr;
use utils::get::{Get, GetRef};
use vfs::FileId;

use crate::SymbolKind;

const DEFAULT_NAME: SmolStr = SmolStr::new_static("unnamed");

#[derive(Debug, Clone)]
pub struct DocumentSymbol {
    pub name: String,
    pub focus_range: TextRange,
    pub full_range: TextRange,
    pub kind: SymbolKind,
    pub detail: Option<String>,
    pub container_name: Option<String>,
    pub children: Vec<DocumentSymbol>,
}

#[derive(Debug, Clone)]
struct SymbolBuilder {
    sym: DocumentSymbol,
}

impl SymbolBuilder {
    pub fn new(name: &Option<SmolStr>, src: impl IsNamedSrc, container_name: Option<&str>) -> Self {
        Self::new_with_child(name, src, container_name, Vec::new())
    }

    pub fn new_with_child(
        name: &Option<SmolStr>,
        src: impl IsNamedSrc,
        container_name: Option<&str>,
        children: Vec<DocumentSymbol>,
    ) -> Self {
        Self {
            sym: DocumentSymbol {
                name: name.as_ref().unwrap_or(&DEFAULT_NAME).to_string(),
                focus_range: src.name_or_full_range(),
                full_range: src.range(),
                kind: SymbolKind::from_syntax_kind(src.kind()),
                detail: None,
                container_name: container_name.map(|s| s.to_owned()),
                children,
            },
        }
    }

    pub fn detail(mut self, detail: String) -> Self {
        self.sym.detail = Some(detail);
        self
    }

    pub fn kind(mut self, kind: SymbolKind) -> Self {
        self.sym.kind = kind;
        self
    }

    pub fn finish(self) -> DocumentSymbol {
        self.sym
    }
}

// TODO: add ty info in detail
pub(crate) fn document_symbols(db: &RootDb, file_id: FileId) -> Vec<DocumentSymbol> {
    let file_id = HirFileId(file_id);
    let (file, src_map) = db.hir_file_with_source_map(file_id);
    let (file, src_map) = (file.as_ref(), src_map.as_ref());

    let mut res = Vec::with_capacity(src_map.items.len() + file.decls.len());

    for &member in src_map.items.iter() {
        match member {
            FileItem::LocalModuleId(idx) => {
                let module_id = ModuleId::new(file_id, idx);
                let module_src = src_map.get(idx);
                collect_module_items(db, module_id, module_src, &mut res);
            }
            FileItem::ProcId(proc_id) => {
                let proc = file.get(proc_id);
                let stmt_id = proc.stmt;
                build_stmt(db, &mut res, stmt_id, None, file, src_map);
            }
            FileItem::DeclarationId(declaration_id) => {
                build_declaration(&mut res, declaration_id, None, file, src_map);
            }
        }
    }

    res
}

fn collect_module_items(
    db: &RootDb,
    module_id: ModuleId,
    module_src: ModuleSrc,
    res: &mut Vec<DocumentSymbol>,
) {
    let (module, src_map) = db.module_with_source_map(module_id);
    let (module, src_map) = (module.as_ref(), src_map.as_ref());

    let mut children =
        Vec::with_capacity(src_map.items.len() + module.decls.len() + module.stmts.len());
    let module_name = module.name.as_ref().map(|s| s.as_str());

    if let Some(params) = &module.param_ports {
        for declaration_id in params.clone() {
            build_declaration(&mut children, declaration_id, module_name, module, src_map);
        }
    }

    match &module.ports {
        Ports::NonAnsi { ports, .. } => {
            for (port_id, port) in ports.iter() {
                let src = src_map.get(port_id);
                let builder = SymbolBuilder::new(&port.label, src, module_name);
                children.push(builder.finish());
            }
        }
        Ports::Ansi(port_decls) => {
            for port_decl in port_decls.values() {
                build_decls(
                    &mut children,
                    &port_decl.decls,
                    SymbolKind::PortDecl,
                    module_name,
                    module,
                    src_map,
                );
            }
        }
    }

    for item in src_map.items.iter() {
        match *item {
            ModuleItem::DeclarationId(declaration_id) => {
                build_declaration(&mut children, declaration_id, module_name, module, src_map)
            }
            ModuleItem::InstantiationId(instantiation_id) => {
                for &instance_id in module.get(instantiation_id).instances.iter() {
                    let hir = module.get(instance_id);
                    let src = src_map.get(instance_id);
                    let builder = SymbolBuilder::new(&hir.name, src, module_name);
                    children.push(builder.finish());
                }
            }
            ModuleItem::ProcId(proc_id) => {
                let proc = module.get(proc_id);
                let stmt_id = proc.stmt;
                build_stmt(db, &mut children, stmt_id, module_name, module, src_map);
            }
            ModuleItem::PortDeclId(port_decl) => {
                let port_decl = module.get(port_decl);
                build_decls(
                    &mut children,
                    &port_decl.decls,
                    SymbolKind::PortDecl,
                    module_name,
                    module,
                    src_map,
                )
            }
            ModuleItem::ContAssignId(_) => {}
        }
    }

    let builder = SymbolBuilder::new_with_child(&module.name, module_src, None, children);
    res.push(builder.finish());
}

fn collect_block_items(
    db: &RootDb,
    block_id: BlockId,
    block_src: BlockSrc,
    cont_name: Option<&str>,
    res: &mut Vec<DocumentSymbol>,
) {
    let (block, src_map) = db.block_with_source_map(block_id);
    let (block, src_map) = (block.as_ref(), src_map.as_ref());

    let mut children = Vec::with_capacity(src_map.items.len() + block.decls.len());
    let block_name = block.name.as_ref().map(|s| s.as_str());

    for item in src_map.items.iter() {
        match *item {
            BlockItem::DeclarationId(declaration_id) => {
                build_declaration(&mut children, declaration_id, block_name, block, src_map)
            }
            BlockItem::StmtId(stmt_id) => {
                build_stmt(db, &mut children, stmt_id, block_name, block, src_map)
            }
        }
    }

    if block.name.is_some() || !children.is_empty() {
        let builder = SymbolBuilder::new_with_child(&block.name, block_src, cont_name, children);
        res.push(builder.finish());
    }
}

fn build_stmt<Arn, SrcMap>(
    db: &RootDb,
    res: &mut Vec<DocumentSymbol>,
    stmt_id: Idx<Stmt>,
    container_name: Option<&str>,
    arena: &Arn,
    src_map: &SrcMap,
) where
    Arn: GetRef<StmtId, Output = Stmt>
        + GetRef<DeclId, Output = Declarator>
        + GetRef<LocalBlockId, Output = BlockInfo>,
    SrcMap: Get<StmtId, Output = StmtSrc>
        + Get<DeclId, Output = DeclaratorSrc>
        + Get<LocalBlockId, Output = BlockSrc>,
{
    let stmt = arena.get(stmt_id);

    if let StmtKind::Block(block_info) = &stmt.kind {
        let block_id = block_info.block_id;
        let block_src = src_map.get(stmt_id).try_into().unwrap();
        collect_block_items(db, block_id, block_src, container_name, res);
        return;
    }

    let mut children = Vec::with_capacity(5);
    let stmt_name = stmt.label.as_ref().map(|s| s.as_str());
    match &stmt.kind {
        StmtKind::Wait(_, stmt_id)
        | StmtKind::TimingCtrl(_, stmt_id)
        | StmtKind::Forever(stmt_id)
        | StmtKind::DoWhile(stmt_id, _)
        | StmtKind::While(_, stmt_id) => {
            build_stmt(db, &mut children, *stmt_id, stmt_name, arena, src_map)
        }
        StmtKind::Cond { then_stmt, else_stmt, .. } => {
            build_stmt(db, &mut children, *then_stmt, stmt_name, arena, src_map);
            if let Some(else_stmt) = else_stmt {
                build_stmt(db, &mut children, *else_stmt, stmt_name, arena, src_map);
            }
        }
        StmtKind::Case { items, .. } => {
            for item in items {
                let stmt_id = match item {
                    CaseItem::Case { clause, .. } => clause,
                    CaseItem::Default(stmt) => stmt,
                };
                build_stmt(db, &mut children, *stmt_id, stmt_name, arena, src_map);
            }
        }
        StmtKind::For { inits, stmt, .. } => match inits {
            ForInit::Init(inits) => {
                for (_, decl_id) in inits {
                    build_decl(
                        &mut children,
                        *decl_id,
                        SymbolKind::DataDecl,
                        stmt_name,
                        arena,
                        src_map,
                    );
                }
                build_stmt(db, &mut children, *stmt, stmt_name, arena, src_map);
            }
            ForInit::Assign(_) => {}
        },

        StmtKind::Empty
        | StmtKind::Expr(_)
        | StmtKind::Jump(_)
        | StmtKind::ProcAssign(_)
        | StmtKind::Disable(_) => {}

        StmtKind::Block(_) => unreachable!(),
    }

    if stmt.label.is_some() || !children.is_empty() {
        let stmt_src = src_map.get(stmt_id);
        let builder =
            SymbolBuilder::new_with_child(&stmt.label, stmt_src, container_name, children);
        res.push(builder.finish());
    }
}

#[inline]
fn build_declaration<Arn, SrcMap>(
    res: &mut Vec<DocumentSymbol>,
    declaration_id: DeclarationId,
    container_name: Option<&str>,
    arena: &Arn,
    src_map: &SrcMap,
) where
    Arn: GetRef<DeclId, Output = Declarator> + GetRef<DeclarationId, Output = Declaration>,
    SrcMap: Get<DeclId, Output = DeclaratorSrc> + Get<DeclarationId, Output = DeclarationSrc>,
{
    let declaration = arena.get(declaration_id);
    let src = src_map.get(declaration_id);
    build_decls(
        res,
        &declaration.decls(),
        SymbolKind::from_syntax_kind(src.kind()),
        container_name,
        arena,
        src_map,
    );
}

#[inline]
fn build_decls<Arn, SrcMap>(
    res: &mut Vec<DocumentSymbol>,
    decls: &DeclsRange,
    kind: SymbolKind,
    container_name: Option<&str>,
    arena: &Arn,
    src_map: &SrcMap,
) where
    Arn: GetRef<DeclId, Output = Declarator>,
    SrcMap: Get<DeclId, Output = DeclaratorSrc>,
{
    for decl in decls.clone() {
        build_decl(res, decl, kind, container_name, arena, src_map);
    }
}

#[inline]
fn build_decl<Arn, SrcMap>(
    res: &mut Vec<DocumentSymbol>,
    decl: Idx<Declarator>,
    kind: SymbolKind,
    container_name: Option<&str>,
    arena: &Arn,
    src_map: &SrcMap,
) where
    Arn: GetRef<DeclId, Output = Declarator>,
    SrcMap: Get<DeclId, Output = DeclaratorSrc>,
{
    let hir = arena.get(decl);
    let src = src_map.get(decl);
    let builder = SymbolBuilder::new(&hir.name, src, container_name).kind(kind);
    res.push(builder.finish());
}
