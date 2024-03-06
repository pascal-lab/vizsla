use crate::hir_def::{
    expr::SelectHolder,
    module::{
        port::{NonAnsiPort, PortReference},
        ModuleDecl, ModuleSourceMap,
    },
};
use la_arena::Arena;
use smol_str::SmolStr;
use syntax::ast::{self, AstNode};

pub(crate) struct ModuleLowerCtx<'a> {
    pub module_decl: &'a mut ModuleDecl,
    pub module_source_map: &'a mut ModuleSourceMap,
    pub file_text: &'a str,
}

impl<'a> ModuleLowerCtx<'a> {
    pub(crate) fn lower_module_decl(&mut self, module_node: &ast::ModuleDeclaration) {
        // TODO: package_import_declaration
        if let Some(ansi_header) = module_node.module_ansi_header() {
            if let Some(param_port_list) = ansi_header.parameter_port_list() {
                self.lower_param_port_list(&param_port_list);
            }
        } else if let Some(non_ansi_header) = module_node.module_nonansi_header() {
            if let Some(param_port_list) = non_ansi_header.parameter_port_list() {
                self.lower_param_port_list(&param_port_list);
            }
            if let Some(list_of_ports) = non_ansi_header.list_of_ports() {
                for port_node in list_of_ports.ports() {
                    (|| {
                        self.module_decl.non_ansi_ports.alloc(NonAnsiPort {
                            ident: port_node
                                .identifier()
                                .and_then(|ident| ident.to_text(self.file_text).map(SmolStr::from)),
                            port_expr: {
                                let mut arena = Arena::default();
                                for port_ref in port_node.port_expression()?.port_references() {
                                    let ident = SmolStr::from(
                                        port_ref.identifier()?.to_text(self.file_text)?,
                                    );
                                    let select = self.module_source_map.selects.alloc(
                                        SelectHolder::ConstSelect(
                                            port_ref.constant_select()?.to_ptr(),
                                        ),
                                    );
                                    arena.alloc(PortReference { ident, select });
                                }
                                arena.shrink_to_fit();
                                arena
                            },
                        });
                        Some(())
                    })();
                }
            }
        }
    }

    fn lower_param_port_list(&mut self, param_port_list: &ast::ParameterPortList) {}
}
