use std::collections::HashMap;
use std::error;

pub async fn recipe_actions(
    origin: &str,
    matches: &clap::ArgMatches<'static>,
) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("list", Some(sub_m)) => recipe_list(origin, sub_m.value_of("pattern")).await,
        ("show", Some(sub_m)) => recipe_show(origin, sub_m.value_of("id")).await,
        ("create", Some(sub_m)) => recipe_create(origin, sub_m.value_of("name")).await,
        ("edit", Some(sub_m)) => {
            recipe_edit(
                origin,
                sub_m.value_of("id"),
                sub_m.value_of("name"),
                sub_m.value_of("author"),
                sub_m.value_of("description"),
            )
            .await
        }
        ("delete", Some(sub_m)) => recipe_delete(origin, sub_m.value_of("id")).await,
        ("requirement", Some(sub_m)) => match sub_m.subcommand() {
            ("create", Some(sub_m)) => {
                requirement_add(
                    origin,
                    sub_m.value_of("id"),
                    sub_m.value_of("ingredient_id"),
                    sub_m.value_of("quantity"),
                )
                .await
            }
            ("update", Some(sub_m)) => {
                requirement_update(
                    origin,
                    sub_m.value_of("id"),
                    sub_m.value_of("ingredient_id"),
                    sub_m.value_of("quantity"),
                )
                .await
            }
            ("delete", Some(sub_m)) => {
                requirement_delete(
                    origin,
                    sub_m.value_of("id"),
                    sub_m.value_of("ingredient_id"),
                )
                .await
            }
            (&_, _) => todo!(),
        },
        ("dependency", Some(sub_m)) => match sub_m.subcommand() {
            ("add", Some(sub_m)) => {
                recipe_link(origin, sub_m.value_of("id"), sub_m.value_of("required_id")).await
            }
            ("delete", Some(sub_m)) => {
                recipe_unlink(origin, sub_m.value_of("id"), sub_m.value_of("required_id")).await
            }
            (&_, _) => todo!(),
        },
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

async fn recipe_list(origin: &str, pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let recipes = ladle::recipe_index(origin, pattern.unwrap_or("")).await?;
    recipes
        .iter()
        .map(|x| println!("{}\t{}", x.id, x.name))
        .for_each(drop);

    Ok(())
}

async fn recipe_show(origin: &str, _id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let recipe = ladle::recipe_get(origin, _id.unwrap()).await?;
    println!("{}", serde_json::to_string(&recipe)?);
    Ok(())
}

async fn recipe_create(origin: &str, name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let params = HashMap::from([("name", name.unwrap())]);
    ladle::recipe_create(origin, params).await?;
    Ok(())
}

async fn recipe_edit(
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

    ladle::recipe_update(origin, id.unwrap(), params).await?;

    Ok(())
}

async fn recipe_delete(origin: &str, id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    ladle::recipe_delete(origin, id.unwrap()).await?;
    Ok(())
}

async fn requirement_add(
    origin: &str,
    id: Option<&str>,
    ingredient: Option<&str>,
    quantity: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    ladle::requirement_create_from_ingredient_name(
        origin,
        id.unwrap(),
        ingredient.unwrap(),
        quantity.unwrap(),
    )
    .await?;
    Ok(())
}

async fn requirement_update(
    origin: &str,
    id: Option<&str>,
    ingredient_id: Option<&str>,
    quantity: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    ladle::requirement_update(
        origin,
        id.unwrap(),
        ingredient_id.unwrap(),
        quantity.unwrap(),
    )
    .await?;
    Ok(())
}

async fn requirement_delete(
    origin: &str,
    id: Option<&str>,
    ingredient_id: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    ladle::requirement_delete(origin, id.unwrap(), ingredient_id.unwrap()).await?;
    Ok(())
}

async fn recipe_link(
    origin: &str,
    id: Option<&str>,
    required_id: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    ladle::recipe_link(origin, id.unwrap(), required_id.unwrap()).await?;
    Ok(())
}

async fn recipe_unlink(
    origin: &str,
    id: Option<&str>,
    required_id: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    ladle::recipe_unlink(origin, id.unwrap(), required_id.unwrap()).await?;
    Ok(())
}
