use clap::Parser;
use eyre::Result;

use atuin_client::{
    database::Database,
    database::{current_context, OptFilters},
    settings::Settings
};
use eyre::eyre;
use super::history::ListMode;

mod cursor;
mod duration;
mod event;
mod history_list;
mod interactive;
pub use duration::format_duration;

#[derive(Parser)]
pub struct Cmd {
    /// Filter search result by directory
    #[arg(long, short)]
    cwd: Option<String>,

    /// Exclude directory from results
    #[arg(long = "exclude-cwd")]
    exclude_cwd: Option<String>,

    /// Filter search result by exit code
    #[arg(long, short)]
    exit: Option<i64>,

    /// Exclude results with this exit code
    #[arg(long = "exclude-exit")]
    exclude_exit: Option<i64>,

    /// Only include results added before this date
    #[arg(long, short)]
    before: Option<String>,

    /// Only include results after this date
    #[arg(long)]
    after: Option<String>,

    /// How many entries to return at most
    #[arg(long)]
    limit: Option<i64>,

    /// Open interactive search UI
    #[arg(long, short)]
    interactive: bool,

    /// Use human-readable formatting for time
    #[arg(long)]
    human: bool,

    query: Vec<String>,

    /// Show only the text of the command
    #[arg(long)]
    cmd_only: bool,
}

impl Cmd {
    pub async fn run(self, db: &mut impl Database, settings: &Settings) -> Result<()> {
        if self.interactive {
            let item = interactive::history(&self.query, settings, db).await?;
            eprintln!("{}", item);
        } else {
            let list_mode = ListMode::from_flags(self.human, self.cmd_only);

            let opt_filter = OptFilters {
                exit: self.exit,
                exclude_exit: self.exclude_exit,
                cwd: self.cwd,
                exclude_cwd: self.exclude_cwd,
                before: self.before,
                after: self.after,
                limit: self.limit,
            };

            let entries =
                run_non_interactive(settings, list_mode, opt_filter, &self.query, db).await?;
            if entries == 0 {
                std::process::exit(1)
            }
        };
        Ok(())
    }
}

// This is supposed to more-or-less mirror the command line version, so ofc
// it is going to have a lot of args
#[allow(clippy::too_many_arguments)]
async fn run_non_interactive(
    settings: &Settings,
    list_mode: ListMode,
    filter_options: OptFilters,
    query: &[String],
    db: &mut impl Database,
) -> Result<usize> {
    let dir = if filter_options.cwd.as_deref() == Some(".") {
        let current = std::env::current_dir()?;
        let current = current.as_os_str();
        let current = current.to_str().unwrap();

        Some(current.to_owned())
    } else {
        filter_options.cwd
    };

    let opt_filter = OptFilters {
        cwd: dir,
        ..filter_options
    };

    let context = current_context();
    let query: String = db
    .get_query(
        settings.search_mode,
        settings.filter_mode,
        &context,
        query.join(" ").as_str(),
        opt_filter,
    )
    .unwrap();

    let r = db.get_stream_query(&query, |v|{
        super::history::print_single(&v,list_mode);
    }).await.map_err(|_| eyre!("Query stream failed"))?;

    Ok(r)
}
