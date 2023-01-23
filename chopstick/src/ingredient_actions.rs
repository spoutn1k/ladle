use crate::error::MatchingError;
use crate::helpers::display_classifications;
use clap::Subcommand;
use futures::future::join_all;
use ladle::models::{Ingredient, IngredientIndex};
use std::error;
use std::io::Write;
use unidecode::unidecode;

/// Ingredient fetching and edition family of commands
#[derive(Subcommand)]
pub enum IngredientSubCommands {
    /// List ingredients from the server
    List {
        /// Ingredient name pattern to match in list
        pattern: Option<String>,
    },

    /// Fetch details about an ingredient
    Show {
        /// Ingredient name, id or identifying pattern
        clue: String,
    },

    /// Create an ingredient
    Create {
        /// Ingredient's name
        name: String,

        /// Mark the ingredient as containing dairy
        #[arg(short, long, default_value_t = false)]
        dairy: bool,

        /// Mark the ingredient as containing meat
        #[arg(short, long, default_value_t = false)]
        meat: bool,

        /// Mark the ingredient as containing gluten
        #[arg(short, long, default_value_t = false)]
        gluten: bool,

        /// Mark the ingredient as containing animal products
        #[arg(short, long, default_value_t = false)]
        animal_product: bool,
    },

    /// Edit an ingredient
    Edit {
        /// Ingredient name, id or identifying pattern
        clue: String,

        /// Change the ingredient's name
        #[arg(short, long)]
        name: Option<String>,

        /// Change the ingredient's dairy content
        #[arg(short, long)]
        dairy: Option<bool>,

        /// Change the ingredient's meat content
        #[arg(short, long)]
        meat: Option<bool>,

        /// Change the ingredient's gluten content
        #[arg(short, long)]
        gluten: Option<bool>,

        /// Change the ingredient's animal product content
        #[arg(short, long)]
        animal_product: Option<bool>,
    },

    /// Delete an ingredient
    Delete {
        /// Ingredient id matching the ingredient to delete
        id: String,
    },

    /// Merge one ingredient into another and update all recipes dependent on the former
    Merge {
        /// Ingredient to keep
        unified_clue: String,

        /// Ingredient to merge and delete
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
    let mut ingredients = ladle::ingredient_index(origin, pattern.unwrap_or("")).await?;
    ingredients.sort_by(|lhs, rhs| unidecode(&lhs.name).cmp(&unidecode(&rhs.name)));

    let name_field_width = ingredients.iter().map(|r| r.name.len()).max().unwrap_or(10);
    let mut term = console::Term::buffered_stdout();

    for index in ingredients.iter() {
        write!(
            term,
            "{}{}\n",
            console::pad_str(
                &index.name,
                name_field_width,
                console::Alignment::Left,
                None
            ),
            index.id
        )?;
    }

    term.flush()?;
    Ok(())
}

async fn ingredient_show(origin: &str, id: &str) -> Result<(), Box<dyn error::Error>> {
    let ingredient = ingredient_identify(origin, id, false).await?;

    let Ingredient {
        id: _,
        name,
        classifications,
        used_in,
    } = ladle::ingredient_get(origin, &ingredient.id).await?;

    let mut term = console::Term::buffered_stdout();

    write!(term, "{}\n", console::style(name).bold())?;

    let terms = display_classifications(&classifications)?;
    if terms.len() > 0 {
        write!(
            term,
            "Contient: {}.\n",
            console::style(terms.join(", ")).italic()
        )?;
    }

    if used_in.len() > 0 {
        write!(term, "\n{}\n", console::style("Utilisé dans:").underlined())?
    }

    for recipe in used_in.iter() {
        write!(term, "  - {}\n", recipe.name)?;
    }

    term.flush()?;
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
        Err(Box::new(MatchingError(
            format!("Failed to identify ingredient from: `{}`", clue),
            matches.iter().map(|r| r.name.clone()).collect(),
        )))
    }
}
