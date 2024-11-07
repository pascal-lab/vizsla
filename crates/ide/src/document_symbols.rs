use hir::{
    db::HirDb, file::HirFileId, hir_def::{
        block::{BlockId, BlockInfo, BlockSrc, LocalBlockId},
        expr::declarator::{DeclId, Declarator, DeclaratorSrc},
        module::{instantiation::InstanceSrc, ModuleId, ModuleSrc},
        stmt::{Stmt, StmtId, StmtSrc},
    }, semantics::Semantics, source_map::IsSrc
};
use ide_db::root_db::RootDb;
use line_index::TextRange;
use smol_str::SmolStr;
use syntax::{
    ast::{self, AstNode, PortList},
    has_name::HasName,
    has_text_range::HasTextRange,
    match_ast,
};
use triomphe::Arc;
use utils::get::{Get, GetRef};
use vfs::FileId;

use crate::SymbolKind;

const DEFAULT_NAME: SmolStr = SmolStr::new_static("<unnamed>");

#[derive(Debug, Clone)]
pub struct DocumentSymbol {
    pub name: String,
    pub focus_range: TextRange,
    pub full_range: TextRange,
    pub kind: SymbolKind,
    pub detail: Option<String>,
    pub container_name: Option<String>,
    pub children: Option<Vec<DocumentSymbol>>,
}

// TODO: add ty info in detail
pub(crate) fn document_symbols(db: &RootDb, file_id: FileId) -> Vec<DocumentSymbol> {
    let sema = Semantics::new(db);
    let root = sema.parse(file_id);

    let file_id = HirFileId(file_id);
    let (file, src_map) = db.hir_file_with_source_map(file_id);

    let mut res = Vec::default();

    // We iterate over the syntax tree, to avoid converting SyntaxNodePtr to AST
    // node, which is expensive.
    for member in root.members().children() {
        use ast::Member::*;
        match member {
            ModuleDeclaration(decl) => {
                let src = ModuleSrc::from(decl);
                let module_id = ModuleId::new(file_id, src_map.get(src));
                collect_module_items(db, module_id, decl, &mut res);
            }
            ProceduralBlock(proc) => {
                let stmt = proc.statement();
                build_stmt(db, &mut res, stmt, None, &file, &src_map);
            }
            DataDeclaration(data_decl) => {
                build_decls(&mut res, data_decl.declarators(), None, &file, &src_map);
            }
            NetDeclaration(net_decl) => {
                build_decls(&mut res, net_decl.declarators(), None, &file, &src_map);
            }
            _ => unimplemented!(),
        };
    }

    res
}

fn collect_module_items(
    db: &RootDb,
    module_id: ModuleId,
    decl: ast::ModuleDeclaration,
    res: &mut Vec<DocumentSymbol>,
) {
    let (module, src_map) = db.module_with_source_map(module_id);
    let header = decl.header();
    let mut module_sym = build(module.name.clone(), decl, None, None);

    let children = &mut Vec::default();
    let cont_name = module.name.clone().map(|it| it.to_string());
    let cont_name = cont_name.as_ref();

    // ports
    if let Some(params) = header.parameters() {
        for decl in params.declarations().children() {
            use ast::ParameterDeclarationBase::*;
            match decl {
                ParameterDeclaration(param) => {
                    let decls = param.declarators();
                    build_decls(children, decls, cont_name, &module, &src_map);
                }
                TypeParameterDeclaration(_) => unimplemented!(),
            }
        }
    }

    if let Some(PortList::AnsiPortList(port_list)) = header.ports() {
        for port in port_list.ports().children() {
            use ast::Member::*;
            match port {
                ImplicitAnsiPort(port) => {
                    let decl = port.declarator();
                    let hir = DeclaratorSrc::from(decl).hir(&module, &src_map);
                    let sym = build(hir.name.clone(), decl, cont_name.cloned(), None);
                    children.push(sym);
                }
                ExplicitAnsiPort(_port) => unimplemented!(),
                _ => unreachable!(),
            }
        }
    }

    for member in decl.members().children() {
        use ast::Member::*;
        match member {
            ContinuousAssign(_) => {}
            DataDeclaration(data_decl) => {
                build_decls(children, data_decl.declarators(), cont_name, &module, &src_map);
            }
            NetDeclaration(net_decl) => {
                build_decls(children, net_decl.declarators(), cont_name, &module, &src_map);
            }
            ParameterDeclarationStatement(param_decl) => {
                use ast::ParameterDeclarationBase::*;
                match param_decl.parameter() {
                    ParameterDeclaration(param) => {
                        build_decls(children, param.declarators(), cont_name, &module, &src_map);
                    }
                    TypeParameterDeclaration(_) => unimplemented!(),
                }
            }
            HierarchyInstantiation(instantiation) => {
                for instance in instantiation.instances().children() {
                    let hir = InstanceSrc::from(instance).hir(&module, &src_map);
                    let sym = build(hir.name.clone(), instance, cont_name.cloned(), None);
                    children.push(sym);
                }
            }
            FunctionDeclaration(_fn_decl) => todo!(),
            ProceduralBlock(proc) => {
                let stmt = proc.statement();
                build_stmt(db, children, stmt, cont_name, &module, &src_map);
            }
            // Ports
            PortDeclaration(port) => {
                build_decls(children, port.declarators(), cont_name, &module, &src_map);
            }
            _ => unimplemented!("unhandled member: {:?}", member.syntax().kind()),
        }
    }

    if !children.is_empty() {
        module_sym.children = Some(std::mem::take(children));
    }
    res.push(module_sym);
}

fn collect_block_items(
    db: &RootDb,
    block_id: BlockId,
    decl: ast::BlockStatement,
    cont_name: Option<String>,
    res: &mut Vec<DocumentSymbol>,
) {
    let (block, src_map) = db.block_with_source_map(block_id);
    let mut block_sym = build(block.name.clone(), decl, cont_name, None);

    let children = &mut Vec::default();
    let cont_name = block.name.clone().map(|it| it.to_string());
    let cont_name = cont_name.as_ref();

    for node in decl.items().children() {
        match_ast! { node.syntax(),
            ast::Statement[it] => build_stmt(db, res, it, cont_name, &block, &src_map),
            ast::DataDeclaration[it] => {
                build_decls(children, it.declarators(), cont_name, &block, &src_map);
            },
            _ => unimplemented!("{:?}", node.syntax().kind()),
        }
    }

    if !children.is_empty() {
        block_sym.children = Some(std::mem::take(children));
    }
    res.push(block_sym);
}

fn build_stmt<'a, Arn, SrcMap>(
    db: &RootDb,
    res: &mut Vec<DocumentSymbol>,
    stmt: ast::Statement<'a>,
    container_name: Option<&String>,
    arena: &'a Arc<Arn>,
    src_map: &'a Arc<SrcMap>,
) where
    Arn: GetRef<StmtId, Output = Stmt> + GetRef<LocalBlockId, Output = BlockInfo>,
    SrcMap: Get<StmtSrc, Output = StmtId> + Get<BlockSrc, Output = LocalBlockId>,
{
    if stmt.name().is_some() {
        let hir = StmtSrc::from(stmt).hir(arena, src_map);
        let sym = build(hir.label.clone(), stmt, container_name.cloned(), None);
        res.push(sym);
    }

    use ast::Statement::*;
    match stmt {
        TimingControlStatement(stmt) => {
            build_stmt(db, res, stmt.statement(), container_name, arena, src_map);
        }

        WaitStatement(stmt) => {
            build_stmt(db, res, stmt.statement(), container_name, arena, src_map);
        }

        ConditionalStatement(stmt) => {
            build_stmt(db, res, stmt.statement(), container_name, arena, src_map);
            if let Some(stmt) =
                stmt.else_clause().and_then(|clause| ast::Statement::cast(clause.clause().syntax()))
            {
                build_stmt(db, res, stmt, container_name, arena, src_map);
            }
        }
        CaseStatement(stmt) => {
            for item in stmt.items().children() {
                use ast::CaseItem::*;
                match item {
                    StandardCaseItem(item) => {
                        if let Some(stmt) = ast::Statement::cast(item.clause().syntax()) {
                            build_stmt(db, res, stmt, container_name, arena, src_map);
                        }
                    }
                    DefaultCaseItem(item) => {
                        if let Some(stmt) = ast::Statement::cast(item.clause().syntax()) {
                            build_stmt(db, res, stmt, container_name, arena, src_map);
                        }
                    }
                    PatternCaseItem(_) => unimplemented!(),
                }
            }
        }

        DoWhileStatement(stmt) => {
            build_stmt(db, res, stmt.statement(), container_name, arena, src_map);
        }
        ForeverStatement(stmt) => {
            build_stmt(db, res, stmt.statement(), container_name, arena, src_map);
        }
        LoopStatement(stmt) => {
            build_stmt(db, res, stmt.statement(), container_name, arena, src_map);
        }
        ForLoopStatement(stmt) => {
            build_stmt(db, res, stmt.statement(), container_name, arena, src_map);
        }

        BlockStatement(stmt) => {
            let hir = BlockSrc::from(stmt).hir(arena, src_map);
            collect_block_items(db, hir.block_id, stmt, container_name.cloned(), res);
        }

        ProceduralAssignStatement(_)
        | ProceduralDeassignStatement(_)
        | DisableStatement(_)
        | ReturnStatement(_)
        | JumpStatement(_)
        | ExpressionStatement(_) => {}
        _ => unimplemented!("{:?}", stmt.syntax().kind()),
    };
}

#[inline]
fn build_decls<'a, Arn, SrcMap>(
    res: &mut Vec<DocumentSymbol>,
    decls: ast::SeparatedList<'a, ast::Declarator<'a>>,
    container_name: Option<&String>,
    arena: &'a Arc<Arn>,
    src_map: &'a Arc<SrcMap>,
) where
    Arn: GetRef<DeclId, Output = Declarator>,
    SrcMap: Get<DeclaratorSrc, Output = DeclId>,
{
    for decl in decls.children() {
        let hir = DeclaratorSrc::from(decl).hir(arena, src_map);
        let sym = build(hir.name.clone(), decl, container_name.cloned(), None);
        res.push(sym);
    }
}

#[inline]
fn build<'a>(
    name: Option<SmolStr>,
    node: impl HasName<'a>,
    container_name: Option<String>,
    detail: Option<String>,
) -> DocumentSymbol {
    let focus_range = node
        .name()
        .and_then(|name| name.text_range())
        .unwrap_or_else(|| node.syntax().text_range().unwrap());
    DocumentSymbol {
        name: name.unwrap_or(DEFAULT_NAME).to_string(),
        focus_range,
        full_range: node.syntax().text_range().unwrap(),
        kind: SymbolKind::from_node(node.syntax()),
        detail,
        container_name,
        children: None,
    }
}
