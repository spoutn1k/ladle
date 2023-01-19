use futures::future::join_all;
use ladle::models::{Dependency, Ingredient, Label, LabelIndex, Recipe, RecipeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error;
use std::fs::File;
use std::io::BufReader;
use unidecode::unidecode;

pub async fn maintenance_actions(
    origin: &str,
    matches: &clap::ArgMatches<'static>,
) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("clone", Some(sub_m)) => {
            clone(origin, sub_m.value_of("file"), sub_m.value_of("remote")).await
        }
        ("clean", Some(_sub_m)) => clean(origin).await,
        ("dump", Some(_sub_m)) => dump(origin).await,
        (&_, _) => todo!(),
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Datadump {
    recipes: Vec<Recipe>,
    ingredients: Vec<Ingredient>,
    labels: Vec<Label>,
}

impl Datadump {
    fn strip(&mut self) {
        let mut recipe_counter: u32 = 0;
        let mut ingredient_counter: u32 = 0;
        let mut label_counter: u32 = 0;

        let mut recipe_table = HashMap::new();
        let mut ingredient_table = HashMap::new();
        let mut label_table = HashMap::new();

        for mut label in self.labels.iter_mut() {
            let new_id = format!("__label_{}", label_counter);
            label_counter += 1;
            label_table.insert(label.id.clone(), new_id.clone());
            label.id = new_id;
            label.tagged_recipes.clear();
        }

        for mut ingredient in self.ingredients.iter_mut() {
            let new_id = format!("__ingredient_{}", ingredient_counter);
            ingredient_counter += 1;
            ingredient_table.insert(ingredient.id.clone(), new_id.clone());
            ingredient.id = new_id;
            ingredient.used_in.clear();
        }

        for mut recipe in self.recipes.iter_mut() {
            let new_id = format!("__recipe_{}", recipe_counter);
            recipe_counter += 1;
            strip_ids(&mut recipe, &recipe_table, &ingredient_table, &label_table);
            recipe_table.insert(recipe.id.clone(), new_id.clone());
            recipe.id = new_id;
        }
    }
}

/// Dump all data from the remote
async fn dump_remote(origin: &str) -> Result<Datadump, Box<dyn error::Error>> {
    let origin_recipes = fetch_recipes(origin).await?;
    let origin_ingredients = fetch_ingredients(origin).await?;
    let origin_labels = fetch_labels(origin).await?;

    let mut dump = Datadump::default();

    let recipe_tiers = recipe_tiers(&origin_recipes);

    for tier in recipe_tiers.iter() {
        let mut tier: Vec<_> = tier.iter().cloned().collect();
        tier.sort_by(|lhs, rhs| unidecode(&lhs.name).cmp(&unidecode(&rhs.name)));

        for recipe in tier.iter_mut() {
            let replacement = recipe.clone();

            dump.recipes.push(replacement);
        }
    }

    dump.ingredients = origin_ingredients.iter().cloned().collect();
    dump.ingredients
        .sort_by(|lhs, rhs| unidecode(&lhs.name).cmp(&unidecode(&rhs.name)));

    dump.labels = origin_labels.iter().cloned().collect();
    dump.labels
        .sort_by(|lhs, rhs| unidecode(&lhs.name).cmp(&unidecode(&rhs.name)));

    Ok(dump)
}

async fn fetch_recipes(origin: &str) -> Result<HashSet<Recipe>, Box<dyn error::Error>> {
    let origin_index = ladle::recipe_index(origin, "").await?;

    let origin_recipes_fetches = origin_index
        .iter()
        .map(|r| ladle::recipe_get(origin, &r.id));

    let recipe_list = join_all(origin_recipes_fetches)
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

    Ok(recipe_list)
}

async fn fetch_ingredients(origin: &str) -> Result<HashSet<Ingredient>, Box<dyn error::Error>> {
    let origin_index = ladle::ingredient_index(origin, "").await?;

    let origin_ingredients_fetches = origin_index
        .iter()
        .map(|r| ladle::ingredient_get(origin, &r.id));

    let ingredient_list = join_all(origin_ingredients_fetches)
        .await
        .iter()
        .filter_map(|response| match response {
            Ok(ingredient) => Some(ingredient.to_owned()),
            Err(message) => {
                log::error!("{}", message);
                None
            }
        })
        .collect();

    Ok(ingredient_list)
}

async fn fetch_labels(origin: &str) -> Result<HashSet<Label>, Box<dyn error::Error>> {
    let origin_index = ladle::label_index(origin, "").await?;

    let origin_labels_fetches = origin_index.iter().map(|r| ladle::label_get(origin, &r.id));

    let label_list = join_all(origin_labels_fetches)
        .await
        .iter()
        .filter_map(|response| match response {
            Ok(label) => Some(label.to_owned()),
            Err(message) => {
                log::error!("{}", message);
                None
            }
        })
        .collect();

    Ok(label_list)
}

/// From a list of recipes, create all referenced ingredients on the remote and output a
/// HashMap of the indexes
async fn gen_ingredient_table<'a>(remote: &str, data: &'a Datadump) -> HashMap<&'a str, String> {
    let mut table: HashMap<&str, String> = HashMap::new();

    for ingredient in data.ingredients.iter() {
        match ladle::ingredient_create(
            remote,
            &ingredient.name as &str,
            ingredient.classifications.dairy,
            ingredient.classifications.meat,
            ingredient.classifications.gluten,
            ingredient.classifications.animal_product,
        )
        .await
        {
            Ok(created) => table
                .insert(&ingredient.id as &str, created.id.to_owned())
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
    let remote_recipe = ladle::recipe_create(
        remote,
        &recipe.name,
        &recipe.author,
        &recipe.directions,
        &recipe.information,
    )
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
            r.optional,
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

async fn clone_dump(data: &Datadump, remote: &str) -> Result<(), Box<dyn error::Error>> {
    let mut recipe_table: HashMap<&str, String> = HashMap::new();
    let ingredient_table = gen_ingredient_table(remote, &data).await;

    for recipe in data.recipes.iter() {
        let new_id = recipe_clone(remote, recipe, &ingredient_table, &recipe_table).await;
        recipe_table.insert(recipe.id.as_str(), new_id);
    }

    Ok(())
}

/// Clone all data from one remote to the other
async fn clone(
    origin: &str,
    file: Option<&str>,
    remote: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let dump;

    if let Some(path) = file {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        dump = serde_json::from_reader(reader)?;
    } else {
        dump = dump_remote(origin).await?;
    }

    clone_dump(&dump, remote.unwrap()).await
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

fn strip_ids(
    recipe: &mut Recipe,
    recipe_table: &HashMap<String, String>,
    ingredient_table: &HashMap<String, String>,
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

async fn dump(origin: &str) -> Result<(), Box<dyn error::Error>> {
    let mut dump = dump_remote(origin).await?;
    dump.strip();
    println!("{}", serde_json::to_string(&dump)?);
    Ok(())
}
