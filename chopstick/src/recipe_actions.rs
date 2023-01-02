use futures::executor::block_on;
use std::collections::HashMap;
use std::error;

pub fn recipe_actions(
    origin: &str,
    matches: &clap::ArgMatches,
) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("list", Some(sub_m)) => recipe_list(origin, sub_m.value_of("pattern")),
        ("show", Some(sub_m)) => recipe_show(origin, sub_m.value_of("id")),
        ("create", Some(sub_m)) => recipe_create(origin, sub_m.value_of("name")),
        ("edit", Some(sub_m)) => recipe_edit(
            origin,
            sub_m.value_of("id"),
            sub_m.value_of("name"),
            sub_m.value_of("author"),
            sub_m.value_of("description"),
        ),
        ("delete", Some(sub_m)) => recipe_delete(origin, sub_m.value_of("id")),
        ("requirement", Some(sub_m)) => match sub_m.subcommand() {
            ("create", Some(sub_m)) => requirement_add(
                origin,
                sub_m.value_of("id"),
                sub_m.value_of("ingredient_id"),
                sub_m.value_of("quantity"),
            ),
            ("update", Some(sub_m)) => requirement_update(
                origin,
                sub_m.value_of("id"),
                sub_m.value_of("ingredient_id"),
                sub_m.value_of("quantity"),
            ),
            ("delete", Some(sub_m)) => requirement_delete(
                origin,
                sub_m.value_of("id"),
                sub_m.value_of("ingredient_id"),
            ),
            (&_, _) => todo!(),
        },
        ("dependency", Some(sub_m)) => match sub_m.subcommand() {
            ("add", Some(sub_m)) => {
                recipe_link(origin, sub_m.value_of("id"), sub_m.value_of("required_id"))
            }
            ("delete", Some(sub_m)) => {
                recipe_unlink(origin, sub_m.value_of("id"), sub_m.value_of("required_id"))
            }
            (&_, _) => todo!(),
        },
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

fn recipe_list(origin: &str, pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let recipes = block_on(ladle::recipe_index(origin, pattern.unwrap_or("")))?;
    recipes
        .iter()
        .map(|x| println!("{}\t{}", x.id, x.name))
        .for_each(drop);

    Ok(())
}

fn recipe_show(origin: &str, _id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let recipe = block_on(ladle::recipe_get(origin, _id.unwrap()))?;
    println!("{}", serde_json::to_string(&recipe)?);
    Ok(())
}

fn recipe_create(origin: &str, name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let params = HashMap::from([("name", name.unwrap())]);
    block_on(ladle::recipe_create(origin, params))?;
    Ok(())
}

fn recipe_edit(
    origin: &str,
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

    block_on(ladle::recipe_update(origin, id.unwrap(), params))?;

    Ok(())
}

fn recipe_delete(origin: &str, id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::recipe_delete(origin, id.unwrap()))?;
    Ok(())
}

fn requirement_add(
    origin: &str,
    id: Option<&str>,
    ingredient: Option<&str>,
    quantity: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::requirement_create_from_ingredient_name(
        origin,
        id.unwrap(),
        ingredient.unwrap(),
        quantity.unwrap(),
    ))?;
    Ok(())
}

fn requirement_update(
    origin: &str,
    id: Option<&str>,
    ingredient_id: Option<&str>,
    quantity: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::requirement_update(
        origin,
        id.unwrap(),
        ingredient_id.unwrap(),
        quantity.unwrap(),
    ))?;
    Ok(())
}

fn requirement_delete(
    origin: &str,
    id: Option<&str>,
    ingredient_id: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::requirement_delete(
        origin,
        id.unwrap(),
        ingredient_id.unwrap(),
    ))?;
    Ok(())
}

fn recipe_link(
    origin: &str,
    id: Option<&str>,
    required_id: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::recipe_link(
        origin,
        id.unwrap(),
        required_id.unwrap(),
    ))?;
    Ok(())
}

fn recipe_unlink(
    origin: &str,
    id: Option<&str>,
    required_id: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::recipe_unlink(
        origin,
        id.unwrap(),
        required_id.unwrap(),
    ))?;
    Ok(())
}
