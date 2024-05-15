//! Error types.

/// An error that can occur when ensuring the association.
#[derive(Debug)]
pub enum DatabaseError {
    /// Error during interacting with DB.
    Diesel(diesel::result::Error),
}
