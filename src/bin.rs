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

    // Same as before...
    match matches.subcommand() {
        ("recipe", Some(sub_m)) => recipe_actions(&sub_m),
        ("ingredient", Some(sub_m)) => ingredient_actions(&sub_m),
        ("label", Some(sub_m)) => label_actions(&sub_m),
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

fn ingredient_actions(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("list", Some(sub_m)) => ingredient_list(sub_m.value_of("pattern")),
        ("show", Some(sub_m)) => ingredient_show(sub_m.value_of("id")),
        ("create", Some(sub_m)) => ingredient_create(sub_m.value_of("name")),
        ("edit", Some(sub_m)) => ingredient_edit(sub_m.value_of("id"), sub_m.value_of("name")),
        ("delete", Some(sub_m)) => ingredient_delete(sub_m.value_of("id")),
        _ => {
            println!("{}", matches.usage())
        }
    }
}

fn label_actions(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("list", Some(sub_m)) => label_list(sub_m.value_of("pattern")),
        ("show", Some(sub_m)) => label_show(sub_m.value_of("id")),
        ("create", Some(sub_m)) => label_create(sub_m.value_of("name")),
        ("edit", Some(sub_m)) => label_edit(sub_m.value_of("id"), sub_m.value_of("name")),
        ("delete", Some(sub_m)) => label_delete(sub_m.value_of("id")),
        _ => {
            println!("{}", matches.usage())
        }
    }
}

fn recipe_list(pattern: Option<&str>) {
    if let Some(recipes) = chopstick::recipe_index(BASE_URL, pattern.unwrap_or("")) {
        recipes
            .iter()
            .map(|x| println!("{}\t{}", x.id, x.name))
            .for_each(drop);
    }
}

fn recipe_show(_id: Option<&str>) {
    if let Some(recipe) = chopstick::recipe_get(BASE_URL, _id.unwrap()) {
        if let Ok(json) = serde_json::to_string(&recipe) {
            println!("{}", json);
        } else {
            eprintln!("Failure serializing recipe: {:?}", recipe);
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

fn ingredient_list(pattern: Option<&str>) {
    if let Some(ingredients) = chopstick::ingredient_index(BASE_URL, pattern.unwrap_or("")) {
        ingredients
            .iter()
            .map(|x| println!("{}\t{}", x.id, x.name))
            .for_each(drop);
    }
}

fn ingredient_show(_id: Option<&str>) {
    if let Some(ingredient) = chopstick::ingredient_get(BASE_URL, _id.unwrap()) {
        if let Ok(json) = serde_json::to_string(&ingredient) {
            println!("{}", json);
        } else {
            eprintln!("Failure serializing ingredient: {:?}", ingredient);
        }
    }
}

fn ingredient_create(name: Option<&str>) {
    chopstick::ingredient_create(BASE_URL, name.unwrap());
}

fn ingredient_edit(id: Option<&str>, name: Option<&str>) {
    let mut params = HashMap::new();

    if let Some(value) = name {
        params.insert("name", value);
    }

    chopstick::ingredient_update(BASE_URL, id.unwrap(), params);
}

fn ingredient_delete(id: Option<&str>) {
    chopstick::ingredient_delete(BASE_URL, id.unwrap());
}

fn label_list(pattern: Option<&str>) {
    if let Some(labels) = chopstick::label_index(BASE_URL, pattern.unwrap_or("")) {
        labels
            .iter()
            .map(|x| println!("{}\t{}", x.id, x.name))
            .for_each(drop);
    }
}

fn label_show(_id: Option<&str>) {
    if let Some(label) = chopstick::label_get(BASE_URL, _id.unwrap()) {
        if let Ok(json) = serde_json::to_string(&label) {
            println!("{}", json);
        } else {
            eprintln!("Failure serializing label: {:?}", label);
        }
    }
}

fn label_create(name: Option<&str>) {
    chopstick::label_create(BASE_URL, name.unwrap());
}

fn label_edit(id: Option<&str>, name: Option<&str>) {
    let mut params = HashMap::new();

    if let Some(value) = name {
        params.insert("name", value);
    }

    chopstick::label_update(BASE_URL, id.unwrap(), params);
}

fn label_delete(id: Option<&str>) {
    chopstick::label_delete(BASE_URL, id.unwrap());
}
