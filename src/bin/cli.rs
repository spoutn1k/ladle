use chopstick::get;
use chopstick::models::{Ingredient, Recipe, Tag};
#[macro_use]
extern crate clap;

static BASE_URL: &str = "http://localhost:8000";

fn main() {
    let matches = clap_app!(Chopstick =>
        (version: "0.0")
        (author: "JBS <jb.skutnik@gmail.com>")
        (about: "Get data from a knife server")
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
        )
        (@subcommand ingredient =>
            (about: "access ingredients")
            (@subcommand list =>
                (about: "list ingredients")
            )
        )
        (@subcommand tag =>
            (about: "Label-related commands")
            (@subcommand list =>
                (about: "List existing labels and their ID")
            )
            (@subcommand show =>
                (about: "List recipes tagged with a given label")
                (@arg id: +required "ID of the label to display")
            )
        )
    )
    .get_matches();

    // Same as before...
    match matches.subcommand() {
        ("recipe", Some(sub_m)) => recipe_actions(&sub_m),
        ("ingredient", Some(sub_m)) => ingredient_actions(&sub_m),
        ("tag", Some(sub_m)) => tag_actions(&sub_m),
        _ => {
            println!("{}", matches.usage())
        }
    }
}

fn recipe_actions(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("list", Some(sub_m)) => recipe_list(sub_m.value_of("pattern")),
        ("show", Some(sub_m)) => recipe_show(sub_m.value_of("id")),
        _ => {
            println!("{}", matches.usage())
        }
    }
}

fn recipe_list(_pattern: Option<&str>) {
    match get::<Vec<Recipe>>(&format!("{}/recipes", BASE_URL)) {
        Ok(recipes) => {
            recipes
                .iter()
                .filter(|recipe| _pattern.is_none() || recipe.name.contains(_pattern.unwrap()))
                .map(|x| println!("{}\t{}", x.id, x.name))
                .for_each(drop);
        }
        Err(e) => {
            eprintln!("{:?}", e);
        }
    }
}

fn recipe_show(_id: Option<&str>) {
    match _id {
        Some(data) => match get::<Recipe>(&format!("{}/recipes/{}", BASE_URL, data)) {
            Ok(recipe) => {
                println!("{:?}", recipe)
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        },
        None => {}
    }
}

fn ingredient_actions(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("list", Some(sub_m)) => ingredient_list(sub_m.value_of("pattern")),
        _ => {
            println!("{}", matches.usage())
        }
    }
}

fn ingredient_list(_pattern: Option<&str>) {
    match get::<Vec<Ingredient>>(&format!("{}/ingredients", BASE_URL)) {
        Ok(ingredients) => {
            ingredients
                .iter()
                .map(|x| println!("{}\t{}", x.id, x.name))
                .for_each(drop);
        }
        Err(e) => {
            eprintln!("{:?}", e);
        }
    }
}

fn tag_actions(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("list", Some(_)) => match get::<Vec<Recipe>>(&format!("{}/labels", BASE_URL)) {
            Ok(recipes) => {
                recipes
                    .iter()
                    .map(|x| println!("{}\t{}", x.id, x.name))
                    .for_each(drop);
            }
            Err(e) => eprintln!("{:?}", e),
        },
        ("show", Some(sub_m)) => tag_show(sub_m.value_of("id")),
        _ => {
            println!("{}", matches.usage())
        }
    }
}

fn tag_show(_id: Option<&str>) {
    match _id {
        Some(data) => match get::<Tag>(&format!("{}/labels/{}", BASE_URL, data)) {
            Ok(label) => label
                .tagged_recipes
                .iter()
                .map(|x| println!("{}\t{}", x.id, x.name))
                .for_each(drop),
            Err(e) => {
                eprintln!("{:?}", e);
            }
        },
        None => {}
    }
}
