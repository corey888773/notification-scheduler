use crate::data::db::DbContext;

pub struct AppState{
    db: DbContext
}

impl AppState{
    pub fn new(db: DbContext) -> Self {
        AppState{db}
    }
}