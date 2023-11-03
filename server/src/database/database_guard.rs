use super::database::Database;

pub(crate) struct DatabaseGuard {
    db: Database,
}

impl DatabaseGuard {
    pub(crate) fn new() -> Self {
        Self {
            db: Database::new(),
        }
    }
}

impl Drop for DatabaseGuard {
    fn drop(&mut self) {
        todo!()
    }
}
