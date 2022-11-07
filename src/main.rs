#![feature(let_chains)]
#![allow(non_snake_case, non_upper_case_globals)]
#![allow(clippy::needless_return)]

use diesel::{sql_query, OptionalExtension, RunQueryDsl};
use entities::{DbCubeData, DbMetaData, DbState};
use itertools::Itertools;
use new_arc::*;
use std::io::Write;

mod cli;
mod entities;

lazy_static::lazy_static! {
    static ref config: cli::CliArgs = cli::parse();
}

macro_rules! verbose {
    () => {
        compile_error!("");
    };
    ($($arg:tt)*) => {{
        if config.verbose {
            println!($($arg)*);
        }
    }};
}

const DEFAULT_PAGE_SIZE: i64 = 100;
const PERIOD: i64 = 100;

fn main() {
    verbose!("Opening & building database, roboshield be damned");
    let mut db = DbConnection::establish(
        &config.database,
        //.as_ref().map(|x| x.as_str()).unwrap_or("./rc_archive.db"),
    )
    .unwrap();

    // build database structure
    entities::build_database(&mut db);
    let mut state = build_state(&mut db);

    save_state(&mut db, &state);

    // begin scraping
    if state.next_page == 0 {
        verbose!("Beginning archival process, looking out for T-sticks");
    } else {
        verbose!(
            "Resuming archival process at page {}, blaming Josh",
            state.next_page
        );
    }
    let api = FactoryAPI::new();
    search_bots(&mut db, &mut state, &api);
    if config.known {
        verbose!("Downloading robot cubes data for all known robots");
        download_missing_bots(&mut db, &api);
    } else {
        verbose!("Looking for non-searchable bots, activating windowmaker module");
        download_all_bots(&mut db, &mut state, &api);
    }
    verbose!("Done.");
}

fn persist_bot(db: &mut DbConnection, response: FactoryInfo<FactoryRobotGetInfo>) -> bool {
    if response.status_code != 200 {
        eprintln!(
            "Got response status {}, self-destructing...",
            response.status_code
        );
        return false;
    }
    let robo_data = response.response;
    if config.new {
        if config.verbose {
            println!(
                "Found new robot #{} (`{}` by {}, {} CPU)",
                robo_data.item_id, robo_data.item_name, robo_data.added_by_display_name, robo_data.cpu
            );
        } else {
            println!("Found new robot #{}", robo_data.item_id);
        }
    }
    DbMetaData::from(robo_data.clone())
        .replace_into(entities::ROBOT_METADATA::dsl::ROBOT_METADATA)
        .execute(db)
        .unwrap();

    DbCubeData::from(robo_data)
        .replace_into(entities::ROBOT_CUBES::dsl::ROBOT_CUBES)
        .execute(db)
        .unwrap();

    return true;
}

fn download_missing_bots(db: &mut DbConnection, api: &FactoryAPI) {
    let missing_bots = sql_query("SELECT * FROM ROBOT_METADATA rm WHERE rm.id NOT IN (SELECT id from ROBOT_CUBES rc);")
        .load::<DbMetaData>(db)
        .unwrap();
    verbose!(
        "Found {} robots which need their cubes downloaded",
        missing_bots.len()
    );
    for bot in missing_bots {
        let response = api.get(bot.id.try_into().unwrap()).unwrap();
        persist_bot(db, response);
    }
}

fn download_all_bots(db: &mut DbConnection, state: &mut DbState, api: &FactoryAPI) {
    let latest_bot_row = sql_query("SELECT * from ROBOT_METADATA rm ORDER BY rm.id DESC LIMIT 1;")
        .load::<DbMetaData>(db)
        .unwrap()
        .into_iter()
        .at_most_one()
        .unwrap();
    let latest_cube_row = sql_query("SELECT * from ROBOT_CUBES rc ORDER BY rc.id DESC LIMIT 1;")
        .load::<DbCubeData>(db)
        .unwrap()
        .into_iter()
        .at_most_one()
        .unwrap();
    let oldest_cube_row = sql_query("SELECT * from ROBOT_CUBES rc ORDER BY rc.id ASC LIMIT 1;")
        .load::<DbCubeData>(db)
        .unwrap()
        .into_iter()
        .at_most_one()
        .unwrap();

    if let Some(highest_bot) = latest_bot_row {
        let highest_id = highest_bot.id;
        let highest_cube_id = if let Some(highest_cubes) = latest_cube_row {
            highest_cubes.id
        } else {
            0
        };
        let lowest_cube_id = if let Some(lowest_cubes) = oldest_cube_row {
            lowest_cubes.id
        } else {
            i64::MAX
        };

        if state.last_sequential_id >= u32::MAX.into() {
            state.last_sequential_id = highest_id;
        }

        // NOTE: IDs are gone through sequentially instead of just retrieving the known ones
        // because the default user cannot search for non-buyable robots, despite them existing.
        // This creates gaps in known (i.e. searchable) IDs, despite IDs being sequential.
        if config.new {
            for id in highest_cube_id + 1..=highest_id {
                if let Ok(response) = api.get(id.try_into().unwrap()) {
                    if !persist_bot(db, response) {
                        break;
                    }
                }
            }
        } else {
            verbose!(
                "Most recent bot has id #{}, existing data for #{} (ignoring down to #{}) to #{}",
                highest_id,
                state.last_sequential_id,
                lowest_cube_id,
                highest_cube_id
            );
            for id in (0..=highest_id).rev() {
                if id <= highest_cube_id && id >= state.last_sequential_id {
                    continue;
                }
                if let Ok(response) = api.get(id.try_into().unwrap()) {
                    if !persist_bot(db, response) {
                        break;
                    }
                    if state.last_sequential_id - id >= PERIOD {
                        state.last_sequential_id = id - (id % PERIOD) + PERIOD;
                        save_state(db, state);
                    }
                }
                if id % PERIOD == 0 {
                    verbose!(
                        "Done bot #{}, last persistent id #{}",
                        id,
                        state.last_sequential_id
                    );
                }
            }
        }
    } else {
        eprintln!("No robots in database, cannot brute-force IDs!");
    }
}

fn search_bots(db: &mut DbConnection, state: &mut DbState, api: &FactoryAPI) {
    let mut req_builder = api
        .list_builder()
        .page(state.next_page.try_into().unwrap())
        .no_minimum_cpu()
        .no_maximum_cpu()
        .order(libfj::robocraft::FactoryOrderType::Added)
        .movement_raw("100000,200000,300000,400000,500000,600000,700000,800000,900000,1000000,1100000,1200000".to_owned())
        .weapon_raw("10000000,20000000,25000000,30000000,40000000,50000000,60000000,65000000,70100000,75000000".to_owned())
        .default_page(false)
        .items_per_page(state.last_page_size.try_into().unwrap());
    loop {
        if config.verbose {
            print!("Retrieving page {}", state.next_page);
            std::io::stdout().flush().unwrap();
        }
        let response = req_builder.clone().send().unwrap();
        if response.status_code != 200 {
            eprintln!(
                "Got response status {}, self-destructing...",
                response.status_code
            );
            break;
        }
        if response.response.roboshop_items.is_empty() {
            verbose!("... Got response page with no items, search has been defeated!");
            break;
        }

        verbose!(
            "... Got {} robots (beep boop)",
            response.response.roboshop_items.len()
        );

        db.transaction::<_, diesel::result::Error, _>(|db| {
            for robot in response.response.roboshop_items {
                DbMetaData::from(robot)
                    .replace_into(entities::ROBOT_METADATA::dsl::ROBOT_METADATA)
                    .execute(db)?;
            }
            Ok(())
        })
        .unwrap();
        // prepare for next loop iteration
        state.next_page += 1;
        save_state(db, state);
        if config.new {
            verbose!("Stopping search before older robots are found");
            break;
        }
        req_builder = req_builder.page(state.next_page.try_into().unwrap());
    }
}

fn save_state(db: &mut DbConnection, state: &DbState) {
    state
        .replace_into(entities::STATE::dsl::STATE)
        .execute(db)
        .unwrap();
}

fn build_state(db: &mut DbConnection) -> DbState {
    let contruct = |last_page_size| DbState {
        last_page_size,
        id: 0,
        next_page: 0,
        last_sequential_id: i32::MAX as _,
    };
    if config.new || config.known {
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
