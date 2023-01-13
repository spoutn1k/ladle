use futures::future::join_all;
use ladle::models::{
    Dependency, Ingredient, IngredientIndex, Label, LabelIndex, Recipe, RecipeIndex,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::error;
use unidecode::unidecode;

pub async fn maintenance_actions(
    origin: &str,
    matches: &clap::ArgMatches<'static>,
) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("clone", Some(sub_m)) => clone(origin, sub_m.value_of("remote")).await,
        ("clean", Some(_sub_m)) => clean(origin).await,
        ("dump", Some(_sub_m)) => dump(origin).await,
        (&_, _) => todo!(),
    }
}

/// From a list of recipes, create all referenced ingredients on the remote and output a
/// HashMap of the indexes
async fn gen_ingredient_table<'a>(
    remote: &str,
    origin_recipes: &'a HashSet<Recipe>,
) -> HashMap<&'a str, String> {
    let mut ingredients_indexes: Vec<&IngredientIndex> = origin_recipes
        .iter()
        .flat_map(|recipe| recipe.requirements.iter().map(|req| &req.ingredient))
        .collect();

    ingredients_indexes.sort_by(|&lhs, &rhs| lhs.name.cmp(&rhs.name));

    let mut table: HashMap<&str, String> = HashMap::new();

    for IngredientIndex { name, id } in ingredients_indexes {
        match ladle::ingredient_create(remote, name).await {
            Ok(ingredient) => table
                .insert(id, ingredient.id.to_owned())
                .unwrap_or_default(),
            Err(message) => {
                log::error!("{}", message);
                String::default()
            }
        };
    }

    table
}

/// Split a set of recipes into tiers: recipes in a tier only depend on recipes in the tiers
/// before.
fn recipe_tiers<'a>(recipe_set: &'a HashSet<Recipe>) -> Vec<HashSet<&'a Recipe>> {
    // Initialize tiers with the recipes having no dependencies
    let (basic_recipes, mut rest): (HashSet<&Recipe>, HashSet<&Recipe>) =
        recipe_set.iter().partition(|r| r.dependencies.len() == 0);

    let mut tiers = vec![basic_recipes];
    let mut tiered = tiers
        .last()
        .unwrap()
        .iter()
        .map(|recipe| recipe.id.as_str())
        .collect::<HashSet<_>>();

    while rest.len() != 0 {
        let (new_tier, new_rest): (HashSet<&Recipe>, HashSet<&Recipe>) =
            rest.iter().partition(|recipe| {
                let dependencies = recipe
                    .dependencies
                    .iter()
                    .map(
                        |Dependency {
                             recipe: RecipeIndex { id, name: _ },
                             quantity: _,
                             optional: _,
                         }| id.as_str(),
                    )
                    .collect::<HashSet<&str>>();

                dependencies.is_subset(&tiered)
            });

        tiers.push(new_tier);
        tiered.extend(
            tiers
                .last()
                .unwrap()
                .iter()
                .map(|recipe| recipe.id.as_str())
                .collect::<HashSet<_>>(),
        );

        rest = new_rest;
    }

    tiers
}

/// Clone fully a recipe. Translate ingredient hashes with the ingredient table, translate
/// dependency hashes with the recipe table. Assumes all dependencies are present on the remote.
async fn recipe_clone(
    remote: &str,
    recipe: &Recipe,
    ingredient_table: &HashMap<&str, String>,
    recipe_table: &HashMap<&str, String>,
) -> String {
    let params = HashMap::from([
        ("name", recipe.name.as_str()),
        ("author", recipe.author.as_str()),
        ("directions", recipe.directions.as_str()),
    ]);

    let remote_recipe = ladle::recipe_create(remote, params)
        .await
        .expect("Failed to create recipe on remote");

    let recipe_tags: Vec<&LabelIndex> = recipe.tags.iter().collect();

    let tag_creations = recipe_tags
        .iter()
        .map(|l| ladle::recipe_tag(remote, &remote_recipe.id, l.name.as_str()));

    join_all(tag_creations)
        .await
        .iter()
        .enumerate()
        .map(|(index, response)| {
            if let Err(message) = response {
                log::error!(
                    "Error tagging recipe {} with label {}: {}",
                    recipe.name,
                    recipe_tags[index].name,
                    message
                )
            }
        })
        .for_each(drop);

    let (recipe_requirements, rejected): (Vec<_>, Vec<_>) = recipe
        .requirements
        .iter()
        .partition(|r| ingredient_table.contains_key(r.ingredient.id.as_str()));

    for requirement in rejected.iter() {
        log::error!(
            "Cannot create requirement of `{}` for `{}`: ingredient not mapped on target remote",
            requirement.ingredient.name,
            recipe.name
        )
    }

    let requirement_creations = recipe_requirements.iter().map(|r| {
        let remote_ingredient_id = ingredient_table.get(r.ingredient.id.as_str()).unwrap();
        ladle::requirement_create(
            remote,
            remote_recipe.id.as_str(),
            remote_ingredient_id.as_str(),
            &r.quantity,
        )
    });

    join_all(requirement_creations)
        .await
        .iter()
        .enumerate()
        .map(|(index, response)| {
            if let Err(message) = response {
                log::error!(
                    "Error adding requirement of `{}` for `{}`: {}",
                    recipe_requirements[index].ingredient.name,
                    recipe.name,
                    message
                )
            }
        })
        .for_each(drop);

    let dependencies =
        recipe
            .dependencies
            .iter()
            .filter_map(|d| match recipe_table.get(d.recipe.id.as_str()) {
                Some(remote_dependency_id) => Some(ladle::dependency_create(
                    remote,
                    remote_recipe.id.as_str(),
                    remote_dependency_id.as_str(),
                    &d.quantity.as_str(),
                    d.optional,
                )),
                None => None,
            });
    join_all(dependencies)
        .await
        .iter()
        .map(|response| {
            if let Err(message) = response {
                log::error!("{:?}", message)
            }
        })
        .for_each(drop);

    remote_recipe.id
}

/// Clone all data from one remote to the other
pub async fn clone(origin: &str, remote: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let remote = remote.unwrap();

    let origin_index = ladle::recipe_index(origin, "").await?;

    let origin_recipes_fetches = origin_index
        .iter()
        .map(|r| ladle::recipe_get(origin, &r.id));

    let origin_recipes: HashSet<Recipe> = join_all(origin_recipes_fetches)
        .await
        .iter()
        .filter_map(|response| match response {
            Ok(recipe) => Some(recipe.to_owned()),
            Err(message) => {
                log::error!("{}", message);
                None
            }
        })
        .collect();

    let ingredient_table = gen_ingredient_table(remote, &origin_recipes).await;
    let recipe_tiers = recipe_tiers(&origin_recipes);
    let mut recipe_table: HashMap<&str, String> = HashMap::new();

    for tier in recipe_tiers.iter() {
        let tier: Vec<_> = tier.iter().collect();
        let clones = tier
            .iter()
            .map(|recipe| recipe_clone(remote, recipe, &ingredient_table, &recipe_table));

        join_all(clones)
            .await
            .iter()
            .enumerate()
            .for_each(|(index, id)| {
                recipe_table.insert(tier[index].id.as_str(), id.to_owned());
            });
    }

    Ok(())
}

async fn clean(origin: &str) -> Result<(), Box<dyn error::Error>> {
    let ingredients = ladle::ingredient_index(origin, "").await?;

    let number = ingredients.len().try_into().ok().unwrap();
    let bar = indicatif::ProgressBar::new(number)
        .with_message("Fetching ingredients")
        .with_style(
            indicatif::ProgressStyle::with_template("{msg:<30} [{wide_bar}] {pos:>4}/{len:4}")
                .unwrap()
                .progress_chars("=>-"),
        );

    let fetches = ingredients
        .iter()
        .map(|ingredient| ladle::ingredient_get(origin, &ingredient.id));

    let mut to_delete = HashSet::new();
    for fetch in fetches {
        bar.inc(1);
        match fetch.await {
            Err(message) => log::error!("{:?}", message),
            Ok(ingredient) => {
                if ingredient.used_in.len() == 0 {
                    to_delete.insert(ingredient);
                }
            }
        }
    }

    for ing in to_delete.iter() {
        ladle::ingredient_delete(origin, ing.id.as_str()).await?;
        log::info!("Deleted ingredient `{}` ({})", ing.name, ing.id);
    }

    bar.finish();

    let labels = ladle::label_index(origin, "").await?;
    let fetches = labels
        .iter()
        .map(|label| ladle::label_get(origin, &label.id));

    let number = fetches.len().try_into().ok().unwrap();
    let bar = indicatif::ProgressBar::new(number)
        .with_message("Fetching labels")
        .with_style(
            indicatif::ProgressStyle::with_template("{msg:<30} [{wide_bar}] {pos:>4}/{len:4}")
                .unwrap()
                .progress_chars("=>-"),
        );

    let mut to_delete = HashSet::new();
    for fetch in fetches {
        bar.inc(1);
        match fetch.await {
            Err(message) => log::error!("{:?}", message),
            Ok(label) => {
                if label.tagged_recipes.len() == 0 {
                    to_delete.insert(label);
                }
            }
        }
    }

    bar.finish();

    for label in to_delete.iter() {
        ladle::label_delete(origin, label.id.as_str()).await?;
        log::info!("Deleted label `{}` ({})", label.name, label.id);
    }

    Ok(())
}

/// From a list of recipes, create all referenced ingredients on the remote and output a
/// HashMap of the indexes
fn gen_ingredient_table_strip_ids(origin_recipes: &HashSet<Recipe>) -> HashMap<String, String> {
    let mut ingredient_counter: u32 = 0;
    let mut ingredients_indexes: Vec<&IngredientIndex> = origin_recipes
        .iter()
        .flat_map(|recipe| recipe.requirements.iter().map(|req| &req.ingredient))
        .collect();

    ingredients_indexes.sort_by(|&lhs, &rhs| unidecode(&lhs.name).cmp(&unidecode(&rhs.name)));

    let mut table = HashMap::new();

    for IngredientIndex { name: _, id } in ingredients_indexes {
        table
            .insert(
                String::from(id),
                format!("__ingredient_{}", ingredient_counter),
            )
            .unwrap_or_default();
        ingredient_counter += 1;
    }

    table
}

/// From a list of recipes, create all referenced ingredients on the remote and output a
/// HashMap of the indexes
fn gen_label_table_strip_ids(origin_recipes: &HashSet<Recipe>) -> HashMap<String, String> {
    let mut label_counter: u32 = 0;
    let mut labels_indexes: Vec<&LabelIndex> = origin_recipes
        .iter()
        .flat_map(|recipe| recipe.tags.iter())
        .collect();

    labels_indexes.sort_by(|&lhs, &rhs| unidecode(&lhs.name).cmp(&unidecode(&rhs.name)));

    let mut table = HashMap::new();

    for LabelIndex { name: _, id } in labels_indexes {
        table
            .insert(String::from(id), format!("__label_{}", label_counter))
            .unwrap_or_default();
        label_counter += 1;
    }

    table
}

fn strip_ids(
    recipe: &mut Recipe,
    ingredient_table: &HashMap<String, String>,
    recipe_table: &HashMap<String, String>,
    label_table: &HashMap<String, String>,
) {
    let mut replaced_requirements = HashSet::new();
    for requirement in recipe.requirements.iter() {
        if let Some(replacement) = ingredient_table.get(&requirement.ingredient.id) {
            let mut replaced = requirement.clone();
            replaced.ingredient.id = replacement.to_owned();
            replaced_requirements.insert(replaced);
        }
    }

    let mut replaced_dependencies = HashSet::new();
    for dependency in recipe.dependencies.iter() {
        if let Some(replacement) = recipe_table.get(&dependency.recipe.id) {
            let mut replaced = dependency.clone();
            replaced.recipe.id = replacement.to_owned();
            replaced_dependencies.insert(replaced);
        }
    }

    let mut replaced_tags = HashSet::new();
    for tag in recipe.tags.iter() {
        if let Some(replacement) = label_table.get(&tag.id) {
            let mut replaced = tag.clone();
            replaced.id = replacement.to_owned();
            replaced_tags.insert(replaced);
        }
    }

    recipe.requirements = replaced_requirements;
    recipe.dependencies = replaced_dependencies;
    recipe.tags = replaced_tags;
}

#[derive(Debug, Serialize, Default)]
struct Dump {
    recipes: Vec<Recipe>,
    ingredients: Vec<Ingredient>,
    labels: Vec<Label>,
}

/// Dump all data from the remote
pub async fn dump(origin: &str) -> Result<(), Box<dyn error::Error>> {
    let origin_index = ladle::recipe_index(origin, "").await?;

    let origin_recipes_fetches = origin_index
        .iter()
        .map(|r| ladle::recipe_get(origin, &r.id));

    let origin_recipes: HashSet<Recipe> = join_all(origin_recipes_fetches)
        .await
        .iter()
        .filter_map(|response| match response {
            Ok(recipe) => Some(recipe.to_owned()),
            Err(message) => {
                log::error!("{}", message);
                None
            }
        })
        .collect();

    let mut dump: Dump = Dump::default();
    let mut recipe_counter: u32 = 0;

    let ingredient_table = gen_ingredient_table_strip_ids(&origin_recipes);

    for (id, replacement) in ingredient_table.iter() {
        let mut ingredient = ladle::ingredient_get(origin, id).await?;

        ingredient.id = replacement.to_owned();
        ingredient.used_in = vec![];
        dump.ingredients.push(ingredient);
    }

    let label_table = gen_label_table_strip_ids(&origin_recipes);

    for (id, replacement) in label_table.iter() {
        let mut label = ladle::label_get(origin, id).await?;

        label.id = replacement.to_owned();
        label.tagged_recipes = vec![];
        dump.labels.push(label);
    }

    let recipe_tiers = recipe_tiers(&origin_recipes);
    let mut recipe_table = HashMap::new();

    for tier in recipe_tiers.iter() {
        let mut tier: Vec<_> = tier.iter().cloned().collect();
        tier.sort_by(|lhs, rhs| unidecode(&lhs.name).cmp(&unidecode(&rhs.name)));

        for recipe in tier.iter_mut() {
            let mut replacement = recipe.clone();
            let replacement_id: String = format!("__recipe_{}", recipe_counter);
            recipe_counter += 1;

            strip_ids(
                &mut replacement,
                &ingredient_table,
                &recipe_table,
                &label_table,
            );

            recipe_table.insert(String::from(&recipe.id), replacement_id.clone());
            replacement.id = replacement_id;

            dump.recipes.push(replacement);
        }
    }

    println!("{}", serde_json::to_string(&dump)?);
    Ok(())
}
