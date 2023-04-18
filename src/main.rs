#![feature(let_chains)]
#![allow(non_snake_case, non_upper_case_globals)]
#![allow(clippy::needless_return)]

use crate::state::build_state;
use crate::state::save_state;
use diesel::RunQueryDsl;
use entities::{DbCubeData, DbMetaData, DbState};
use new_arc::*;
use sql::*;
use std::io::Write;

mod cli;
mod entities;
mod sql;
mod state;

lazy_static::lazy_static! {
    static ref config: cli::CliArgs = cli::parse();
}

macro_rules! verbose {
    () => {
        compile_error!("");
    };
    ($($arg:tt)*) => {
        {
        if config.verbose {
            println!($($arg)*);
        }
        }
    };
}

const DEFAULT_PAGE_SIZE: i64 = 100;
const PERIOD: i64 = 100;

fn main() {
    verbose!("Opening & building database, roboshield be damned");
    let mut db = DbConnection::establish(
        &config.database, //.as_ref().map(|x| x.as_str()).unwrap_or("./rc_archive.db"),
    )
    .unwrap();

    let mode = config.mode.unwrap_or(1);
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
    if mode == 1 {
        //höchte ids runterladen
        download_newer_bots(&mut db, &api)
    }
    if mode == 2 {
        //cubedata download
        download_missing_block_data(&mut db, &api);
    }
    if mode == 3 {
        //pic download
    }
    if mode == 4 {
        //suche existirende bots
    }
    if mode == 5 {
        //cube table
    }
    if mode == 6 {
        search_bots(&mut db, &mut state, &api);
    }
    verbose!("Done.");
}

fn download_newer_bots(db: &mut DbConnection, api: &FactoryAPI) {
    let latest_bot_row = get_latest_bot(db);
    if let Some(highest_bot) = latest_bot_row {
        let highest_id = highest_bot.id;

        for id in highest_id + 1.. {
            if let Ok(response) = api.get(id.try_into().unwrap()) {
                if !persist_bot(db, response) {
                    break;
                }
            }
        }
    }
}

fn download_missing_block_data(db: &mut DbConnection, api: &FactoryAPI) {
    let missing_bots = get_Bots_without_block_data(db);

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
    let latest_bot_row = get_latest_bot(db);
    //let oldest_cube_row = get_oldest_cube(db);

    if let Some(highest_bot) = latest_bot_row {
        let highest_id = highest_bot.id;
        // NOTE: IDs are gone through sequentially instead of just retrieving the known ones
        // because the default user cannot search for non-buyable robots, despite them existing.
        // This creates gaps in known (i.e. searchable) IDs, despite IDs being sequential.
        for id in highest_id + 1.. {
            if let Ok(response) = api.get(id.try_into().unwrap()) {
                if !persist_bot(db, response) {
                    break;
                }
            }
        }
    }
}

///search for bots in factory that are in a page
/// gets entities::ROBOT_METADATA::dsl::ROBOT_METADATA
fn search_bots(db: &mut DbConnection, state: &mut DbState, api: &FactoryAPI) {
    //get bots lowest first
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

    //get all pages newer than atm page
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
            state.next_page = 0;
            save_state(db, state);
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

        // prepare for next loop iteration (search for next page)
        state.next_page += 1;
        save_state(db, state);
        req_builder = req_builder.page(state.next_page.try_into().unwrap());
    }
}

/// handles bot downloads
/// outputs if request isnt failed
fn persist_bot(db: &mut DbConnection, response: FactoryInfo<FactoryRobotGetInfo>) -> bool {
    if response.status_code != 200 {
        eprintln!(
            "Got response status {}, self-destructing...",
            response.status_code
        );
        return false;
    }

    let robo_data = response.response;
    if config.verbose {
        println!(
            "Found new robot #{} (`{}` by {}, {} CPU)",
            robo_data.item_id, robo_data.item_name, robo_data.added_by_display_name, robo_data.cpu
        );
    }

    //put bot metadata in sql
    DbMetaData::from(robo_data.clone())
        .replace_into(entities::ROBOT_METADATA::dsl::ROBOT_METADATA)
        .execute(db)
        .unwrap();
    //put bot cube data in sql
    DbCubeData::from(robo_data)
        .replace_into(entities::ROBOT_CUBES::dsl::ROBOT_CUBES)
        .execute(db)
        .unwrap();

    return true;
}
