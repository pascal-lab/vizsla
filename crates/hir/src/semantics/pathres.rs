use syntax::{
    SyntaxNode, SyntaxToken, SyntaxTokenWithParent,
    ast::{self, AstNode},
};
use utils::get::GetRef;

use super::SemanticsImpl;
use crate::{
    container::{ContainerId, InBlock, InContainer, InFile, InModule, InPackage, InSubroutine},
    hir_def::{
        aggregate::ClassId,
        block::BlockId,
        declaration::Declaration,
        expr::declarator::{DeclId, DeclaratorParent},
        lower_ident_opt,
        module::{ModuleId, instantiation::InstanceId, port::NonAnsiPortId},
        package::PackageId,
        stmt::StmtId,
        subroutine::SubroutineId,
        typedef::TypedefId,
    },
    scope::{
        self, BlockEntry, ModuleEntry, PackageEntry, PackageImportEntry, SubroutineEntry, UnitEntry,
    },
};

impl SemanticsImpl<'_> {
    pub fn nameres_ident(
        &self,
        SyntaxTokenWithParent { parent, tok }: SyntaxTokenWithParent,
    ) -> Option<PathResolution> {
        let file_id = self.find_file(parent);
        let ident = lower_ident_opt(Some(tok))?;
        self.with_ctx(|ctx| {
            let container = ctx.find_container(InFile::new(file_id, parent));
            ctx.name_to_def(InContainer::new(container, ident))
        })
    }

    pub fn nameres_named_port_conn(
        &self,
        conn: ast::NamedPortConnection,
    ) -> Option<PathResolution> {
        let entry = self.nameres_instance_conn(conn.name(), conn.syntax())?;

        if matches!(entry.value, ModuleEntry::AnsiPortEntry(_) | ModuleEntry::NonAnsiPortEntry(_)) {
            Some(entry.into())
        } else {
            None
        }
    }

    pub fn nameres_named_param_assign(
        &self,
        conn: ast::NamedParamAssignment,
    ) -> Option<PathResolution> {
        let entry = self.nameres_instance_conn(conn.name(), conn.syntax())?;
        let module = self.db.module(entry.module_id);
        if let ModuleEntry::DeclId(decl_id) = entry.value
            && let DeclaratorParent::DeclarationId(declaration_id) = module.get(decl_id).parent
            && let Declaration::ParamDecl(_) = module.get(declaration_id)
        {
            Some(entry.into())
        } else {
            None
        }
    }

    fn nameres_instance_conn(
        &self,
        name: Option<SyntaxToken>,
        node: SyntaxNode,
    ) -> Option<InModule<ModuleEntry>> {
        let db = self.db;
        let conn_name = lower_ident_opt(name)?;

        let instantiation = ast::HierarchyInstantiation::cast(node.parent()?.parent()?)?;
        let module_id = self.nameres_instantiation(instantiation)?;

        let module_scope = db.module_scope(module_id);
        let entry = module_scope.get(&conn_name)?;

        Some(InModule::new(module_id, entry))
    }

    pub fn nameres_instantiation(
        &self,
        instantiation: ast::HierarchyInstantiation,
    ) -> Option<ModuleId> {
        let module_name = lower_ident_opt(instantiation.type_())?;
        match self.db.unit_scope().get(&module_name)? {
            UnitEntry::ModuleId(module_id) => Some(module_id),
            UnitEntry::FiledDeclId(_) => None,
            UnitEntry::TypedefId(_) => None,
            UnitEntry::ClassId(_) => None,
            UnitEntry::PackageId(_) => None,
            UnitEntry::SubroutineId(_) => None,
        }
    }

    pub(in crate::semantics) fn find_container(&self, node: InFile<SyntaxNode>) -> ContainerId {
        self.with_ctx(|ctx| ctx.find_container(node))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PathResolution {
    Module(ModuleId),
    Decl(InContainer<DeclId>),
    ParamDecl(InModule<DeclId>),
    Typedef(InContainer<TypedefId>),
    Class(InContainer<ClassId>),
    PackageImport(InModule<PackageImportEntry>),
    NonAnsiPort {
        // There won't be a situation where all fields are None.
        label: Option<NonAnsiPortId>,
        port_decl: Option<DeclId>,
        data_decl: Option<DeclId>,
        module: ModuleId,
    },
    AnsiPort(InModule<DeclId>),
    Instance(InModule<InstanceId>),
    Stmt(InContainer<StmtId>),
    Block(BlockId),
    Package(PackageId),
    Subroutine(InContainer<SubroutineId>),
}

impl From<UnitEntry> for PathResolution {
    fn from(entry: UnitEntry) -> Self {
        use UnitEntry::*;
        match entry {
            ModuleId(idx) => Self::Module(idx),
            FiledDeclId(idx) => Self::Decl(idx.into()),
            TypedefId(idx) => Self::Typedef(idx.into()),
            ClassId(idx) => Self::Class(idx.into()),
            PackageId(idx) => Self::Package(idx),
            SubroutineId(idx) => Self::Subroutine(idx.into()),
        }
    }
}

impl From<InModule<ModuleEntry>> for PathResolution {
    fn from(entry: InModule<ModuleEntry>) -> Self {
        use ModuleEntry::*;
        match entry.value {
            DeclId(decl_id) => Self::Decl(entry.with_value(decl_id).into()),
            InstanceId(idx) => Self::Instance(entry.with_value(idx)),
            StmtId(idx) => Self::Stmt(entry.with_value(idx).into()),
            NonAnsiPortEntry(scope::NonAnsiPortEntry { label, port_decl, data_decl }) => {
                Self::NonAnsiPort { label, port_decl, data_decl, module: entry.module_id }
            }
            AnsiPortEntry(scope::AnsiPortEntry(idx)) => Self::AnsiPort(entry.with_value(idx)),
            BlockId(block_id) => Self::Block(block_id),
            TypedefId(typedef_id) => Self::Typedef(entry.with_value(typedef_id).into()),
            ClassId(class_id) => Self::Class(entry.with_value(class_id).into()),
            PackageImportEntry(import_entry) => Self::PackageImport(entry.with_value(import_entry)),
            PackageMember(pkg_entry) => match pkg_entry {
                PackageEntry::DeclId(in_pkg_decl) => Self::Decl(in_pkg_decl.into()),
                PackageEntry::TypedefId(in_pkg_typedef) => Self::Typedef(in_pkg_typedef.into()),
                PackageEntry::ClassId(in_pkg_class) => Self::Class(in_pkg_class.into()),
                PackageEntry::StructId(_) | PackageEntry::ProcId(_) => unreachable!(),
                PackageEntry::SubroutineId(in_pkg_sub) => Self::Subroutine(in_pkg_sub.into()),
                PackageEntry::Package(in_pkg_pkg) => Self::Package(in_pkg_pkg.value),
            },
            SubroutineId(sub_id) => Self::Subroutine(entry.with_value(sub_id).into()),
        }
    }
}

impl From<InBlock<BlockEntry>> for PathResolution {
    fn from(entry: InBlock<BlockEntry>) -> Self {
        use BlockEntry::*;
        match entry.value {
            DeclId(idx) => Self::Decl(entry.with_value(idx).into()),
            StmtId(idx) => Self::Stmt(entry.with_value(idx).into()),
            BlockId(block_id) => Self::Block(block_id),
            TypedefId(typedef_id) => Self::Typedef(entry.with_value(typedef_id).into()),
        }
    }
}

impl From<InSubroutine<SubroutineEntry>> for PathResolution {
    fn from(entry: InSubroutine<SubroutineEntry>) -> Self {
        use SubroutineEntry::*;
        match entry.value {
            DeclId(idx) => Self::Decl(entry.with_value(idx).into()),
            StmtId(idx) => Self::Stmt(entry.with_value(idx).into()),
            BlockId(block_id) => Self::Block(block_id),
            TypedefId(typedef_id) => Self::Typedef(entry.with_value(typedef_id).into()),
        }
    }
}

impl From<InPackage<PackageEntry>> for PathResolution {
    fn from(entry: InPackage<PackageEntry>) -> Self {
        use PackageEntry::*;
        match entry.value {
            DeclId(in_pkg_decl) => Self::Decl(in_pkg_decl.into()),
            TypedefId(in_pkg_typedef) => Self::Typedef(in_pkg_typedef.into()),
            ClassId(in_pkg_class) => Self::Class(in_pkg_class.into()),
            StructId(_) | ProcId(_) => unreachable!(),
            SubroutineId(in_pkg_sub) => Self::Subroutine(in_pkg_sub.into()),
            Package(in_pkg_pkg) => Self::Package(in_pkg_pkg.value),
        }
    }
}
