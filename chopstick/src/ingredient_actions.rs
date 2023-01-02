use futures::executor::block_on;
use futures::future::join_all;
use ladle::models::Ingredient;
use std::collections::HashMap;
use std::error;

pub fn ingredient_actions(
    origin: &str,
    matches: &clap::ArgMatches,
) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("list", Some(sub_m)) => ingredient_list(origin, sub_m.value_of("pattern")),
        ("show", Some(sub_m)) => ingredient_show(origin, sub_m.value_of("id")),
        ("create", Some(sub_m)) => ingredient_create(origin, sub_m.value_of("name")),
        ("edit", Some(sub_m)) => {
            ingredient_edit(origin, sub_m.value_of("id"), sub_m.value_of("name"))
        }
        ("delete", Some(sub_m)) => ingredient_delete(origin, sub_m.value_of("id")),
        ("merge", Some(sub_m)) => ingredient_merge(
            origin,
            sub_m.value_of("target_id"),
            sub_m.value_of("obsolete_id"),
        ),
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

fn ingredient_list(origin: &str, pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let ingredients = block_on(ladle::ingredient_index(origin, pattern.unwrap_or("")))?;
    ingredients
        .iter()
        .map(|x| println!("{}\t{}", x.id, x.name))
        .for_each(drop);
    Ok(())
}

fn ingredient_show(origin: &str, id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let Ingredient {
        id: _,
        name: _,
        used_in,
    } = block_on(ladle::ingredient_get(origin, id.unwrap()))?;
    used_in
        .iter()
        .map(|r| println!("{}\t{}", r.id, r.name))
        .for_each(drop);
    Ok(())
}

fn ingredient_create(origin: &str, name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::ingredient_create(origin, name.unwrap()))?;
    Ok(())
}

fn ingredient_edit(
    origin: &str,
    id: Option<&str>,
    name: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let mut params = HashMap::new();

    if let Some(value) = name {
        params.insert("name", value);
    }

    block_on(ladle::ingredient_update(origin, id.unwrap(), params))?;
    Ok(())
}

fn ingredient_delete(origin: &str, id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::ingredient_delete(origin, id.unwrap()))?;
    Ok(())
}

fn ingredient_merge(
    origin: &str,
    id: Option<&str>,
    obsolete_id: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let target_id = id.unwrap();
    let obsolete_id = obsolete_id.unwrap();

    let uses = block_on(ladle::ingredient_get(origin, obsolete_id))?;

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

    let targets = block_on(join_all(uses))
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

    block_on(join_all(additions));
    block_on(join_all(deletions));
    block_on(ladle::ingredient_delete(origin, obsolete_id))?;

    Ok(())
}
