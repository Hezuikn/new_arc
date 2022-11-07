use new_arc::*;

pub fn build_database(db: &mut DbConnection) {
    db.batch_execute(
        "BEGIN;
    CREATE TABLE IF NOT EXISTS ROBOT_METADATA (
        id INTEGER NOT NULL PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT NOT NULL,
        thumbnail TEXT NOT NULL,
        added_by TEXT NOT NULL,
        added_by_display_name TEXT NOT NULL,
        added_date TEXT NOT NULL,
        expiry_date TEXT NOT NULL,
        cpu INTEGER NOT NULL,
        total_robot_ranking INTEGER NOT NULL,
        rent_count INTEGER NOT NULL,
        buy_count INTEGER NOT NULL,
        buyable INTEGER NOT NULL,
        featured INTEGER NOT NULL,
        combat_rating REAL NOT NULL,
        cosmetic_rating REAL NOT NULL
    );
    CREATE TABLE IF NOT EXISTS ROBOT_CUBES (
        id INTEGER NOT NULL PRIMARY KEY,
        cube_data TEXT NOT NULL,
        colour_data LONGTEXT NOT NULL,
        cube_amounts TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS STATE (
        id INTEGER NOT NULL PRIMARY KEY,
        next_page INTEGER NOT NULL,
        last_page_size INTEGER NOT NULL,
        last_sequential_id BIGINT NOT NULL
    );
    COMMIT;",
    )
    .unwrap();
}

#[derive(Clone, Debug, QueryableByName, Queryable, Insertable, macros::Table)]
#[name_table(STATE)]
#[diesel(table_name = STATE)]
pub struct DbState {
    pub id: i64,
    pub next_page: i64,
    pub last_page_size: i64,
    pub last_sequential_id: i64,
}

#[derive(Clone, Debug, QueryableByName, Queryable, Insertable, macros::Table)]
#[name_table(ROBOT_METADATA)]
#[diesel(table_name = ROBOT_METADATA)]
pub struct DbMetaData {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub thumbnail: String,
    pub added_by: String,
    pub added_by_display_name: String,
    pub added_date: String,
    pub expiry_date: String,
    pub cpu: i64,
    pub total_robot_ranking: i64,
    pub rent_count: i64,
    pub buy_count: i64,
    pub buyable: bool,
    pub featured: bool,
    pub combat_rating: f32,
    pub cosmetic_rating: f32,
}

macro_rules! meta_data_selection {
    ($ty:ident) => {
        impl From<$ty> for DbMetaData {
            fn from(other: $ty) -> Self {
                Self {
                    id: other.item_id.try_into().unwrap(),
                    name: other.item_name,
                    description: other.item_description,
                    thumbnail: other.thumbnail,
                    added_by: other.added_by,
                    added_by_display_name: other.added_by_display_name,
                    added_date: other.added_date,
                    expiry_date: other.expiry_date,
                    cpu: other.cpu.try_into().unwrap(),
                    total_robot_ranking: other.total_robot_ranking.try_into().unwrap(),
                    rent_count: other.rent_count.try_into().unwrap(),
                    buy_count: other.buy_count.try_into().unwrap(),
                    buyable: other.buyable,
                    featured: other.featured,
                    combat_rating: other.combat_rating,
                    cosmetic_rating: other.cosmetic_rating,
                }
            }
        }
    };
}

meta_data_selection!(FactoryRobotListInfo);
meta_data_selection!(FactoryRobotGetInfo);

#[derive(Clone, Debug, QueryableByName, Queryable, Insertable, macros::Table)]
#[name_table(ROBOT_CUBES)]
#[diesel(table_name = ROBOT_CUBES)]
pub struct DbCubeData {
    pub id: i64,
    pub cube_data: String,
    pub colour_data: String,
    pub cube_amounts: String,
}

impl From<FactoryRobotGetInfo> for DbCubeData {
    fn from(other: FactoryRobotGetInfo) -> Self {
        Self {
            id: other.item_id.try_into().unwrap(),
            cube_data: other.cube_data,
            colour_data: other.colour_data,
            cube_amounts: other.cube_amounts,
        }
    }
}
