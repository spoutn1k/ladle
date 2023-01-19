use crate::ChopstickError;
use clap::Subcommand;
use futures::future::join_all;
use ladle::models::{Ingredient, IngredientIndex};
use std::error;

/// Ingredient fetching and edition family of commands
#[derive(Subcommand)]
pub enum IngredientSubCommands {
    List {
        /// Ingredient name pattern to match in list
        pattern: Option<String>,
    },

    Show {
        /// Ingredient name, id or identifying pattern
        clue: String,
    },

    /// Create a ingredient
    Create {
        /// Ingredient name
        name: String,

        dairy: bool,
        meat: bool,
        gluten: bool,
        animal_product: bool,
    },

    /// Edit a ingredient
    Edit {
        /// Ingredient name, id or identifying pattern
        clue: String,

        #[arg(short, long)]
        name: Option<String>,

        #[arg(short, long)]
        dairy: Option<bool>,

        #[arg(short, long)]
        meat: Option<bool>,

        #[arg(short, long)]
        gluten: Option<bool>,

        #[arg(short, long)]
        animal_product: Option<bool>,
    },

    /// Delete ingredient
    Delete {
        /// Ingredient id
        id: String,
    },

    Merge {
        unified_clue: String,
        obsolete_clue: String,
    },
}

pub async fn actions(
    origin: &str,
    cmd: IngredientSubCommands,
) -> Result<(), Box<dyn error::Error>> {
    match cmd {
        IngredientSubCommands::List { pattern } => {
            ingredient_list(origin, pattern.as_deref()).await
        }
        IngredientSubCommands::Show { clue } => ingredient_show(origin, &clue).await,
        IngredientSubCommands::Create {
            name,
            dairy,
            meat,
            gluten,
            animal_product,
        } => ingredient_create(origin, &name, dairy, meat, gluten, animal_product).await,
        IngredientSubCommands::Edit {
            clue,
            name,
            dairy,
            meat,
            gluten,
            animal_product,
        } => {
            ingredient_edit(
                origin,
                &clue,
                name.as_deref(),
                dairy,
                meat,
                gluten,
                animal_product,
            )
            .await
        }
        IngredientSubCommands::Delete { id } => ingredient_delete(origin, &id).await,
        IngredientSubCommands::Merge {
            unified_clue,
            obsolete_clue,
        } => ingredient_merge(origin, &unified_clue, &obsolete_clue).await,
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

async fn ingredient_show(origin: &str, id: &str) -> Result<(), Box<dyn error::Error>> {
    let ingredient = ingredient_identify(origin, id, false).await?;

    let Ingredient {
        id,
        name,
        classifications,
        used_in,
    } = ladle::ingredient_get(origin, &ingredient.id).await?;

    println!("{}\t{}", name, id);
    println!("{:?}", classifications);

    used_in
        .iter()
        .map(|r| println!("- {}\t{}", r.name, r.id))
        .for_each(drop);

    Ok(())
}

async fn ingredient_create(
    origin: &str,
    name: &str,
    dairy: bool,
    meat: bool,
    gluten: bool,
    animal_product: bool,
) -> Result<(), Box<dyn error::Error>> {
    ladle::ingredient_create(origin, name, dairy, meat, gluten, animal_product).await?;
    Ok(())
}

async fn ingredient_edit(
    origin: &str,
    id: &str,
    name: Option<&str>,
    dairy: Option<bool>,
    meat: Option<bool>,
    gluten: Option<bool>,
    animal_product: Option<bool>,
) -> Result<(), Box<dyn error::Error>> {
    let ingredient = ingredient_identify(origin, id, false).await?;

    ladle::ingredient_update(
        origin,
        &ingredient.id,
        name,
        dairy,
        meat,
        gluten,
        animal_product,
    )
    .await
}

async fn ingredient_delete(origin: &str, id: &str) -> Result<(), Box<dyn error::Error>> {
    let ingredient = ingredient_identify(origin, id, false).await?;

    ladle::ingredient_delete(origin, &ingredient.id).await
}

/// Given two ingredient ids, migrate all requirements involving the obsolete id to the main id,
/// then delete the obsolete ingredient
async fn ingredient_merge(
    origin: &str,
    target_clue: &str,
    obsolete_clue: &str,
) -> Result<(), Box<dyn error::Error>> {
    let target_id = ingredient_identify(origin, target_clue, false).await?.id;
    let obsolete_id = ingredient_identify(origin, obsolete_clue, false).await?.id;

    let uses = ladle::ingredient_get(origin, &obsolete_id).await?;

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
        ladle::requirement_create(origin, recipe_id, &target_id, quantity, false).await
    });

    let deletions = targets.iter().map(|(recipe_id, _)| async {
        ladle::requirement_delete(origin, recipe_id, &obsolete_id).await
    });

    join_all(additions).await;
    join_all(deletions).await;
    ladle::ingredient_delete(origin, &obsolete_id).await?;

    Ok(())
}

pub async fn ingredient_identify(
    url: &str,
    clue: &str,
    create: bool,
) -> Result<IngredientIndex, Box<dyn error::Error>> {
    if let Ok(Ingredient {
        name,
        id,
        classifications: _,
        used_in: _,
    }) = ladle::ingredient_get(url, clue).await
    {
        return Ok(IngredientIndex { id, name });
    }

    let matches = ladle::ingredient_index(url, clue).await?;

    if matches.len() == 1 {
        let ingredient = matches.first().unwrap();
        if ingredient.name != clue {
            log::info!(
                "Identified ingredient `{}` from `{}`",
                ingredient.name,
                clue
            );
        }
        return Ok(ingredient.to_owned());
    }

    for indice in matches.iter() {
        if indice.name == clue {
            return Ok(indice.to_owned());
        }
    }

    if create {
        ladle::ingredient_create(url, clue, false, false, false, false).await
    } else {
        Err(Box::new(ChopstickError(format!(
            "Failed to identify ingredient from: `{}`",
            clue
        ))))
    }
}
