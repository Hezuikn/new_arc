use crate::config;
use crate::entities::DbState;
use crate::DEFAULT_PAGE_SIZE;
use diesel::prelude::*;
use new_arc::DbConnection;
use new_arc::ReplaceIntoExt;
//update state in the database
use crate::entities;
pub fn save_state(db: &mut DbConnection, state: &DbState) {
    state
        .replace_into(entities::STATE::dsl::STATE)
        .execute(db)
        .unwrap();
}

//get status from db
pub fn build_state(db: &mut DbConnection) -> DbState {
    //inline function i think
    //TODO eindeutiger machen
    let contruct = |last_page_size| DbState {
        last_page_size,
        id: 0,
        next_page: 0,
        last_sequential_id: u32::MAX as _,
        last_page_testet: 0,
    };
    if config.new || config.known {
        //100
        contruct(config.size.unwrap_or(DEFAULT_PAGE_SIZE))
    } else if let Some(state) = entities::STATE::dsl::STATE
        .first::<DbState>(db)
        .optional()
        .unwrap()
    {
        if let Some(page_size) = config.size && state.last_page_size != page_size {
            contruct(page_size)
        } else {
            state
        }
    } else {
        contruct(config.size.unwrap_or(DEFAULT_PAGE_SIZE))
    }
}
