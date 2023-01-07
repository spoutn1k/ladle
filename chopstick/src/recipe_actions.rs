use crate::ingredient_actions::ingredient_identify;
use crate::ChopstickError;
use ladle::models::RecipeIndex;
use std::collections::HashMap;
use std::error;

pub async fn recipe_actions(
    origin: &str,
    matches: &clap::ArgMatches<'static>,
) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("list", Some(sub_m)) => recipe_list(origin, sub_m.value_of("pattern")).await,
        ("show", Some(sub_m)) => recipe_show(origin, sub_m.value_of("recipe")).await,
        ("create", Some(sub_m)) => recipe_create(origin, sub_m.value_of("name")).await,
        ("edit", Some(sub_m)) => {
            recipe_edit(
                origin,
                sub_m.value_of("recipe"),
                sub_m.value_of("name"),
                sub_m.value_of("author"),
                sub_m.value_of("description"),
            )
            .await
        }
        ("delete", Some(sub_m)) => recipe_delete(origin, sub_m.value_of("recipe")).await,
        ("requirement", Some(sub_m)) => match sub_m.subcommand() {
            ("create", Some(sub_m)) => {
                requirement_add(
                    origin,
                    sub_m.value_of("recipe"),
                    sub_m.value_of("ingredient"),
                    sub_m.value_of("quantity"),
                )
                .await
            }
            ("update", Some(sub_m)) => {
                requirement_update(
                    origin,
                    sub_m.value_of("recipe"),
                    sub_m.value_of("ingredient"),
                    sub_m.value_of("quantity"),
                )
                .await
            }
            ("delete", Some(sub_m)) => {
                requirement_delete(
                    origin,
                    sub_m.value_of("recipe"),
                    sub_m.value_of("ingredient"),
                )
                .await
            }
            (&_, _) => todo!(),
        },
        ("dependency", Some(sub_m)) => match sub_m.subcommand() {
            ("add", Some(sub_m)) => {
                recipe_link(origin, sub_m.value_of("recipe"), sub_m.value_of("required")).await
            }
            ("delete", Some(sub_m)) => {
                recipe_unlink(origin, sub_m.value_of("recipe"), sub_m.value_of("required")).await
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

async fn recipe_show(origin: &str, recipe_clue: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue.unwrap()).await?;
    let recipe_data = ladle::recipe_get(origin, &recipe.id).await?;

    println!("{}", serde_json::to_string(&recipe_data)?);
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
    recipe_clue: Option<&str>,
    ingredient_clue: Option<&str>,
    quantity: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue.unwrap()).await?;
    let ingredient = ingredient_identify(origin, ingredient_clue.unwrap(), false).await?;

    ladle::requirement_create(origin, &recipe.id, &ingredient.id, quantity.unwrap()).await
}

async fn requirement_update(
    origin: &str,
    recipe_clue: Option<&str>,
    ingredient_clue: Option<&str>,
    quantity: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue.unwrap()).await?;
    let ingredient = ingredient_identify(origin, ingredient_clue.unwrap(), false).await?;

    ladle::requirement_update(origin, &recipe.id, &ingredient.id, quantity.unwrap()).await
}

async fn requirement_delete(
    origin: &str,
    recipe_clue: Option<&str>,
    ingredient_clue: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue.unwrap()).await?;
    let ingredient = ingredient_identify(origin, ingredient_clue.unwrap(), false).await?;

    ladle::requirement_delete(origin, &recipe.id, &ingredient.id).await
}

async fn recipe_link(
    origin: &str,
    recipe_clue: Option<&str>,
    required_clue: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue.unwrap()).await?;
    let required = recipe_identify(origin, required_clue.unwrap()).await?;

    ladle::recipe_link(origin, &recipe.id, &required.id).await
}

async fn recipe_unlink(
    origin: &str,
    recipe_clue: Option<&str>,
    required_clue: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue.unwrap()).await?;
    let required = recipe_identify(origin, required_clue.unwrap()).await?;

    ladle::recipe_unlink(origin, &recipe.id, &required.id).await
}

async fn recipe_identify(url: &str, clue: &str) -> Result<RecipeIndex, Box<dyn error::Error>> {
    if let Ok(recipe) = ladle::recipe_get(url, clue).await {
        return Ok(RecipeIndex {
            id: recipe.id,
            name: recipe.name,
        });
    }

    let matches = ladle::recipe_index(url, clue).await?;

    if matches.len() == 1 {
        let recipe = matches.first().unwrap();
        if recipe.name != clue {
            log::info!("Identified recipe `{}` from `{}`", recipe.name, clue);
        }
        return Ok(recipe.to_owned());
    }

    for indice in matches.iter() {
        if indice.name == clue {
            return Ok(indice.to_owned());
        }
    }

    Err(Box::new(ChopstickError(format!(
        "Failed to identify recipe from: `{}`",
        clue
    ))))
}
