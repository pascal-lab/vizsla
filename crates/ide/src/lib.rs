pub use base_db::Cancelled;
pub type Cancellable<T> = Result<T, Cancelled>;

pub mod analysis;
pub mod analysis_host;
