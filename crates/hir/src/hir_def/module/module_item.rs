use crate::{
    hir_def::{
        data::{Dimension, LowerDataDecl, LowerDimension, LowerNetDecl, LowerVarDecl},
        expr::{ExprId, LowerExpr},
        lower::Lower,
        module::{
            lower::ModuleLowerCtx,
            port::{LowerPortDecl, PortDecl},
        },
        pack_or_gen_item::{LowerPackOrGenItemDecl, PackOrGenItemDecl},
        try_match, Ident,
    },
    InFile,
};
use la_arena::{Idx, IdxRange, RawIdx};
use smallvec::SmallVec;
use syntax::ast::{self, ptr};
use utils::try_;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModuleItem {
    PortDecl(PortDecl),
    PackOrGenItemDecl(PackOrGenItemDecl),
    ModuleInstantiation(ModuleInstantiation),
    ContinuousAssignment(ContinuousAssignment),
    ProcessConstruct(ProcessConstruct),
    // TODO: Add more module items
    // ParamOverride(Idx<ParamOverride>),
    // InterfaceInstantiation(Idx<InterfaceInstantiation>),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum LocalModuleItemSrc {
    NonePortItem(ptr::NonPortModuleItemPtr),
    PortDecl(ptr::PortDeclarationPtr),
}

pub type ModuleItemSrc = InFile<LocalModuleItemSrc>;

// #[derive(Debug, PartialEq, Eq, Clone, Hash)]
// pub struct ParamOverride {
//     pub hierarchical_ident: HierarchicalIdent,
//     pub expr: NodeId,
//     pub node_id: NodeId,
// }

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModuleInstantiation {
    pub ident: Ident,
    pub param_assigns: ParamAssigns,
    pub hierarchical_instances: IdxRange<HierarchicalInstance>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ParamAssigns {
    Ordered(SmallVec<[ExprId; 1]>),
    Named(SmallVec<[(Ident, Option<ExprId>); 1]>),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct HierarchicalInstance {
    pub ident: Ident,
    pub dimensions: Option<SmallVec<[Dimension; 1]>>,
    pub port_connects: Option<PortConnects>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PortConnects {
    Ordered(SmallVec<[Option<ExprId>; 1]>),
    Named(SmallVec<[(Ident, Option<ExprId>); 1]>),
}

// TODO: net: [drive_strength][delay3] variable: [delay_control]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ContinuousAssignment {
    // TODO: complete this
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum AlwaysType {
    Always,
    AlwaysComb,
    AlwaysLatch,
    AlwaysFf,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ProcessType {
    Initial,
    Always(AlwaysType),
    Final,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ProcessConstruct {
    pub process_type: ProcessType,
    //pub stmt: NodeId,
}

impl<'a> ModuleLowerCtx<'a> {
    pub(crate) fn lower_module_item(&mut self, item: &ast::ModuleItem) {
        try_! {
            try_match! {
                item.port_declaration(), port_decl => {
                    let module_item = ModuleItem::PortDecl(self.lower_port_decl(&port_decl)?);
                    let src = self.in_file(LocalModuleItemSrc::PortDecl(port_decl.to_ptr()));
                    let idx = self.module_decl.module_items.alloc(module_item);
                    self.module_src_map.module_item.insert(src, idx);
                },
                item.non_port_module_item(), non_port_item => {
                    let module_item = self.lower_non_port_module_item(&non_port_item)?;
                    let src = self.in_file(LocalModuleItemSrc::NonePortItem(non_port_item.to_ptr()));
                    let idx = self.module_decl.module_items.alloc(module_item);
                    self.module_src_map.module_item.insert(src, idx);
                },
                _ => { return None; }
            };
        };
    }

    pub(crate) fn lower_non_port_module_item(
        &mut self,
        item: &ast::NonPortModuleItem,
    ) -> Option<ModuleItem> {
        let module_item = try_match! {
            item.module_or_generate_item(), module_or_generate_item => {
                self.lower_module_or_gen_item(&module_or_generate_item)?
            },
            item.generate_region(), _generate_region => {
                unimplemented!("generate_region")
            },
            item.specify_block(), _ => {
                unimplemented!("specify_block")
            },
            item.specparam_declaration(), _ => {
                unimplemented!("specparam_declaration")
            },
            item.program_declaration(), _ => {
                unimplemented!("program_declaration")
            },
            item.module_declaration(), _ => {
                unimplemented!("module_declaration")
            },
            item.interface_declaration(), _ => {
                unimplemented!("interface_declaration")
            },
            item.timeunits_declaration(), _ => {
                unimplemented!("timeunits_declaration")
            },
            _ => { return None; }
        };
        Some(module_item)
    }

    pub(crate) fn lower_module_or_gen_item(
        &mut self,
        item: &ast::ModuleOrGenerateItem,
    ) -> Option<ModuleItem> {
        let module_item = try_match! {
            item.module_common_item(), module_common_item => {
                self.lower_module_common_item(&module_common_item)?
            },
            item.module_instantiation(), module_instantiation => {
                self.lower_module_instantiation(&module_instantiation)?
            },
            item.parameter_override(), _param_override => {
                unimplemented!("parameter_override")
            },
            item.gate_instantiation(), _gate_instantiation => {
                unimplemented!("gate_instantiation")
            },
            item.udp_instantiation(), _udp_instantiation => {
                unimplemented!("udp_instantiation")
            },
            _ => { return None; }
        };
        Some(module_item)
    }

    pub(crate) fn lower_module_common_item(
        &mut self,
        item: &ast::ModuleCommonItem,
    ) -> Option<ModuleItem> {
        let module_item = try_match! {
            item.module_or_generate_item_declaration(), module_or_generate_item_decl => {
                self.lower_module_or_gen_item_decl(&module_or_generate_item_decl)?
            },
            item.continuous_assign(), _ => {
                // ModuleItem::ContinuousAssignment(ContinuousAssignment{

                // })
                unimplemented!()
            },
            item.initial_construct(), _ => {
                unimplemented!()
            },
            item.always_construct(), _ => {
                unimplemented!()
            },
            item.final_construct(), _ => {
                unimplemented!()
            },
            item.interface_instantiation(), _ => {
                unimplemented!("interface_instantiation")
            },
            item.program_instantiation(), _ => {
                unimplemented!("program_instantiation")
            },
            item.assertion_item(), _ => {
                unimplemented!("assertion_item")
            },
            item.bind_directive(), _ => {
                unimplemented!("bind_directive")
            },
            item.net_alias(), _ => {
                unimplemented!("net_alias")
            },
            item.loop_generate_construct(), _ => {
                unimplemented!("loop_generate_construct")
            },
            item.conditional_generate_construct(), _ => {
                unimplemented!("conditional_generate_construct")
            },
            item.elaboration_system_task(), _ => {
                unimplemented!("elaboration_system_task")
            },
            _ => { return None;}
        };
        Some(module_item)
    }

    pub(crate) fn lower_module_or_gen_item_decl(
        &mut self,
        item: &ast::ModuleOrGenerateItemDeclaration,
    ) -> Option<ModuleItem> {
        let module_item = try_match! {
            item.package_or_generate_item_declaration(), pack_or_gen_item_decl => {
                ModuleItem::PackOrGenItemDecl(
                    self.lower_pack_or_gen_item_decl(&pack_or_gen_item_decl)?
                )
            },
            item.genvar_declaration(), _genvar_decl => {
                unimplemented!("genvar_declaration");
            },
            item.clocking_declaration(), _clocking_decl => {
                unimplemented!("clocking_declaration");
            },
            item.token_clocking(), _ => {
                unimplemented!("... ::= default clocking identifier");
            },
            item.token_disable(), _ => {
                unimplemented!("... ::= default disable iff expression_or_dist");
            },
            _ => { return None; },
        };
        Some(module_item)
    }

    fn lower_module_instantiation(
        &mut self,
        module_inst: &ast::ModuleInstantiation,
    ) -> Option<ModuleItem> {
        let ident = self.lower_ident(&module_inst.identifier()?)?;
        let param_value_assigns = module_inst.parameter_value_assignment()?;
        let param_assigns_node = param_value_assigns.list_of_parameter_assignments()?;
        let param_assigns = try_match! {
            param_assigns_node.list_of_ordered_parameter_assignments(), ordered => {
                let mut assigns: SmallVec<[ExprId; 1]> = SmallVec::new();
                for assign in ordered.ordered_parameter_assignments() {
                    assigns.push(self.lower_param_expr(&assign.param_expression()?)?)
                }
                ParamAssigns::Ordered(assigns)
            },
            param_assigns_node.list_of_named_parameter_assignments(), named => {
                let mut assigns: SmallVec<[(Ident, Option<ExprId>); 1]> = SmallVec::new();
                for assign in named.named_parameter_assignments() {
                    let ident = self.lower_ident(&assign.identifier()?)?;
                    let expr = assign.param_expression().map(|param_expr| self.lower_param_expr(&param_expr))?;
                    assigns.push((ident, expr))
                }
                ParamAssigns::Named(assigns)
            },
            _ => { return None; }
        };
        let begin_idx = self.module_decl.data.hierarchical_instances.len();
        let begin_idx = Idx::from_raw(RawIdx::from(begin_idx as u32));
        for instance_node in module_inst.hierarchical_instances() {
            try_! {
                let instance = self.lower_hierarchy_instance(&instance_node)?;
                let src = self.in_file(instance_node.to_ptr());
                let idx = self.module_decl.data.hierarchical_instances.alloc(instance);
                self.module_src_map.hierarchical_instance.insert(src, idx);
            };
        }
        let end_idx = self.module_decl.data.hierarchical_instances.len();
        let end_idx = Idx::from_raw(RawIdx::from(end_idx as u32));
        let hierarchical_instances = IdxRange::new(begin_idx..end_idx);
        Some(ModuleItem::ModuleInstantiation(ModuleInstantiation {
            ident,
            param_assigns,
            hierarchical_instances,
        }))
    }

    fn lower_hierarchy_instance(
        &mut self,
        instance: &ast::HierarchicalInstance,
    ) -> Option<HierarchicalInstance> {
        let name = instance.name_of_instance()?;
        let ident = self.lower_ident(&name.identifier()?)?;
        let mut dimensions: SmallVec<[Dimension; 1]> = SmallVec::new();
        for dim in name.unpacked_dimensions() {
            let dim = self.lower_unpacked_dimension(&dim)?;
            dimensions.push(dim);
        }
        let dimensions = if dimensions.is_empty() { None } else { Some(dimensions) };
        let list = instance.list_of_port_connections()?;
        let port_connects = try_match! {
            list.list_of_ordered_port_connections(), ordered => {
                let mut connects: SmallVec<[Option<ExprId>; 1]> = SmallVec::new();
                for connect in ordered.ordered_port_connections() {
                    connects.push(connect.expression().and_then(|expr| self.lower_expr(&expr)));
                }
                Some(PortConnects::Ordered(connects))
            },
            list.list_of_named_port_connections(), named => {
                let mut connects: SmallVec<[(Ident, Option<ExprId>); 1]> = SmallVec::new();
                for connect in named.named_port_connections() {
                    let ident = self.lower_ident(&connect.identifier()?)?;
                    let expr = connect.expression().and_then(|expr| self.lower_expr(&expr));
                    connects.push((ident, expr));
                }
                Some(PortConnects::Named(connects))
            },
            _ => None,
        };
        Some(HierarchicalInstance { ident, dimensions, port_connects })
    }
}

impl LowerNetDecl for ModuleLowerCtx<'_> {}

impl LowerVarDecl for ModuleLowerCtx<'_> {}

impl LowerDataDecl for ModuleLowerCtx<'_> {}

impl LowerPackOrGenItemDecl for ModuleLowerCtx<'_> {}
