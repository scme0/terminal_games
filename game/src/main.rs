mod game;

use std::io;
use crossterm::{ErrorKind, Result};
use flexi_logger::{FileSpec, FlexiLoggerError, Logger};
use log::{info};

fn main() -> Result<()> {
    let logger_result = Logger::try_with_str("info");
    match logger_result {
        Ok(logger) => {
            let start_result =
                logger.log_to_file(FileSpec::default().suppress_timestamp())
                    .start();
            if let Err(e) = start_result {
                handle_flexi_logger_error(e)?;
            }
        }
        Err(e) => handle_flexi_logger_error(e)?,
    }

    info!("*** Minesweeper 0.0.1 ***");
    game::start()
}

fn handle_flexi_logger_error(error: FlexiLoggerError) -> Result<()>{
    Err(ErrorKind::new(io::ErrorKind::Other, error))
}
