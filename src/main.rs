mod app;
mod cli;

use app::*;
use cli::cli;
use cli_log::init_cli_log;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    init_cli_log!();
    let cli = cli();
    if *cli
        .get_one("print_default_config")
        .expect("Option has default value")
    {
        app::print_default_config();
        return Ok(());
    }

    let result = App::run(cli);
    ratatui::restore();
    result
}
