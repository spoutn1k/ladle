use crate::BASE_URL;
use futures::executor::block_on;
use futures::future::join_all;
use ladle::models::Ingredient;
use std::collections::HashMap;
use std::error;

pub fn ingredient_actions(matches: &clap::ArgMatches) -> Result<(), Box<dyn error::Error>> {
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

fn ingredient_list(pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let ingredients = block_on(ladle::ingredient_index(BASE_URL, pattern.unwrap_or("")))?;
    ingredients
        .iter()
        .map(|x| println!("{}\t{}", x.id, x.name))
        .for_each(drop);
    Ok(())
}

fn ingredient_show(_id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let Ingredient {
        id: _,
        name: _,
        used_in,
    } = block_on(ladle::ingredient_get(BASE_URL, _id.unwrap()))?;
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
