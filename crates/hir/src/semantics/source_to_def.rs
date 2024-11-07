use rustc_hash::FxHashMap;
use syntax::{
    SyntaxAncestors, SyntaxNode,
    ast::{self, AstNode},
    match_ast,
};
use utils::get::{Get, GetRef};

use crate::{
    container::{ContainerId, InFile},
    db::HirDb,
    file::HirFileId,
    hir_def::{
        block::{BlockId, BlockSrc},
        module::{ModuleId, ModuleSrc},
    },
    source_map::ToAstNode,
};

#[derive(Default, Debug)]
pub(super) struct Source2DefCache<'db> {
    find_container: FxHashMap<(HirFileId, SyntaxNode<'db>), ContainerId>,
}

pub(super) struct Source2DefCtx<'db, 'cache> {
    pub(super) db: &'db dyn HirDb,
    pub(super) cache: &'cache mut Source2DefCache<'db>,
}

impl Source2DefCtx<'_, '_> {
    pub(super) fn module_to_def(
        &mut self,
        InFile { file_id, value: src }: InFile<ModuleSrc>,
    ) -> Option<ModuleId> {
        let (_, file_source_map) = self.db.hir_file_with_source_map(file_id);
        Some(ModuleId::new(file_id, file_source_map.get(src)))
    }

    pub(super) fn block_to_def(
        &mut self,
        InFile { file_id, value: block_src }: InFile<BlockSrc>,
    ) -> Option<BlockId> {
        let tree = self.db.parse(file_id);
        let node = block_src.to_node(&tree)?;
        self.block_to_def_inner(file_id, node, block_src)
    }

    // This is a faster version of block_to_def that doesn't require a [`to_node`]
    fn block_to_def_inner(
        &mut self,
        file_id: HirFileId,
        block: ast::BlockStatement,
        block_src: BlockSrc,
    ) -> Option<BlockId> {
        let node = block.syntax();
        let container = self.find_container(InFile::new(file_id, node));

        let block_id = match container {
            ContainerId::HirFileId(file_id) => {
                let (file, file_src_map) = self.db.hir_file_with_source_map(file_id);
                let local_block_id = file_src_map.get(block_src);
                file.get(local_block_id).block_id
            }
            ContainerId::ModuleId(module_id) => {
                let (module, module_src_map) = self.db.module_with_source_map(module_id);
                let local_block_id = module_src_map.get(block_src);
                module.get(local_block_id).block_id
            }
            ContainerId::BlockId(block_id) => {
                let (block, block_src_map) = self.db.block_with_source_map(block_id);
                let local_block_id = block_src_map.get(block_src);
                block.get(local_block_id).block_id
            }
        };

        Some(block_id)
    }

    fn container_to_def(&mut self, file_id: HirFileId, node: SyntaxNode) -> Option<ContainerId> {
        let cont_id = match_ast! { node,
            ast::ModuleDeclaration[module] => {
                let src = module.into();
                self.module_to_def(InFile::new(file_id, src))?.into()
            },
            ast::BlockStatement[block] => {
                let block_src = BlockSrc::from(block);
                self.block_to_def_inner(file_id, block, block_src)?.into()
            },
            ast::CompilationUnit => file_id.into(),
            _ => return None,
        };

        Some(cont_id)
    }

    pub(super) fn find_container(
        &mut self,
        InFile { value: node, file_id }: InFile<SyntaxNode>,
    ) -> ContainerId {
        let node = unsafe { std::mem::transmute::<SyntaxNode<'_>, SyntaxNode<'_>>(node) };

        if let Some(container_id) = self.cache.find_container.get(&(file_id, node)) {
            return *container_id;
        }

        let container_id = SyntaxAncestors::start_from(node)
            .skip(1) // skip the node itself
            .find_map(|node| self.container_to_def(file_id, node))
            .unwrap_or(file_id.into());
        self.cache.find_container.insert((file_id, node), container_id);
        container_id
    }
}
