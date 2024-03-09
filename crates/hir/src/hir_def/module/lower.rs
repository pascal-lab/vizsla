use crate::hir_def::{
    data::{
        DataSubDecl, LocalDataSubDeclSrc, LowerDataSubDecl, LowerDataType, LowerParamDecl,
        ParamDecl,
    },
    expr::{self, LocalSelectSrc, LowerExprSrc, LowerSelectSrc},
    lower::Lower,
    module::{
        port::{NonAnsiPort, PortReference},
        ModuleDecl, ModuleSourceMap,
    },
};
use la_arena::{Arena, ArenaMap, Idx};
use smol_str::SmolStr;
use syntax::ast::{self, AstNode};
use utils::try_;

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
            if let Some(port_decl_list) = ansi_header.list_of_port_declarations() {
                for port_decl in port_decl_list.ansi_port_declarations() {}
            }
        } else if let Some(non_ansi_header) = module_node.module_nonansi_header() {
            if let Some(param_port_list) = non_ansi_header.parameter_port_list() {
                self.lower_param_port_list(&param_port_list);
            }
            try_!({
                let list_of_ports = non_ansi_header.list_of_ports()?;
                for port_node in list_of_ports.ports() {
                    try_!({
                        let ident = port_node
                            .identifier()
                            .and_then(|ident| ident.to_text(self.file_text()).map(SmolStr::from));
                        let port_expr = {
                            let mut arena = Arena::default();
                            for port_ref in port_node.port_expression()?.port_references() {
                                let ident = SmolStr::from(
                                    port_ref.identifier()?.to_text(self.file_text())?,
                                );
                                let select =
                                    self.lower_const_select_src(&port_ref.constant_select()?);
                                arena.alloc(PortReference { ident, select });
                            }
                            arena.shrink_to_fit();
                            arena
                        };
                        self.module_decl.non_ansi_ports.alloc(NonAnsiPort { ident, port_expr });
                    });
                }
            });
        }
    }

    fn lower_param_port_list(&mut self, param_port_list: &ast::ParameterPortList) {
        if let Some(param_assign_list) = param_port_list.list_of_param_assignments() {
            let sub_decls = self.lower_param_sub_decl_list(&param_assign_list);
            self.module_decl.param_port_list.alloc(ParamDecl {
                local: false,
                data_type: None,
                sub_decls,
            });
        }
        for param_port_decl in param_port_list.parameter_port_declarations() {
            try_!({
                if let Some(any_param_decl) = param_port_decl.any_parameter_declaration() {
                    let any_param_decl = self.lower_any_param_decl(&any_param_decl)?;
                    self.module_decl.param_port_list.alloc(any_param_decl);
                } else if let Some(data_type) = param_port_decl.data_type() {
                    let data_type = self.lower_data_type(&data_type);
                    let sub_decls = self
                        .lower_param_sub_decl_list(&param_port_decl.list_of_param_assignments()?);
                    self.module_decl.param_port_list.alloc(ParamDecl {
                        local: false,
                        data_type: Some(data_type),
                        sub_decls,
                    });
                } else if param_port_decl.token_type().is_some() {
                    unimplemented!("parameter_port_declaration ::= type list_of_type_assignments");
                }
            });
        }
    }
}

impl Lower for ModuleLowerCtx<'_> {
    fn file_text(&self) -> &str {
        self.file_text
    }
}

impl LowerExprSrc for ModuleLowerCtx<'_> {
    fn arena_expr_srcs(&mut self) -> &mut Arena<expr::LocalExprSrc> {
        &mut self.module_source_map.expr_srcs
    }
}

impl LowerSelectSrc for ModuleLowerCtx<'_> {
    fn arena_select_srcs(&mut self) -> &mut Arena<LocalSelectSrc> {
        &mut self.module_source_map.select_srcs
    }
}

impl LowerDataSubDecl for ModuleLowerCtx<'_> {
    fn arena_data_sub_decls(&mut self) -> &mut Arena<DataSubDecl> {
        &mut self.module_decl.data.data_sub_decls
    }

    fn src_map_data_sub_decls(&mut self) -> &mut ArenaMap<Idx<DataSubDecl>, LocalDataSubDeclSrc> {
        &mut self.module_source_map.data_sub_decls
    }
}

impl LowerDataType for ModuleLowerCtx<'_> {}

impl LowerParamDecl for ModuleLowerCtx<'_> {}
