#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TaskDecl {}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FunctionDecl {}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum TFDecl {
    TaskDecl(TaskDecl),
    FunctionDecl(FunctionDecl),
}
