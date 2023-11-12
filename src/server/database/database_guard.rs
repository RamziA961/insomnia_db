use super::database::Database;

#[derive(Clone, Debug)]
pub(crate) struct DatabaseGuard {
    db: Database,
}

impl DatabaseGuard {
    pub(crate) fn new() -> Self {
        Self {
            db: Database::new(),
        }
    }

    pub(crate) fn inner(&self) -> Database {
        self.db.clone()
    }
}

impl Drop for DatabaseGuard {
    fn drop(&mut self) {
        self.db.halt_background_tasks();
    }
}
