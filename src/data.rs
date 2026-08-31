use sqlx::SqlitePool;

#[derive(Debug)]
pub struct Data {
    pool: SqlitePool,
}

/// struct that holds application/user data
impl Data {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
