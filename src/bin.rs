use chopstick::models::{Label, Recipe};
use chopstick::{get, list_ingredients, list_labels, recipe_index};
use std::collections::HashMap;
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
        )
        (@subcommand ingredient =>
            (about: "Ingredients-related commands")
            (@subcommand list =>
                (about: "List existing ingredients")
                (@arg pattern: "List only ingredients with names matching the given pattern")
            )
        )
        (@subcommand tag =>
            (about: "Label-related commands")
            (@subcommand list =>
                (about: "List existing labels and their ID")
                (@arg pattern: "List labels whose name match the given pattern")
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
        ("create", Some(sub_m)) => recipe_create(sub_m.value_of("name")),
        ("edit", Some(sub_m)) => recipe_edit(
            sub_m.value_of("id"),
            sub_m.value_of("name"),
            sub_m.value_of("author"),
            sub_m.value_of("description"),
        ),
        ("delete", Some(sub_m)) => recipe_delete(sub_m.value_of("id")),
        _ => {
            println!("{}", matches.usage())
        }
    }
}

fn recipe_list(pattern: Option<&str>) {
    if let Some(recipes) = recipe_index(BASE_URL, pattern.unwrap_or("")) {
        recipes
            .iter()
            .map(|x| println!("{}\t{}", x.id, x.name))
            .for_each(drop);
    }
}

fn recipe_show(_id: Option<&str>) {
    if let Some(data) = _id {
        match get::<Recipe>(&format!("{}/recipes/{}", BASE_URL, data)) {
            Ok(recipe) => println!("{:?}", recipe),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

fn recipe_create(name: Option<&str>) {
    chopstick::recipe_create(BASE_URL, name.unwrap());
}

fn recipe_edit(
    id: Option<&str>,
    name: Option<&str>,
    author: Option<&str>,
    description: Option<&str>,
) {
    let mut params = HashMap::new();

    if let Some(value) = name {
        params.insert("name", value);
    }

    if let Some(value) = author {
        params.insert("author", value);
    }

    if let Some(value) = description {
        params.insert("description", value);
    }

    chopstick::recipe_update(BASE_URL, id.unwrap(), params);
}

fn recipe_delete(id: Option<&str>) {
    chopstick::recipe_delete(BASE_URL, id.unwrap());
}

fn ingredient_actions(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("list", Some(sub_m)) => ingredient_list(sub_m.value_of("pattern")),
        _ => {
            println!("{}", matches.usage())
        }
    }
}

fn ingredient_list(pattern: Option<&str>) {
    if let Some(ingredients) = list_ingredients(BASE_URL, pattern.unwrap_or("")) {
        ingredients
            .iter()
            .map(|x| println!("{}\t{}", x.id, x.name))
            .for_each(drop);
    }
}

fn tag_actions(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("list", Some(sub_m)) => tag_list(sub_m.value_of("pattern")),
        ("show", Some(sub_m)) => tag_show(sub_m.value_of("id")),
        _ => {
            println!("{}", matches.usage())
        }
    }
}

fn tag_list(pattern: Option<&str>) {
    if let Some(labels) = list_labels(BASE_URL, pattern.unwrap_or("")) {
        labels
            .iter()
            .map(|x| println!("{}\t{}", x.id, x.name))
            .for_each(drop);
    }
}

fn tag_show(id: Option<&str>) {
    if let Some(data) = id {
        match get::<Label>(&format!("{}/labels/{}", BASE_URL, data)) {
            Ok(label) => label
                .tagged_recipes
                .iter()
                .map(|x| println!("{}\t{}", x.id, x.name))
                .for_each(drop),
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
}
