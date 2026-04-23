use rustc_hash::FxHashSet;
use utils::get::GetRef;

use crate::{
    container::{ContainerId, ContainerParent, InContainer, InModule, InSubroutine},
    db::HirDb,
    hir_def::{
        Ident,
        aggregate::StructId,
        declaration::Declaration,
        expr::{
            Expr, ExprId,
            data_ty::{BuiltinDataTyId, DataTy, NamedDataTy},
            declarator::{DeclId, DeclaratorParent},
        },
        module::{ModuleId, port::PortDeclId},
        stmt::{ForInit, StmtKind},
        subroutine::SubroutinePortId,
        typedef::TypedefId,
    },
    scope::UnitEntry,
    semantics::pathres::PathResolution,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinTy {
    Data(BuiltinDataTyId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Unknown,
    Error,
    Void,
    Builtin(BuiltinTy),
    Struct(InContainer<StructId>),
    Alias { typedef: InContainer<TypedefId>, target: Box<Ty> },
    Module(ModuleId),
    Block(crate::hir_def::block::BlockId),
}

#[derive(Debug, Clone)]
pub struct TyResult {
    pub ty: Ty,
    pub diagnostics: Vec<TyInferDiagnostic>,
}

impl TyResult {
    fn new(ty: Ty) -> Self {
        TyResult { ty, diagnostics: Vec::new() }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TyInferDiagnostic {
    TypedefCycle(InContainer<TypedefId>),
}

#[derive(Debug, Clone)]
pub struct TyMember {
    pub name: Ident,
    pub ty: Ty,
    pub origin: Option<PathResolution>,
}

pub fn normalize_data_ty(db: &dyn HirDb, container: ContainerId, data_ty: DataTy) -> TyResult {
    normalize_data_ty_inner(db, container, data_ty, &mut FxHashSet::default())
}

pub fn type_of_typedef(db: &dyn HirDb, typedef: InContainer<TypedefId>) -> TyResult {
    type_of_typedef_inner(db, typedef, &mut FxHashSet::default())
}

pub fn type_of_decl(db: &dyn HirDb, decl: InContainer<DeclId>) -> TyResult {
    let Some(data_ty) = data_ty_of_decl(db, decl) else {
        return TyResult::new(Ty::Unknown);
    };
    normalize_data_ty(db, decl.cont_id, data_ty)
}

pub fn type_of_path_resolution(db: &dyn HirDb, res: PathResolution) -> TyResult {
    match res {
        PathResolution::Module(module_id) => TyResult::new(Ty::Module(module_id)),
        PathResolution::Decl(decl) => type_of_decl(db, decl),
        PathResolution::Typedef(typedef) => type_of_typedef(db, typedef),
        PathResolution::ParamDecl(decl) | PathResolution::AnsiPort(decl) => {
            type_of_decl(db, decl.into())
        }
        PathResolution::NonAnsiPort { port_decl, data_decl, module, .. } => data_decl
            .or(port_decl)
            .map(|decl| type_of_decl(db, InContainer::new(module.into(), decl)))
            .unwrap_or_else(|| TyResult::new(Ty::Unknown)),
        PathResolution::SubroutinePort(port) => type_of_subroutine_port(db, port),
        PathResolution::Instance(instance) => {
            instance_target_module_id(db, instance.module_id, instance.value)
                .map(|module_id| TyResult::new(Ty::Module(module_id)))
                .unwrap_or_else(|| TyResult::new(Ty::Unknown))
        }
        PathResolution::Block(block_id) => TyResult::new(Ty::Block(block_id)),
        PathResolution::Subroutine(_) | PathResolution::Stmt(_) => TyResult::new(Ty::Unknown),
    }
}

pub fn type_of_expr(db: &dyn HirDb, expr: InContainer<ExprId>) -> TyResult {
    let Some(hir_expr) = expr_of(db, expr) else {
        return TyResult::new(Ty::Unknown);
    };

    match hir_expr {
        Expr::Ident(ident) => resolve_name(db, expr.cont_id, &ident)
            .map(|res| type_of_path_resolution(db, res))
            .unwrap_or_else(|| TyResult::new(Ty::Unknown)),
        Expr::Field { receiver, field } => {
            let Some(field) = field else {
                return TyResult::new(Ty::Unknown);
            };
            let base = type_of_expr(db, expr.with_value(receiver));
            if matches!(base.ty, Ty::Unknown | Ty::Error) {
                return base;
            }
            let mut selected = select_member(db, &base.ty, &field);
            selected.diagnostics.extend(base.diagnostics);
            selected
        }
        Expr::ElementSelect { receiver, .. } => type_of_expr(db, expr.with_value(receiver)),
        Expr::Cast { ty, .. } => normalize_data_ty(db, expr.cont_id, ty),
        _ => TyResult::new(Ty::Unknown),
    }
}

pub fn members_of_ty(db: &dyn HirDb, ty: &Ty) -> Vec<TyMember> {
    match ty {
        Ty::Alias { target, .. } => members_of_ty(db, target),
        Ty::Struct(struct_id) => struct_members(db, *struct_id),
        Ty::Module(module_id) => module_members(db, *module_id),
        Ty::Block(block_id) => block_members(db, *block_id),
        Ty::Unknown | Ty::Error | Ty::Void | Ty::Builtin(_) => Vec::new(),
    }
}

pub fn select_member(db: &dyn HirDb, base: &Ty, name: &Ident) -> TyResult {
    members_of_ty(db, base)
        .into_iter()
        .find(|member| &member.name == name)
        .map(|member| TyResult::new(member.ty))
        .unwrap_or_else(|| TyResult::new(Ty::Unknown))
}

fn normalize_data_ty_inner(
    db: &dyn HirDb,
    container: ContainerId,
    data_ty: DataTy,
    seen: &mut FxHashSet<InContainer<TypedefId>>,
) -> TyResult {
    match data_ty {
        DataTy::Builtin(builtin) => {
            if matches!(
                db.lookup_intern_ty(builtin),
                crate::hir_def::expr::data_ty::BuiltinDataTy::Void
            ) {
                TyResult::new(Ty::Void)
            } else {
                TyResult::new(Ty::Builtin(BuiltinTy::Data(builtin)))
            }
        }
        DataTy::Struct(struct_id) => TyResult::new(Ty::Struct(struct_id)),
        DataTy::Named(named) => type_of_named_data_ty(db, container, named, seen),
    }
}

fn type_of_named_data_ty(
    db: &dyn HirDb,
    container: ContainerId,
    named: NamedDataTy,
    seen: &mut FxHashSet<InContainer<TypedefId>>,
) -> TyResult {
    let expr_id = match named {
        NamedDataTy::Ident(expr_id) | NamedDataTy::Field(expr_id) => expr_id,
    };
    let Some(Expr::Ident(ident)) = expr_of(db, InContainer::new(container, expr_id)) else {
        return TyResult::new(Ty::Unknown);
    };

    match resolve_name(db, container, &ident) {
        Some(PathResolution::Typedef(typedef)) => type_of_typedef_inner(db, typedef, seen),
        Some(res) => type_of_path_resolution(db, res),
        None => TyResult::new(Ty::Unknown),
    }
}

fn type_of_typedef_inner(
    db: &dyn HirDb,
    typedef: InContainer<TypedefId>,
    seen: &mut FxHashSet<InContainer<TypedefId>>,
) -> TyResult {
    if !seen.insert(typedef) {
        return TyResult {
            ty: Ty::Error,
            diagnostics: vec![TyInferDiagnostic::TypedefCycle(typedef)],
        };
    }

    let Some(def) = typedef_of(db, typedef) else {
        seen.remove(&typedef);
        return TyResult::new(Ty::Unknown);
    };
    let Some(data_ty) = def.ty else {
        seen.remove(&typedef);
        return TyResult::new(Ty::Unknown);
    };

    let mut target = normalize_data_ty_inner(db, typedef.cont_id, data_ty, seen);
    seen.remove(&typedef);
    let ty = if matches!(target.ty, Ty::Error) {
        Ty::Error
    } else {
        Ty::Alias { typedef, target: Box::new(target.ty) }
    };
    TyResult { ty, diagnostics: std::mem::take(&mut target.diagnostics) }
}

fn struct_members(db: &dyn HirDb, struct_id: InContainer<StructId>) -> Vec<TyMember> {
    let Some(def) = struct_of(db, struct_id) else {
        return Vec::new();
    };

    def.members
        .iter()
        .filter_map(|member| {
            let name = member.name.clone()?;
            let ty = member
                .ty
                .map(|ty| normalize_data_ty(db, ty.cont_id, ty.value).ty)
                .unwrap_or(Ty::Unknown);
            Some(TyMember { name, ty, origin: None })
        })
        .collect()
}

fn module_members(db: &dyn HirDb, module_id: ModuleId) -> Vec<TyMember> {
    let mut members: Vec<_> = db
        .module_scope(module_id)
        .iter()
        .map(|(name, entry)| {
            let origin = PathResolution::from(InModule::new(module_id, entry));
            let ty = type_of_path_resolution(db, origin).ty;
            TyMember { name: name.clone(), ty, origin: Some(origin) }
        })
        .collect();
    sort_members(&mut members);
    members
}

fn block_members(db: &dyn HirDb, block_id: crate::hir_def::block::BlockId) -> Vec<TyMember> {
    let mut members: Vec<_> = db
        .block_scope(block_id)
        .iter()
        .map(|(name, entry)| {
            let origin = PathResolution::from(crate::container::InBlock::new(block_id, entry));
            let ty = type_of_path_resolution(db, origin).ty;
            TyMember { name: name.clone(), ty, origin: Some(origin) }
        })
        .collect();
    sort_members(&mut members);
    members
}

fn sort_members(members: &mut Vec<TyMember>) {
    members.sort_by(|left, right| left.name.cmp(&right.name));
    members.dedup_by(|left, right| left.name == right.name);
}

fn data_ty_of_decl(db: &dyn HirDb, decl: InContainer<DeclId>) -> Option<DataTy> {
    let declarator = decl_of(db, decl)?;
    match declarator.parent {
        DeclaratorParent::DeclarationId(declaration_id) => {
            Some(declaration_of(db, decl.with_value(declaration_id))?.ty())
        }
        DeclaratorParent::PortDeclId(port_decl_id) => port_decl_ty(db, decl.cont_id, port_decl_id),
        DeclaratorParent::StmtId(stmt_id) => {
            for_init_decl_ty(db, decl.cont_id, stmt_id, decl.value)
        }
    }
}

fn port_decl_ty(db: &dyn HirDb, cont_id: ContainerId, port_decl_id: PortDeclId) -> Option<DataTy> {
    let ContainerId::ModuleId(module_id) = cont_id else {
        return None;
    };
    let module = db.module(module_id);
    Some(module.ports.get(port_decl_id).header.ty())
}

fn for_init_decl_ty(
    db: &dyn HirDb,
    cont_id: ContainerId,
    stmt_id: crate::hir_def::stmt::StmtId,
    decl_id: DeclId,
) -> Option<DataTy> {
    let stmt = stmt_of(db, InContainer::new(cont_id, stmt_id))?;
    let StmtKind::For { inits: ForInit::Init(inits), .. } = &stmt.kind else {
        return None;
    };
    inits.iter().find_map(|(ty, decl)| (*decl == decl_id).then_some(*ty))
}

fn type_of_subroutine_port(db: &dyn HirDb, port: InSubroutine<SubroutinePortId>) -> TyResult {
    let subroutine = db.subroutine(port.subroutine);
    let port_id = port;
    let Some(port) = subroutine.ports.get(port_id.value.0 as usize) else {
        return TyResult::new(Ty::Unknown);
    };
    port.ty
        .map(|ty| normalize_data_ty(db, ContainerId::SubroutineId(port_id.subroutine), ty))
        .unwrap_or_else(|| TyResult::new(Ty::Unknown))
}

fn resolve_name(db: &dyn HirDb, cont_id: ContainerId, ident: &Ident) -> Option<PathResolution> {
    ContainerParent::start_from(db, cont_id).find_map(|id| match id {
        ContainerId::HirFileId(_) => db.unit_scope().get(ident).map(PathResolution::from),
        ContainerId::ModuleId(module_id) => db
            .module_scope(module_id)
            .get(ident)
            .map(|entry| PathResolution::from(InModule::new(module_id, entry))),
        ContainerId::BlockId(block_id) => db
            .block_scope(block_id)
            .get(ident)
            .map(|entry| PathResolution::from(crate::container::InBlock::new(block_id, entry))),
        ContainerId::SubroutineId(subroutine_id) => db
            .subroutine_scope(subroutine_id)
            .get(ident)
            .map(|entry| PathResolution::from(InSubroutine::new(subroutine_id, entry))),
    })
}

fn instance_target_module_id(
    db: &dyn HirDb,
    module_id: ModuleId,
    instance_id: crate::hir_def::module::instantiation::InstanceId,
) -> Option<ModuleId> {
    let module = db.module(module_id);
    let instance = module.get(instance_id);
    let instantiation = module.get(instance.parent);
    let module_name = instantiation.module_name.as_ref()?;
    match db.unit_scope().get(module_name)? {
        UnitEntry::ModuleId(module_id) => Some(module_id),
        _ => None,
    }
}

fn expr_of(db: &dyn HirDb, expr: InContainer<ExprId>) -> Option<Expr> {
    match expr.cont_id {
        ContainerId::HirFileId(file_id) => Some(db.hir_file(file_id).get(expr.value).clone()),
        ContainerId::ModuleId(module_id) => Some(db.module(module_id).get(expr.value).clone()),
        ContainerId::BlockId(block_id) => Some(db.block(block_id).get(expr.value).clone()),
        ContainerId::SubroutineId(subroutine_id) => {
            Some(db.subroutine(subroutine_id).get(expr.value).clone())
        }
    }
}

fn decl_of(
    db: &dyn HirDb,
    decl: InContainer<DeclId>,
) -> Option<crate::hir_def::expr::declarator::Declarator> {
    match decl.cont_id {
        ContainerId::HirFileId(file_id) => Some(db.hir_file(file_id).get(decl.value).clone()),
        ContainerId::ModuleId(module_id) => Some(db.module(module_id).get(decl.value).clone()),
        ContainerId::BlockId(block_id) => Some(db.block(block_id).get(decl.value).clone()),
        ContainerId::SubroutineId(subroutine_id) => {
            Some(db.subroutine(subroutine_id).get(decl.value).clone())
        }
    }
}

fn declaration_of(
    db: &dyn HirDb,
    decl: InContainer<crate::hir_def::declaration::DeclarationId>,
) -> Option<Declaration> {
    match decl.cont_id {
        ContainerId::HirFileId(file_id) => Some(db.hir_file(file_id).get(decl.value).clone()),
        ContainerId::ModuleId(module_id) => Some(db.module(module_id).get(decl.value).clone()),
        ContainerId::BlockId(block_id) => Some(db.block(block_id).get(decl.value).clone()),
        ContainerId::SubroutineId(subroutine_id) => {
            Some(db.subroutine(subroutine_id).get(decl.value).clone())
        }
    }
}

fn typedef_of(
    db: &dyn HirDb,
    typedef: InContainer<TypedefId>,
) -> Option<crate::hir_def::typedef::Typedef> {
    match typedef.cont_id {
        ContainerId::HirFileId(file_id) => Some(db.hir_file(file_id).get(typedef.value).clone()),
        ContainerId::ModuleId(module_id) => Some(db.module(module_id).get(typedef.value).clone()),
        ContainerId::BlockId(block_id) => Some(db.block(block_id).get(typedef.value).clone()),
        ContainerId::SubroutineId(subroutine_id) => {
            Some(db.subroutine(subroutine_id).get(typedef.value).clone())
        }
    }
}

fn struct_of(
    db: &dyn HirDb,
    struct_id: InContainer<StructId>,
) -> Option<crate::hir_def::aggregate::StructDef> {
    match struct_id.cont_id {
        ContainerId::HirFileId(file_id) => Some(db.hir_file(file_id).get(struct_id.value).clone()),
        ContainerId::ModuleId(module_id) => Some(db.module(module_id).get(struct_id.value).clone()),
        ContainerId::BlockId(block_id) => Some(db.block(block_id).get(struct_id.value).clone()),
        ContainerId::SubroutineId(subroutine_id) => {
            Some(db.subroutine(subroutine_id).get(struct_id.value).clone())
        }
    }
}

fn stmt_of(
    db: &dyn HirDb,
    stmt: InContainer<crate::hir_def::stmt::StmtId>,
) -> Option<crate::hir_def::stmt::Stmt> {
    match stmt.cont_id {
        ContainerId::HirFileId(file_id) => Some(db.hir_file(file_id).get(stmt.value).clone()),
        ContainerId::ModuleId(module_id) => Some(db.module(module_id).get(stmt.value).clone()),
        ContainerId::BlockId(block_id) => Some(db.block(block_id).get(stmt.value).clone()),
        ContainerId::SubroutineId(subroutine_id) => {
            Some(db.subroutine(subroutine_id).get(stmt.value).clone())
        }
    }
}
