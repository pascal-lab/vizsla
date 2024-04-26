use crate::db::HirDb;

mod source_to_def;

pub struct Semantics<'db, DB> {
    db: &'db DB,
}

pub struct SemanticsImpl<'db> {
    db: &'db dyn HirDb,
    // s2d_cache: FxHashMap<()>
}
