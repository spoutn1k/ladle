use crate::BASE_URL;
use futures::executor::block_on;
use std::collections::HashMap;
use std::error;

pub fn recipe_actions(matches: &clap::ArgMatches) -> Result<(), Box<dyn error::Error>> {
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
    let params = HashMap::from([("name", name.unwrap())]);
    block_on(ladle::recipe_create(BASE_URL, params))?;
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
