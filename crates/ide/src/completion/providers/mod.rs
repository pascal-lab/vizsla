pub(crate) mod dot_completion;
pub(crate) mod identifier;
pub(crate) mod parameter_list;
pub(crate) mod port_connection;
pub(crate) mod scope_resolution;
pub(crate) mod type_reference;

pub(crate) use dot_completion::complete_dot_access;
pub(crate) use identifier::complete_identifier;
pub(crate) use parameter_list::complete_parameter_list;
pub(crate) use port_connection::complete_port_connection;
pub(crate) use scope_resolution::complete_scope_resolution;
pub(crate) use type_reference::complete_type_reference;
