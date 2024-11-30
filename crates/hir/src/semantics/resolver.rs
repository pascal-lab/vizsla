use syntax::ast::{self, AstNode};
use utils::get::Get;

use super::SemanticsImpl;
use crate::{
    container::{ContainerId, InFile, InModule},
    hir_def::module::instantiation::{PortConnId, PortConnSrc},
};

impl<'db> SemanticsImpl<'db> {
    pub fn resolve_named_port_conn(&self, conn: ast::PortConnection) -> InModule<PortConnId> {
        let db = self.db;
        let file_id = self.find_file(conn.syntax());
        let ContainerId::ModuleId(module_id) =
            self.find_container(InFile::new(file_id.into(), conn.syntax()))
        else {
            unreachable!("NamedPortConnection should be in a module");
        };

        let src = PortConnSrc::from(conn);
        let (_, module_src_map) = db.module_with_source_map(module_id);
        let conn_id = module_src_map.get(src);
        InModule::new(module_id, conn_id)
    }
}
