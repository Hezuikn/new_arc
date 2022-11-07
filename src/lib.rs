//MysqlConnection
pub use diesel::connection::SimpleConnection;

#[cfg(feature = "sqlite")]
pub use diesel::SqliteConnection as DbConnection;
#[cfg(feature = "mysql")]
pub use diesel::MysqlConnection as DbConnection;

#[cfg(not(any(feature = "sqlite", feature = "mysql")))]
compile_error!("choose sql:\n--features sqlite\n--features mysql");

pub use diesel::Connection;

pub use diesel::sql_types::*;
pub use diesel::{FromSqlRow, Identifiable, Insertable, Queryable, QueryableByName};
pub use libfj::robocraft::{FactoryInfo, FactoryRobotGetInfo, FactoryRobotListInfo};
pub use libfj::robocraft_simple::FactoryAPI;

mod ext {
    use diesel::{query_builder::ReplaceStatement, replace_into, Insertable, Table};

    pub trait ReplaceIntoExt<T>: Insertable<T> {
        fn replace_into(self, table: T) -> ReplaceStatement<T, <Self as Insertable<T>>::Values>
        where
            T: Table,
            Self: Sized,
        {
            replace_into(table).values(self)
        }
    }
    impl<Tab, T: Insertable<Tab>> ReplaceIntoExt<Tab> for T {}
}
pub use ext::ReplaceIntoExt;
