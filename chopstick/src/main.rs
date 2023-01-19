mod ingredient_actions;
mod label_actions;
mod maintenance_actions;
mod recipe_actions;

use clap::{Parser, Subcommand};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct ChopstickError(String);

impl fmt::Display for ChopstickError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ChopstickError {}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Server URL to contact
    #[arg(short, long)]
    server: Option<String>,

    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    Recipe {
        #[command(subcommand)]
        recipe: recipe_actions::RecipeSubCommands,
    },

    Ingredient {
        #[command(subcommand)]
        ingredient: ingredient_actions::IngredientSubCommands,
    },

    Label {
        #[command(subcommand)]
        label: label_actions::LabelSubCommands,
    },

    Maintenance {
        #[command(subcommand)]
        maintenance: maintenance_actions::MaintenanceSubCommands,
    },
}

#[tokio::main]
async fn main() {
    let matches = Cli::parse();

    if matches.verbose == 1 {
        SimpleLogger::new()
            .with_module_level("ladle", LevelFilter::Debug)
            .with_module_level("chopstick", LevelFilter::Debug)
            .init()
            .unwrap();
    } else {
        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .init()
            .unwrap();
    }

    let mut origin: Option<String> = None;

    if let Some(mut home) = dirs::home_dir() {
        home.push(".config");
        home.push("chopstick");
        home.set_extension("toml");
        match config::Config::builder()
            .add_source(config::File::with_name(home.to_str().unwrap()))
            .build()
        {
            Ok(settings) => match settings.get::<String>("default_remote") {
                Ok(server) => origin = Some(server),
                Err(message) => log::debug!("{:?}", message),
            },
            Err(message) => log::debug!("{:?}", message),
        }
    }

    if let Some(server) = matches.server {
        origin = Some(server.to_owned());
    }

    if let Some(server) = origin {
        let server = server.as_str();
        let exec = match matches.command {
            Subcommands::Recipe { recipe } => recipe_actions::actions(server, recipe).await,
            Subcommands::Ingredient { ingredient } => {
                ingredient_actions::actions(server, ingredient).await
            }
            Subcommands::Label { label } => label_actions::actions(server, label).await,
            Subcommands::Maintenance { maintenance } => {
                maintenance_actions::actions(server, maintenance).await
            }
        };

        if let Err(message) = exec {
            log::error!("{}", message);
        }
    } else {
        log::error!("Missing parameter: [-s --server] server");
    }
}
