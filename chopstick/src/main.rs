mod ingredient_actions;
mod label_actions;
mod maintenance_actions;
mod recipe_actions;

use log::LevelFilter;
use simple_logger::SimpleLogger;

#[macro_use]
extern crate clap;

static BASE_URL: &str = "http://localhost:8000";

#[tokio::main]
async fn main() {
    let matches = clap_app!(Chopstick =>
        (version: "0.1")
        (author: "JBS <jb.skutnik@gmail.com>")
        (about: "Get data from a knife server")
        (@arg verbose: -v --verbose "Enable debug log")
        (@subcommand remote =>
            (about: "maintenance")
            (@subcommand clone =>
                (about: "clone recipes")
                (@arg remote: +required "URL of the destination")
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
                (@arg id: +required "target recipe id")
            )
            (@subcommand create =>
                (about: "create a new recipe")
                (@arg name: +required "target recipe name")
            )
            (@subcommand edit =>
                (about: "edit a recipe")
                (@arg id: +required "target recipe id")
                (@arg name: -n --name +takes_value "new recipe name")
                (@arg author: -a --author +takes_value "new recipe author")
                (@arg description: -d --description +takes_value "new recipe description")
            )
            (@subcommand delete =>
                (about: "delete a recipe")
                (@arg id: +required "target recipe id")
            )
            (@subcommand requirement =>
                (about: "edit recipe requirements")
                (@subcommand create =>
                    (about: "add a requirement to a recipe")
                    (@arg id: +required "target recipe id")
                    (@arg ingredient_id: +required "required ingredient id")
                    (@arg quantity: +required "required quantity")
                )
                (@subcommand update =>
                    (about: "update a requirement to a recipe")
                    (@arg id: +required "target recipe id")
                    (@arg ingredient_id: +required "required ingredient id")
                    (@arg quantity: +required "required quantity")
                )
                (@subcommand delete =>
                    (about: "delete a requirement from a recipe")
                    (@arg id: +required "target recipe id")
                    (@arg ingredient_id: +required "required ingredient id")
                )
            )
            (@subcommand dependency =>
                (about: "edit recipe dependencies")
                (@subcommand add =>
                    (about: "add a dependency to a recipe")
                    (@arg id: +required "target recipe id")
                    (@arg required_id: +required "required recipe id")
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
                (@arg id: +required "target ingredient id")
            )
            (@subcommand create =>
                (about: "create an ingredient")
                (@arg name: +required "target ingredient name")
            )
            (@subcommand edit =>
                (about: "edit an ingredient")
                (@arg id: +required "target ingredient id")
                (@arg name: -n --name +takes_value "new ingredient name")
            )
            (@subcommand delete =>
                (about: "delete an ingredient")
                (@arg id: +required "target ingredient id")
            )
            (@subcommand merge =>
                (about: "merge two ingredients")
                (@arg target_id: +required "target ingredient id")
                (@arg obsolete_id: +required "target ingredient id")
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
                (@arg id: +required "target label id")
            )
            (@subcommand create =>
                (about: "create an label")
                (@arg name: +required "target label name")
            )
            (@subcommand edit =>
                (about: "edit an label")
                (@arg id: +required "target label id")
                (@arg name: -n --name +takes_value "new label name")
            )
            (@subcommand delete =>
                (about: "delete an label")
                (@arg id: +required "target label id")
            )
        )
    )
    .get_matches();

    if matches.is_present("verbose") {
        SimpleLogger::new()
            .with_level(LevelFilter::Off)
            .with_module_level("ladle", LevelFilter::Debug)
            .init()
            .unwrap();
    }

    let exec = match matches.subcommand() {
        ("recipe", Some(sub_m)) => recipe_actions::recipe_actions(&sub_m),
        ("ingredient", Some(sub_m)) => ingredient_actions::ingredient_actions(&sub_m),
        ("label", Some(sub_m)) => label_actions::label_actions(&sub_m),
        ("remote", Some(sub_m)) => maintenance_actions::maintenance_actions(&sub_m),
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    };

    if let Err(message) = exec {
        eprintln!("{}", message);
    }
}
