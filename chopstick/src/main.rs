mod ingredient_actions;
mod label_actions;
mod maintenance_actions;
mod recipe_actions;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::error::Error;
use std::fmt;

#[macro_use]
extern crate clap;

#[derive(Debug)]
struct ChopstickError(String);

impl fmt::Display for ChopstickError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ChopstickError {}

#[tokio::main]
async fn main() {
    let matches = clap_app!(Chopstick =>
        (version: "0.1")
        (author: "JBS <jb.skutnik@gmail.com>")
        (about: "Get data from a knife server")
        (@arg verbose: -v --verbose "Enable debug log")
        (@arg server: -s --server +takes_value "Remote knife server URL")
        (@subcommand remote =>
            (about: "maintenance")
            (@subcommand clone =>
                (about: "clone recipes")
                (@arg remote: +required "URL of the destination")
            )
            (@subcommand clean =>
                (about: "delete unused ingredients and labels")
            )
        )
        (@subcommand recipe =>
            (about: "access recipes")
            (@subcommand list =>
                (about: "list recipes")
                (@arg pattern: "list recipes matching a pattern")
            )
            (@subcommand show =>
                (about: "show details about a recipe")
                (@arg recipe: +required "target recipe id or name")
            )
            (@subcommand create =>
                (about: "create a new recipe")
                (@arg name: +required "target recipe name")
            )
            (@subcommand edit =>
                (about: "edit a recipe")
                (@arg recipe: +required "target recipe id or name")
                (@arg name: -n --name +takes_value "new recipe name")
                (@arg author: -a --author +takes_value "new recipe author")
                (@arg directions: -d --directions +takes_value "new recipe directions")
            )
            (@subcommand delete =>
                (about: "delete a recipe")
                (@arg recipe: +required "target recipe id or name")
            )
            (@subcommand tag =>
                (about: "manage recipe tags")
                (@subcommand add =>
                    (about: "tag a recipe")
                    (@arg recipe: +required "target recipe id or name")
                    (@arg label: +required "label to tag with")
                )
                (@subcommand delete =>
                    (about: "remove a tag from a recipe")
                    (@arg recipe: +required "target recipe id or name")
                    (@arg label: +required "label to untag")
                )
            )
            (@subcommand requirement =>
                (about: "edit recipe requirements")
                (@subcommand create =>
                    (about: "add a requirement to a recipe")
                    (@arg recipe: +required "target recipe id or name")
                    (@arg ingredient: +required "required ingredient id or name")
                    (@arg quantity: +required "required quantity")
                )
                (@subcommand update =>
                    (about: "update a requirement to a recipe")
                    (@arg recipe: +required "target recipe id or name")
                    (@arg ingredient: +required "required ingredient id or name")
                    (@arg quantity: +required "required quantity")
                )
                (@subcommand delete =>
                    (about: "delete a requirement from a recipe")
                    (@arg recipe: +required "target recipe id or name")
                    (@arg ingredient: +required "required ingredient id or name")
                )
            )
            (@subcommand dependency =>
                (about: "edit recipe dependencies")
                (@subcommand create =>
                    (about: "add a dependency to a recipe")
                    (@arg recipe: +required "target recipe id or name")
                    (@arg required: +required "required recipe id or name")
                )
                (@subcommand delete =>
                    (about: "remove a dependency from a recipe")
                    (@arg recipe: +required "target recipe id or name")
                    (@arg required: +required "required recipe id or name")
                )
            )
        )
        (@subcommand ingredient =>
            (about: "Ingredients-related commands")
            (@subcommand list =>
                (about: "List existing ingredients")
                (@arg pattern: "List only ingredients with names matching the given pattern")
            )
            (@subcommand show =>
                (about: "show details about an ingredient")
                (@arg ingredient: +required "target ingredient id or name")
            )
            (@subcommand create =>
                (about: "create an ingredient")
                (@arg name: +required "target ingredient name")
            )
            (@subcommand edit =>
                (about: "edit an ingredient")
                (@arg ingredient: +required "target ingredient id or name")
                (@arg name: -n --name +takes_value "new ingredient name")
            )
            (@subcommand delete =>
                (about: "delete an ingredient")
                (@arg ingredient: +required "target ingredient id or name")
            )
            (@subcommand merge =>
                (about: "merge two ingredients")
                (@arg target: +required "target ingredient id or name")
                (@arg obsolete: +required "target ingredient id or name")
            )
        )
        (@subcommand label =>
            (about: "Label-related commands")
            (@subcommand list =>
                (about: "list existing labels")
                (@arg pattern: "list labels whose name match the given pattern")
            )
            (@subcommand show =>
                (about: "list recipes tagged with a given label")
                (@arg label: +required "target label id or name")
            )
            (@subcommand create =>
                (about: "create a label")
                (@arg name: +required "target label name")
            )
            (@subcommand edit =>
                (about: "edit a label")
                (@arg label: +required "target label id or name")
                (@arg name: -n --name +takes_value "new label name")
            )
            (@subcommand delete =>
                (about: "delete a label")
                (@arg label: +required "target label id or name")
            )
        )
    )
    .get_matches();

    if matches.is_present("verbose") {
        SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .with_module_level("reqwest", LevelFilter::Trace)
            //.with_module_level("chopstick", LevelFilter::Debug)
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

    if let Some(server) = matches.value_of("server") {
        origin = Some(server.to_owned());
    }

    if let Some(server) = origin {
        let server = server.as_str();
        let exec = match matches.subcommand() {
            ("recipe", Some(sub_m)) => recipe_actions::recipe_actions(server, &sub_m).await,
            ("ingredient", Some(sub_m)) => {
                ingredient_actions::ingredient_actions(server, &sub_m).await
            }
            ("label", Some(sub_m)) => label_actions::label_actions(server, &sub_m).await,
            ("remote", Some(sub_m)) => {
                maintenance_actions::maintenance_actions(server, &sub_m).await
            }
            _ => {
                println!("{}", matches.usage());
                Ok(())
            }
        };

        if let Err(message) = exec {
            log::error!("{}", message);
        }
    } else {
        log::error!("Missing parameter: [-s --server] server");
    }
}
