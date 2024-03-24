use crate::hir_def::{
    expr::{ExprId, LowerExpr, MinTypMaxExpr, Path},
    try_match, InFile, SourceMap,
};
use la_arena::{Arena, Idx};
use syntax::ast::{self, ptr};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DelayControl {
    pub expr: MinTypMaxExpr,
}

pub(crate) trait LowerDelayControl: LowerExpr {
    fn lower_delay_control(&mut self, delay_control: &ast::DelayControl) -> Option<DelayControl> {
        let expr = try_match! {
            delay_control.delay_value(), delay_value => {
                self.lower_delay_value(&delay_value)?
            },
            delay_control.mintypmax_expression(), mintypmax_expression => {
                self.lower_min_typ_max_expr(&mintypmax_expression)?
            },
            _ => { return None; }
        };
        Some(DelayControl { expr })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum EventExpr {
    Expr {
        sensitivity: Option<Sensitivity>,
        expr: ExprId,
        // TODO: iff expression
    },
    Or(EventExprId, EventExprId),
    // TODO: sequence_instance
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Sensitivity {
    Posedge,
    Negedge,
    Edge,
}

pub(crate) fn lower_sensitivity(edge_identifier: &ast::EdgeIdentifier) -> Option<Sensitivity> {
    try_match! {
        edge_identifier.token_posedge(), _ => Some(Sensitivity::Posedge),
        edge_identifier.token_negedge(), _ => Some(Sensitivity::Negedge),
        edge_identifier.token_edge(), _ => Some(Sensitivity::Edge),
        _ => None,
    }
}

pub type EventExprId = Idx<EventExpr>;

pub(crate) trait LowerEventExpr: LowerExpr {
    fn arena_event_exprs(&mut self) -> &mut Arena<EventExpr>;

    fn src_map_event_expr(&mut self) -> &mut SourceMap<InFile<ptr::EventExpressionPtr>, EventExpr>;

    fn lower_event_expr(&mut self, event_expr: &ast::EventExpression) -> Option<EventExprId> {
        try_match! {
            // FIXME: support for "iff expression" and "(event_identifier)" is needed
            event_expr.expression(), expr => {
                let sensitivity = event_expr.edge_identifier().and_then(|edge_identifier| lower_sensitivity(&edge_identifier));
                let expr = self.lower_expr(&expr)?;
                let src = self.in_file(event_expr.to_ptr());
                let idx = self.arena_event_exprs().alloc(EventExpr::Expr{sensitivity, expr});
                self.src_map_event_expr().insert(src, idx);
                Some(idx)
            },
            event_expr.token_or(), _ => {
                let lhs = self.lower_event_expr(&event_expr.event_expressions().next()?)?;
                let rhs = self.lower_event_expr(&event_expr.event_expressions().next()?)?;
                let src = self.in_file(event_expr.to_ptr());
                let idx = self.arena_event_exprs().alloc(EventExpr::Or(lhs, rhs));
                self.src_map_event_expr().insert(src, idx);
                Some(idx)
            },
            event_expr.token_comma(), _ => {
                let sensitivity = lower_sensitivity(&event_expr.edge_identifier()?);
                let expr = self.lower_expr(&event_expr.expression()?)?;
                let src = self.in_file(event_expr.to_ptr());
                let idx = self.arena_event_exprs().alloc(EventExpr::Expr{sensitivity, expr});
                self.src_map_event_expr().insert(src, idx);
                Some(idx)
            },
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum EventControl {
    Path(Path),
    EventExpr(Idx<EventExpr>),
    Star,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum DelayOrEventControl {
    DelayControl(DelayControl),
    EventControl(EventControl),
    RepeatControl { expr: ExprId, event_control: EventControl },
}

pub(crate) trait LowerDelayOrEventControl: LowerDelayControl + LowerEventExpr {
    fn lower_delay_or_event_control(
        &mut self,
        control: &ast::DelayOrEventControl,
    ) -> Option<DelayOrEventControl> {
        try_match! {
            control.delay_control(), delay_control => {
                let delay_control = self.lower_delay_control(&delay_control)?;
                Some(DelayOrEventControl::DelayControl(delay_control))
            },
            control.event_control(), event_control => {
                let event_control = self.lower_event_control(&event_control)?;
                Some(DelayOrEventControl::EventControl(event_control))
            },
            control.token_repeat(), _ => {
                let expr = self.lower_expr(&control.expression()?)?;
                let event_control = self.lower_event_control(&control.event_control()?)?;
                Some(DelayOrEventControl::RepeatControl{expr, event_control})
            },
            _ => None,
        }
    }

    fn lower_event_control(&mut self, event_control: &ast::EventControl) -> Option<EventControl> {
        try_match! {
            event_control.hierarchical_event_identifier(), _path => {
                // let path = self.lower_path(&path)?;
                // Some(EventControl::Path(path))
                // TODO: implement hierarchical_event_identifier
                unimplemented!("hierarchical_event_identifier")
            },
            event_control.event_expression(), event_expression => {
                let event_expr = self.lower_event_expr(&event_expression)?;
                Some(EventControl::EventExpr(event_expr))
            },
            event_control.token_star(), _ => Some(EventControl::Star),
            _ => None,
        }
    }
}
