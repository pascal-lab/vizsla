use continuous_assgin::{ContinuousAssign, ContinuousAssignId, ContinuousAssignSrc};
use instantiation::{
    Instance, InstanceId, InstanceSrc, Instantiation, InstantiationId, InstantiationSrc,
    ParamAssign, ParamAssignId, ParamAssignSrc, PortConnection, PortConnectionId,
    PortConnectionSrc,
};
use la_arena::{Arena, Idx};
use port::{
    AnsiPort, AnsiPortId, AnsiPortSrc, NonAnsiPort, NonAnsiPortId, NonAnsiPortSrc, ParamPort,
    ParamPortId, ParamPortSrc, PortDecl, PortDeclId, PortDeclSrc, PortRef, PortRefId, PortRefSrc,
    PortSrcs, Ports,
};
use syntax::ast::{self, AstNode, PortList};
use triomphe::Arc;
use utils::{
    define_enum_deriving_from,
    get::{Get, GetRef},
};

use super::{
    Ident,
    block::{BlockInfo, BlockSrc, LocalBlockId},
    declaration::{
        Declaration, DeclarationId, DeclarationSrc, LowerDeclaration, LowerDeclarationCtx,
    },
    expr::{
        Expr, ExprId, ExprSrc, LowerExpr, LowerExprCtx,
        declarator::{DeclId, Declarator, DeclaratorSrc, LowerDecl, LowerDeclCtx},
        timing_control::{EventExpr, EventExprId, EventExprSrc, LowerEventExpr, LowerEventExprCtx},
    },
    proc::{LowerProc, LowerProcCtx, Proc, ProcId, ProcSrc},
    stmt::{LowerStmt, LowerStmtCtx, Stmt, StmtId, StmtSrc},
    ty::NetKind,
};
use crate::{
    container::InFile,
    db::{HirDb, InternDb},
    define_src,
    file::HirFileId,
    impl_arena_idx, impl_source_map_idx,
    source_map::{SourceMap, ToAstNode},
};

pub mod continuous_assgin;
pub mod instantiation;
pub mod port;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Module {
    pub name: Option<Ident>,
    pub items: Arena<ModuleItem>,

    pub params: Arena<ParamPort>,
    pub ports: Ports,
    pub port_decls: Arena<PortDecl>,

    pub cont_assigns: Arena<ContinuousAssign>,
    pub declarations: Arena<Declaration>,

    pub instantiations: Arena<Instantiation>,
    pub inst_param_assigns: Arena<ParamAssign>,
    pub inst_port_conns: Arena<PortConnection>,
    pub instances: Arena<Instance>,

    pub procs: Arena<Proc>,

    pub exprs: Arena<Expr>,
    pub event_exprs: Arena<EventExpr>,
    pub decls: Arena<Declarator>,
    pub stmts: Arena<Stmt>,
}

define_src!(ModuleSrc(ast::ModuleDeclaration));

impl_arena_idx! { Module =>
    params[ParamPort],
    ports[NonAnsiPort],
    ports[AnsiPort],
    ports[PortRef],
    port_decls[PortDecl],

    items[ModuleItem],
    cont_assigns[ContinuousAssign],
    declarations[Declaration],

    instantiations[Instantiation],
    inst_param_assigns[ParamAssign],
    instances[Instance],
    inst_port_conns[PortConnection],

    procs[Proc],

    exprs[Expr],
    event_exprs[EventExpr],
    decls[Declarator],
    stmts[Stmt],
    stmts[LocalBlockId => BlockInfo],
}

define_enum_deriving_from! {
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum ModuleItem {
        ContinuousAssignId,
        DeclarationId,
        InstantiationId,
        ProcId,
    }
}

impl Module {
    pub fn shrink_to_fit(&mut self) {
        self.params.shrink_to_fit();
        self.ports.shrink_to_fit();
        self.port_decls.shrink_to_fit();

        self.cont_assigns.shrink_to_fit();
        self.declarations.shrink_to_fit();

        self.instantiations.shrink_to_fit();
        self.inst_param_assigns.shrink_to_fit();
        self.instances.shrink_to_fit();
        self.inst_port_conns.shrink_to_fit();

        self.procs.shrink_to_fit();

        self.exprs.shrink_to_fit();
        self.event_exprs.shrink_to_fit();
        self.decls.shrink_to_fit();
        self.stmts.shrink_to_fit();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ModuleInfo {
    pub name: Option<Ident>,
}

pub type LocalModuleId = Idx<ModuleInfo>;
pub type ModuleId = InFile<LocalModuleId>;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ModuleSourceMap {
    pub params: SourceMap<ParamPortSrc, ParamPort>,
    pub ports: PortSrcs,
    pub port_decls: SourceMap<PortDeclSrc, PortDecl>,

    pub cont_assigns: SourceMap<ContinuousAssignSrc, ContinuousAssign>,
    pub declarations: SourceMap<DeclarationSrc, Declaration>,

    pub instantiations: SourceMap<InstantiationSrc, Instantiation>,
    pub inst_param_assigns: SourceMap<ParamAssignSrc, ParamAssign>,
    pub inst_port_conns: SourceMap<PortConnectionSrc, PortConnection>,
    pub instances: SourceMap<InstanceSrc, Instance>,

    pub procs: SourceMap<ProcSrc, Proc>,

    pub exprs: SourceMap<ExprSrc, Expr>,
    pub event_exprs: SourceMap<EventExprSrc, EventExpr>,
    pub decls: SourceMap<DeclaratorSrc, Declarator>,
    pub stmts: SourceMap<StmtSrc, Stmt>,
}

impl_source_map_idx! { ModuleSourceMap =>
    params[ParamPortSrc, ParamPortId],
    port_decls[PortDeclSrc, PortDeclId],
    ports[NonAnsiPortSrc, NonAnsiPortId],
    ports[AnsiPortSrc, AnsiPortId],
    ports[PortRefSrc, PortRefId],

    cont_assigns[ContinuousAssignSrc, ContinuousAssignId],
    declarations[DeclarationSrc, DeclarationId],

    instantiations[InstantiationSrc, InstantiationId],
    inst_param_assigns[ParamAssignSrc, ParamAssignId],
    inst_port_conns[PortConnectionSrc, PortConnectionId],
    instances[InstanceSrc, InstanceId],

    procs[ProcSrc, ProcId],

    exprs[ExprSrc, ExprId],
    event_exprs[EventExprSrc, EventExprId],
    decls[DeclaratorSrc, DeclId],
    stmts[StmtSrc, StmtId],
    stmts[BlockSrc, LocalBlockId],
}

impl ModuleSourceMap {
    pub fn shrink_to_fit(&mut self) {
        self.params.shrink_to_fit();
        self.ports.shrink_to_fit();
        self.port_decls.shrink_to_fit();

        self.cont_assigns.shrink_to_fit();
        self.declarations.shrink_to_fit();

        self.instantiations.shrink_to_fit();
        self.inst_param_assigns.shrink_to_fit();
        self.instances.shrink_to_fit();
        self.inst_port_conns.shrink_to_fit();

        self.procs.shrink_to_fit();

        self.exprs.shrink_to_fit();
        self.event_exprs.shrink_to_fit();
        self.decls.shrink_to_fit();
        self.stmts.shrink_to_fit();
    }
}

pub(crate) struct LowerModuleCtx<'a> {
    pub(crate) db: &'a dyn InternDb,
    pub(crate) file_id: HirFileId,
    pub(crate) module_id: ModuleId,
    pub(crate) default_net_type: Option<NetKind>,

    pub(crate) module: &'a mut Module,
    pub(crate) module_source_map: &'a mut ModuleSourceMap,
}

impl LowerExpr for LowerModuleCtx<'_> {
    fn expr_ctx(&mut self) -> LowerExprCtx {
        LowerExprCtx {
            db: self.db,
            exprs: &mut self.module.exprs,
            expr_srcs: &mut self.module_source_map.exprs,
        }
    }
}

impl LowerDecl for LowerModuleCtx<'_> {
    fn decl_ctx(&mut self) -> LowerDeclCtx {
        LowerDeclCtx {
            db: self.db,
            decls: &mut self.module.decls,
            decl_srcs: &mut self.module_source_map.decls,

            exprs: &mut self.module.exprs,
            expr_srcs: &mut self.module_source_map.exprs,
        }
    }
}

impl LowerEventExpr for LowerModuleCtx<'_> {
    fn event_expr_ctx(&mut self) -> LowerEventExprCtx {
        LowerEventExprCtx {
            db: self.db,
            event_exprs: &mut self.module.event_exprs,
            event_expr_srcs: &mut self.module_source_map.event_exprs,

            exprs: &mut self.module.exprs,
            expr_srcs: &mut self.module_source_map.exprs,
        }
    }
}

impl LowerDeclaration for LowerModuleCtx<'_> {
    fn declaration_ctx(&mut self) -> LowerDeclarationCtx<'_> {
        LowerDeclarationCtx {
            db: self.db,
            declarations: &mut self.module.declarations,
            declaration_srcs: &mut self.module_source_map.declarations,

            decls: &mut self.module.decls,
            decl_srcs: &mut self.module_source_map.decls,

            event_exprs: &mut self.module.event_exprs,
            event_expr_srcs: &mut self.module_source_map.event_exprs,

            exprs: &mut self.module.exprs,
            expr_srcs: &mut self.module_source_map.exprs,
        }
    }
}

impl LowerStmt for LowerModuleCtx<'_> {
    fn stmt_ctx(&mut self) -> LowerStmtCtx<'_> {
        LowerStmtCtx {
            db: self.db,
            file_id: self.file_id,
            cont_id: self.module_id.into(),
            stmts: &mut self.module.stmts,
            stmt_srcs: &mut self.module_source_map.stmts,

            exprs: &mut self.module.exprs,
            expr_srcs: &mut self.module_source_map.exprs,

            event_exprs: &mut self.module.event_exprs,
            event_expr_srcs: &mut self.module_source_map.event_exprs,

            decls: &mut self.module.decls,
            decl_srcs: &mut self.module_source_map.decls,
        }
    }
}

impl LowerProc for LowerModuleCtx<'_> {
    fn proc_ctx(&mut self) -> LowerProcCtx<'_> {
        LowerProcCtx {
            db: self.db,
            file_id: self.file_id,
            cont_id: self.module_id.into(),
            procs: &mut self.module.procs,
            proc_srcs: &mut self.module_source_map.procs,

            stmts: &mut self.module.stmts,
            stmt_srcs: &mut self.module_source_map.stmts,

            exprs: &mut self.module.exprs,
            expr_srcs: &mut self.module_source_map.exprs,

            event_exprs: &mut self.module.event_exprs,
            event_expr_srcs: &mut self.module_source_map.event_exprs,

            decls: &mut self.module.decls,
            decl_srcs: &mut self.module_source_map.decls,
        }
    }
}

impl LowerModuleCtx<'_> {
    pub(crate) fn lower_module_decl(&mut self, decl: ast::ModuleDeclaration) {
        let header = decl.header();
        if let Some(param_ports) = header.parameters() {
            self.lower_param_ports(param_ports);
        }

        match header.ports() {
            Some(PortList::AnsiPortList(port_list)) => self.lower_ansi_ports(port_list),
            Some(PortList::NonAnsiPortList(port_list)) => self.lower_nonansi_port(port_list),
            Some(PortList::WildcardPortList(_)) => unimplemented!(),
            None => {}
        };

        for member in decl.members().children() {
            use ast::Member::*;
            let idx = match member {
                ContinuousAssign(assign) => self.lower_continuous_assign(assign).into(),
                DataDeclaration(data_decl) => {
                    self.declaration_ctx().lower_data_decl(data_decl).into()
                }
                NetDeclaration(net_decl) => self.declaration_ctx().lower_net_decl(net_decl).into(),
                ParameterDeclarationStatement(param_decl) => {
                    self.declaration_ctx().lower_param_decl_stmt(param_decl).into()
                }
                HierarchyInstantiation(instantiation) => {
                    self.lower_instantiation(instantiation).into()
                }
                FunctionDeclaration(_fn_decl) => todo!(),
                ProceduralBlock(proc) => self.proc_ctx().lower_proc(proc).into(),
                // Ports
                PortDeclaration(port) => {
                    self.lower_port_decl(port);
                    continue;
                }
                ExplicitAnsiPort(_) | ImplicitAnsiPort(_) => unreachable!(),
                _ => unimplemented!("unhandled module member: {:?}", member.syntax().kind()),
            };
            self.module.items.alloc(idx);
        }
    }
}

pub(crate) fn module_with_source_map_query(
    db: &dyn HirDb,
    module_id @ InFile { value: local_module_id, cont_id: file_id }: ModuleId,
) -> (Arc<Module>, Arc<ModuleSourceMap>) {
    let (file, file_source_map) = db.hir_file_with_source_map(file_id);
    let tree = db.parse(file_id);

    let mut module = Module { name: file.get(local_module_id).name.clone(), ..Default::default() };
    let mut module_source_map = ModuleSourceMap::default();

    let Some(ast_module) = file_source_map.get(local_module_id).to_node(&tree) else {
        return (Arc::new(module), Arc::new(module_source_map));
    };

    let mut lower_ctx = LowerModuleCtx {
        db,
        default_net_type: Some(NetKind::Wire),
        file_id,
        module_id,
        module: &mut module,
        module_source_map: &mut module_source_map,
    };
    lower_ctx.lower_module_decl(ast_module);

    module.shrink_to_fit();
    module_source_map.shrink_to_fit();
    (Arc::new(module), Arc::new(module_source_map))
}
