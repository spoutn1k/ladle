use crate::ChopstickError;
use futures::future::join_all;
use ladle::models::{Ingredient, IngredientIndex};
use std::collections::HashMap;
use std::error;

pub async fn ingredient_actions(
    origin: &str,
    matches: &clap::ArgMatches<'static>,
) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("list", Some(sub_m)) => ingredient_list(origin, sub_m.value_of("pattern")).await,
        ("show", Some(sub_m)) => ingredient_show(origin, sub_m.value_of("id")).await,
        ("create", Some(sub_m)) => ingredient_create(origin, sub_m.value_of("name")).await,
        ("edit", Some(sub_m)) => {
            ingredient_edit(origin, sub_m.value_of("id"), sub_m.value_of("name")).await
        }
        ("delete", Some(sub_m)) => ingredient_delete(origin, sub_m.value_of("id")).await,
        ("merge", Some(sub_m)) => {
            ingredient_merge(
                origin,
                sub_m.value_of("target_id"),
                sub_m.value_of("obsolete_id"),
            )
            .await
        }
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

async fn ingredient_list(origin: &str, pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let ingredients = ladle::ingredient_index(origin, pattern.unwrap_or("")).await?;
    ingredients
        .iter()
        .map(|x| println!("{}\t{}", x.id, x.name))
        .for_each(drop);
    Ok(())
}

async fn ingredient_show(origin: &str, id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let ingredient = ingredient_identify(origin, id.unwrap(), false).await?;

    let Ingredient { id, name, used_in } = ladle::ingredient_get(origin, &ingredient.id).await?;

    println!("{}\t{}", name, id);
    used_in
        .iter()
        .map(|r| println!("- {}\t{}", r.name, r.id))
        .for_each(drop);

    Ok(())
}

async fn ingredient_create(origin: &str, name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    ladle::ingredient_create(origin, name.unwrap()).await?;
    Ok(())
}

async fn ingredient_edit(
    origin: &str,
    id: Option<&str>,
    name: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let ingredient = ingredient_identify(origin, id.unwrap(), false).await?;

    let mut params = HashMap::new();

    if let Some(value) = name {
        params.insert("name", value);
    }

    ladle::ingredient_update(origin, &ingredient.id, params).await
}

async fn ingredient_delete(origin: &str, id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let ingredient = ingredient_identify(origin, id.unwrap(), false).await?;

    ladle::ingredient_delete(origin, &ingredient.id).await
}

/// Given two ingredient ids, migrate all requirements involving the obsolete id to the main id,
/// then delete the obsolete ingredient
async fn ingredient_merge(
    origin: &str,
    id: Option<&str>,
    obsolete_id: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let target_id = id.unwrap();
    let obsolete_id = obsolete_id.unwrap();

    let uses = ladle::ingredient_get(origin, obsolete_id).await?;

    let uses = uses.used_in.iter().map(|recipe| async {
        match ladle::recipe_get_requirements(origin, &recipe.id)
            .await
            .unwrap_or(vec![])
            .iter()
            .find(|r| r.ingredient.id == obsolete_id)
        {
            Some(requirement) => Some((recipe.id.clone(), requirement.quantity.clone())),
            None => None,
        }
    });

    let targets = join_all(uses)
        .await
        .iter()
        .filter_map(|x| match x {
            Some((id, qt)) => Some((id.clone(), qt.clone())),
            None => None,
        })
        .collect::<Vec<(String, String)>>();

    let additions = targets.iter().map(|(recipe_id, quantity)| async {
        ladle::requirement_create(origin, recipe_id, target_id, quantity).await
    });

    let deletions = targets.iter().map(|(recipe_id, _)| async {
        ladle::requirement_delete(origin, recipe_id, obsolete_id).await
    });

    join_all(additions).await;
    join_all(deletions).await;
    ladle::ingredient_delete(origin, obsolete_id).await?;

    Ok(())
}

async fn ingredient_identify(
    url: &str,
    clue: &str,
    create: bool,
) -> Result<IngredientIndex, Box<dyn error::Error>> {
    if let Ok(Ingredient {
        name,
        id,
        used_in: _,
    }) = ladle::ingredient_get(url, clue).await
    {
        return Ok(IngredientIndex { id, name });
    }

    let matches = ladle::ingredient_index(url, clue).await?;

    if matches.len() == 1 {
        return Ok(matches.first().unwrap().to_owned());
    }

    for indice in matches.iter() {
        if indice.name == clue {
            return Ok(indice.to_owned());
        }
    }

    if create {
        ladle::ingredient_create(url, clue).await
    } else {
        Err(Box::new(ChopstickError(format!(
            "Failed to identify ingredient from: `{}`",
            clue
        ))))
    }
}
