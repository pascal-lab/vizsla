use crate::{
    db::HirDb,
    file::{HirFileId, InFile},
    hir_def::{
        block::{BlockId, BlockInfo, BlockItemDecl},
        data::{DataDecl, DataSubDecl},
        module::{
            module_item::{HierarchicalInst, Inst, ModuleItem},
            port::{AnsiPortDecl, PortDecl},
            Module,
        },
        pack_or_gen_item::PackOrGenItemDecl,
        stmt::StmtItem,
        FileItem, Ident, ModuleId,
    },
};
use la_arena::Idx;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use triomphe::Arc;

trait Scope {
    type Entry;

    fn entries(&mut self) -> &mut FxHashMap<Ident, Self::Entry>;

    fn insert_entry(&mut self, ident: Ident, entry: Self::Entry) {
        match self.entries().entry(ident) {
            Entry::Occupied(_) => todo!("diagnostics"),
            Entry::Vacant(e) => e.insert(entry),
        };
    }
}

#[derive(Debug, Clone)]
pub enum BlockScopeOwner {
    Module { id: ModuleId, scope: Arc<ModuleScope> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitScope {
    pub entries: FxHashMap<Ident, UnitScopeEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnitScopeEntry {
    Module(ModuleId),
    //Data(Idx<DataSubDecl>),
    //TF()
}

impl Scope for UnitScope {
    type Entry = UnitScopeEntry;

    fn entries(&mut self) -> &mut FxHashMap<Ident, UnitScopeEntry> {
        &mut self.entries
    }
}

impl UnitScope {
    pub fn unit_scope_query(db: &dyn HirDb) -> Arc<UnitScope> {
        let mut scope = UnitScope { entries: FxHashMap::default() };

        db.files().iter().map(|file_id| HirFileId(*file_id)).for_each(|file_id| {
            scope.collect_hir_file(db, &file_id);
        });

        Arc::new(scope)
    }

    fn collect_hir_file(&mut self, db: &dyn HirDb, file_id: &HirFileId) {
        let hir_file = db.hir_file(*file_id);
        let hir_file = hir_file.as_ref();

        for item in &hir_file.items {
            match item {
                FileItem::Module(module_id) => {
                    let module = &hir_file.data[*module_id];
                    self.insert_entry(
                        module.ident.clone(),
                        UnitScopeEntry::Module(InFile::new(*file_id, *module_id)),
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleScope {
    pub module_id: ModuleId,
    // pub block_scopes: ArenaMap<Idx<Block>, BlockScope>,
    pub entries: FxHashMap<Ident, ModuleScopeEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleScopeEntry {
    Data(Idx<DataSubDecl>),
    NonAnsiPort { port_decl: Idx<DataSubDecl>, data_sub_decl: Option<Idx<DataSubDecl>> },
    ModuleInst(Idx<HierarchicalInst>),
    BlockInfo(Idx<BlockInfo>),
    // TODO?: Stmt(Idx<Stmt>)
    // TODO?: Module(ModuleId)
    // TODO: TF()
}

impl Scope for ModuleScope {
    type Entry = ModuleScopeEntry;

    fn entries(&mut self) -> &mut FxHashMap<Ident, ModuleScopeEntry> {
        &mut self.entries
    }
}

impl ModuleScope {
    pub fn module_scope_query(db: &dyn HirDb, module_id: ModuleId) -> Arc<ModuleScope> {
        let module = db.module(module_id);
        let module = module.as_ref();
        let mut scope = ModuleScope {
            module_id,
            // block_scopes: ArenaMap::default(),
            entries: FxHashMap::default(),
        };

        scope.collect_param_port_list(module);
        scope.collect_ansi_port_decls(module);
        scope.collect_module_items(module);

        Arc::new(scope)
    }

    fn collect_data_sub_decl(&mut self, module: &Module, idx: Idx<DataSubDecl>) {
        let sub_decl = &module[idx];
        let ident = sub_decl.ident.clone();
        let entry = ModuleScopeEntry::Data(idx);
        self.insert_entry(ident, entry);
    }

    fn collect_data_decl(&mut self, module: &Module, idx: Idx<DataDecl>) {
        let data_decl = &module[idx];
        let sub_decls = match data_decl {
            DataDecl::NetDecl(net_decl) => &net_decl.sub_decls,
            DataDecl::ParamDecl(param_decl) => &param_decl.sub_decls,
            DataDecl::VarDecl(var_decl) => &var_decl.sub_decls,
        }
        .clone();
        for idx in sub_decls {
            self.collect_data_sub_decl(module, idx)
        }
    }

    fn collect_param_port_list(&mut self, module: &Module) {
        if let Some(param_port_list) = &module.param_port_list {
            let param_port_list = param_port_list.clone();
            for idx in param_port_list {
                self.collect_data_decl(module, idx)
            }
        }
    }

    fn collect_ansi_port_decls(&mut self, module: &Module) {
        module.ansi_port_decls.iter().for_each(|(_, port)| {
            match port {
                AnsiPortDecl::IODecl(io_decl) => {
                    self.collect_data_sub_decl(module, io_decl.sub_decl)
                }
            };
        });
    }

    fn collect_non_ansi_port_decl(&mut self, module: &Module, idx: Idx<PortDecl>) {
        let port_decl = &module[idx];
        match port_decl {
            PortDecl::IODecl(io_decl) => {
                let sub_decls = io_decl.data_decls.clone();
                for idx in sub_decls {
                    self.collect_data_sub_decl(module, idx)
                }
            }
        }
    }

    fn collect_hierarchy_inst(&mut self, module: &Module, idx: Idx<HierarchicalInst>) {
        let inst = &module[idx];
        let ident = inst.ident.clone();
        let entry = ModuleScopeEntry::ModuleInst(idx);
        self.insert_entry(ident, entry);
    }

    fn collect_module_inst(&mut self, module: &Module, idx: Idx<Inst>) {
        let inst = &module[idx];
        for hinst_idx in inst.hierarchical_insts.clone() {
            self.collect_hierarchy_inst(module, hinst_idx);
        }
    }

    fn collect_module_items(&mut self, module: &Module) {
        module.module_items.iter().for_each(|(_, item)| {
            match item {
                ModuleItem::PortDecl(idx) => {
                    self.collect_non_ansi_port_decl(module, *idx);
                }
                ModuleItem::PackOrGenItemDecl(decl) => match decl {
                    PackOrGenItemDecl::DataDecl(idx) => self.collect_data_decl(module, *idx),
                },
                ModuleItem::ProcessConstruct(pc) => {
                    if let Some(stmt) = pc.stmt {
                        if let StmtItem::BlockInfo(blk) = module[stmt].item {
                            // self.collect_block_scope(module, blk, None);
                            if let Some(ident) = &module[blk].ident {
                                self.insert_entry(ident.clone(), ModuleScopeEntry::BlockInfo(blk));
                            }
                        }
                    }
                }
                ModuleItem::ModuleInst(idx) => {
                    self.collect_module_inst(module, *idx);
                }
                _ => {}
            };
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockScope {
    pub block_id: BlockId,
    pub ident: Option<Ident>,
    // pub parent: Option<Idx<Block>>,
    pub entries: FxHashMap<Ident, BlockScopeEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlockScopeEntry {
    Data(Idx<DataSubDecl>),
    Block(Idx<BlockInfo>),
    // TODO?: Stmt(Idx<Stmt>)
}

impl Scope for BlockScope {
    type Entry = BlockScopeEntry;

    fn entries(&mut self) -> &mut FxHashMap<Ident, BlockScopeEntry> {
        &mut self.entries
    }
}

impl BlockScope {
    pub fn new(block_id: BlockId, ident: Option<Ident>) -> Self {
        BlockScope {
            block_id,
            ident,
            // parent,
            entries: FxHashMap::default(),
        }
    }

    pub fn block_scope_query(db: &dyn HirDb, block_id: BlockId) -> Arc<BlockScope> {
        let block = db.block(block_id);
        let mut block_scope = BlockScope::new(block_id, block.info.ident.clone());

        for item in &block.data.block_item_decls {
            match item {
                BlockItemDecl::DataDecl(idx) => {
                    let data_decl = &block[*idx];
                    let sub_decls = match data_decl {
                        DataDecl::NetDecl(net_decl) => &net_decl.sub_decls,
                        DataDecl::ParamDecl(param_decl) => &param_decl.sub_decls,
                        DataDecl::VarDecl(var_decl) => &var_decl.sub_decls,
                    }
                    .clone();
                    for idx in sub_decls {
                        let sub_decl = &block[idx];
                        let ident = sub_decl.ident.clone();
                        block_scope.insert_entry(ident, BlockScopeEntry::Data(idx));
                    }
                }
            }
        }

        for (_, stmt) in block.data.stmts.iter() {
            if let StmtItem::BlockInfo(idx) = stmt.item {
                if let Some(ident) = &block[idx].ident {
                    block_scope.insert_entry(ident.clone(), BlockScopeEntry::Block(idx));
                }
            }
        }

        Arc::new(block_scope)
    }
}
