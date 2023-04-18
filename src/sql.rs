use diesel::{sql_query, RunQueryDsl};
use itertools::Itertools;
pub use libfj::robocraft::{FactoryInfo, FactoryRobotGetInfo, FactoryRobotListInfo};
pub use libfj::robocraft_simple::FactoryAPI;
use new_arc::*;

use crate::entities::{DbCubeData, DbMetaData};

pub fn get_Bots_without_block_data(db: &mut DbConnection) -> Vec<DbMetaData> {
    sql_query("SELECT * FROM ROBOT_METADATA rm WHERE rm.id NOT IN (SELECT id from ROBOT_CUBES rc);")
        .load::<DbMetaData>(db)
        .unwrap()
}

/// get bot with highest id
pub fn get_latest_bot(db: &mut DbConnection) -> Option<DbMetaData> {
    sql_query("SELECT * from ROBOT_METADATA rm ORDER BY rm.id DESC LIMIT 1;")
        .load::<DbMetaData>(db)
        .unwrap()
        .into_iter()
        .at_most_one()
        .unwrap()
}

pub fn get_latest_cube(db: &mut DbConnection) -> Option<DbCubeData> {
    sql_query("SELECT * from ROBOT_CUBES rc ORDER BY rc.id DESC LIMIT 1;")
        .load::<DbCubeData>(db)
        .unwrap()
        .into_iter()
        .at_most_one()
        .unwrap()
}
pub fn get_oldest_bot(db: &mut DbConnection) -> Option<DbMetaData> {
    sql_query("SELECT * from ROBOT_METADATA rm ORDER BY rm.id ASC LIMIT 1;")
        .load::<DbMetaData>(db)
        .unwrap()
        .into_iter()
        .at_most_one()
        .unwrap()
}
pub fn get_oldest_cube(db: &mut DbConnection) -> Option<DbCubeData> {
    sql_query("SELECT * from ROBOT_CUBES rc ORDER BY rc.id ASC LIMIT 1;")
        .load::<DbCubeData>(db)
        .unwrap()
        .into_iter()
        .at_most_one()
        .unwrap()
}
