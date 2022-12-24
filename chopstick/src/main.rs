use futures::executor::block_on;
use futures::future::join_all;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::error;
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
        ("recipe", Some(sub_m)) => recipe_actions(&sub_m),
        ("ingredient", Some(sub_m)) => ingredient_actions(&sub_m),
        ("label", Some(sub_m)) => label_actions(&sub_m),
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    };

    if let Err(message) = exec {
        eprintln!("{}", message);
    }
}

fn recipe_actions(matches: &clap::ArgMatches) -> Result<(), Box<dyn error::Error>> {
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
        ("requirement", Some(sub_m)) => match sub_m.subcommand() {
            ("create", Some(sub_m)) => requirement_add(
                sub_m.value_of("id"),
                sub_m.value_of("ingredient_id"),
                sub_m.value_of("quantity"),
            ),
            ("update", Some(sub_m)) => requirement_update(
                sub_m.value_of("id"),
                sub_m.value_of("ingredient_id"),
                sub_m.value_of("quantity"),
            ),
            ("delete", Some(sub_m)) => {
                requirement_delete(sub_m.value_of("id"), sub_m.value_of("ingredient_id"))
            }
            (&_, _) => todo!(),
        },
        ("dependency", Some(sub_m)) => match sub_m.subcommand() {
            ("add", Some(sub_m)) => {
                recipe_link(sub_m.value_of("id"), sub_m.value_of("required_id"))
            }
            (&_, _) => todo!(),
        },
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

fn ingredient_actions(matches: &clap::ArgMatches) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("list", Some(sub_m)) => ingredient_list(sub_m.value_of("pattern")),
        ("show", Some(sub_m)) => ingredient_show(sub_m.value_of("id")),
        ("create", Some(sub_m)) => ingredient_create(sub_m.value_of("name")),
        ("edit", Some(sub_m)) => ingredient_edit(sub_m.value_of("id"), sub_m.value_of("name")),
        ("delete", Some(sub_m)) => ingredient_delete(sub_m.value_of("id")),
        ("merge", Some(sub_m)) => {
            ingredient_merge(sub_m.value_of("target_id"), sub_m.value_of("obsolete_id"))
        }
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

fn label_actions(matches: &clap::ArgMatches) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("list", Some(sub_m)) => label_list(sub_m.value_of("pattern")),
        ("show", Some(sub_m)) => label_show(sub_m.value_of("id")),
        ("create", Some(sub_m)) => label_create(sub_m.value_of("name")),
        ("edit", Some(sub_m)) => label_edit(sub_m.value_of("id"), sub_m.value_of("name")),
        ("delete", Some(sub_m)) => label_delete(sub_m.value_of("id")),
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

fn recipe_list(pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let recipes = block_on(ladle::recipe_index(BASE_URL, pattern.unwrap_or("")))?;
    recipes
        .iter()
        .map(|x| println!("{}\t{}", x.id, x.name))
        .for_each(drop);

    Ok(())
}

fn recipe_show(_id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let recipe = block_on(ladle::recipe_get(BASE_URL, _id.unwrap()))?;
    println!("{}", serde_json::to_string(&recipe)?);
    Ok(())
}

fn recipe_create(name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::recipe_create(BASE_URL, name.unwrap()))?;
    Ok(())
}

fn recipe_edit(
    id: Option<&str>,
    name: Option<&str>,
    author: Option<&str>,
    description: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
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

    block_on(ladle::recipe_update(BASE_URL, id.unwrap(), params))?;

    Ok(())
}

fn recipe_delete(id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::recipe_delete(BASE_URL, id.unwrap()))?;
    Ok(())
}

fn ingredient_list(pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let ingredients = block_on(ladle::ingredient_index(BASE_URL, pattern.unwrap_or("")))?;
    ingredients
        .iter()
        .map(|x| println!("{}\t{}", x.id, x.name))
        .for_each(drop);
    Ok(())
}

fn ingredient_show(_id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let ladle::models::IngredientIndex { used_in } =
        block_on(ladle::ingredient_get(BASE_URL, _id.unwrap()))?;
    used_in
        .iter()
        .map(|r| println!("{}\t{}", r.id, r.name))
        .for_each(drop);
    Ok(())
}

fn ingredient_create(name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::ingredient_create(BASE_URL, name.unwrap()))?;
    Ok(())
}

fn ingredient_edit(id: Option<&str>, name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let mut params = HashMap::new();

    if let Some(value) = name {
        params.insert("name", value);
    }

    block_on(ladle::ingredient_update(BASE_URL, id.unwrap(), params))?;
    Ok(())
}

fn ingredient_delete(id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::ingredient_delete(BASE_URL, id.unwrap()))?;
    Ok(())
}

fn label_list(pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::label_index(BASE_URL, pattern.unwrap_or("")))?
        .iter()
        .map(|x| println!("{}\t{}", x.id, x.name))
        .for_each(drop);
    Ok(())
}

fn label_show(_id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let ladle::models::LabelIndex { tagged_recipes } =
        block_on(ladle::label_get(BASE_URL, _id.unwrap()))?;
    tagged_recipes
        .iter()
        .map(|r| {
            println!("{}\t{}", r.id, r.name);
        })
        .for_each(drop);

    Ok(())
}

fn label_create(name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::label_create(BASE_URL, name.unwrap()))?;
    Ok(())
}

fn label_edit(id: Option<&str>, name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let mut params = HashMap::new();

    if let Some(value) = name {
        params.insert("name", value);
    }

    block_on(ladle::label_update(BASE_URL, id.unwrap(), params))?;
    Ok(())
}

fn label_delete(id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::label_delete(BASE_URL, id.unwrap()))?;
    Ok(())
}

fn requirement_add(
    id: Option<&str>,
    ingredient: Option<&str>,
    quantity: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::requirement_create_from_ingredient_name(
        BASE_URL,
        id.unwrap(),
        ingredient.unwrap(),
        quantity.unwrap(),
    ))?;
    Ok(())
}

fn requirement_update(
    id: Option<&str>,
    ingredient_id: Option<&str>,
    quantity: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::requirement_update(
        BASE_URL,
        id.unwrap(),
        ingredient_id.unwrap(),
        quantity.unwrap(),
    ))?;
    Ok(())
}

fn requirement_delete(
    id: Option<&str>,
    ingredient_id: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::requirement_delete(
        BASE_URL,
        id.unwrap(),
        ingredient_id.unwrap(),
    ))?;
    Ok(())
}

fn recipe_link(id: Option<&str>, required_id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::recipe_link(
        BASE_URL,
        id.unwrap(),
        required_id.unwrap(),
    ))?;
    Ok(())
}

fn ingredient_merge(
    id: Option<&str>,
    obsolete_id: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let target_id = id.unwrap();
    let obsolete_id = obsolete_id.unwrap();

    let uses = block_on(ladle::ingredient_get(BASE_URL, obsolete_id))?;

    let uses = uses.used_in.iter().map(|recipe| async {
        match ladle::recipe_get_requirements(BASE_URL, &recipe.id)
            .await
            .unwrap_or(vec![])
            .iter()
            .find(|r| r.ingredient.id == obsolete_id)
        {
            Some(requirement) => Some((recipe.id.clone(), requirement.quantity.clone())),
            None => None,
        }
    });

    let targets = block_on(join_all(uses))
        .iter()
        .filter_map(|x| match x {
            Some((id, qt)) => Some((id.clone(), qt.clone())),
            None => None,
        })
        .collect::<Vec<(String, String)>>();

    let additions = targets.iter().map(|(recipe_id, quantity)| async {
        ladle::requirement_create(BASE_URL, recipe_id, target_id, quantity).await
    });

    let deletions = targets.iter().map(|(recipe_id, _)| async {
        ladle::requirement_delete(BASE_URL, recipe_id, obsolete_id).await
    });

    block_on(join_all(additions));
    block_on(join_all(deletions));
    block_on(ladle::ingredient_delete(BASE_URL, obsolete_id))?;

    Ok(())
}
