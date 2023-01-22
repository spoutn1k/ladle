use crate::ingredient_actions::ingredient_identify;
use crate::label_actions::label_identify;
use crate::ChopstickError;
use clap::Subcommand;
use ladle::models::RecipeIndex;
use std::error;
use unidecode::unidecode;

/// Recipe fetching and edition family of commands
#[derive(Subcommand)]
pub enum RecipeSubCommands {
    /// List recipes from the server
    List {
        /// Recipe name pattern to match in list
        pattern: Option<String>,
    },

    /// Fetch details about a recipe
    Show {
        /// Recipe name, id or identifying pattern
        clue: String,
    },

    /// Create a recipe on the server
    Create {
        /// Recipe's name
        name: String,

        /// Recipe's author
        #[arg(short, long)]
        author: Option<String>,

        /// Recipe's directions
        #[arg(short, long)]
        directions: Option<String>,

        /// Recipe's information
        #[arg(short, long)]
        information: Option<String>,
    },

    /// Edit an existing recipe on the server
    Edit {
        /// Recipe name, id or identifying pattern
        clue: String,

        /// Change recipe name
        #[arg(short, long)]
        name: Option<String>,

        /// Change recipe author
        #[arg(short, long)]
        author: Option<String>,

        /// Change recipe directions
        #[arg(short, long)]
        directions: Option<String>,

        /// Change recipe information
        #[arg(short, long)]
        information: Option<String>,
    },

    /// Delete a recipe from the server
    Delete {
        /// Recipe id matching the recipe to delete
        id: String,
    },

    Requirement {
        #[command(subcommand)]
        requirement: RequirementSubCommands,
    },

    Dependency {
        #[command(subcommand)]
        dependency: DependencySubCommands,
    },

    Tag {
        #[command(subcommand)]
        tag: TagSubCommands,
    },
}

/// Manage a recipe's requirements
#[derive(Subcommand)]
pub enum RequirementSubCommands {
    /// Create a requirement
    Create {
        /// Recipe name, id or identifying pattern
        recipe_clue: String,

        /// Ingredient name, id or identifying pattern
        ingredient_clue: String,

        /// Required quantity
        quantity: String,

        /// Optional status
        #[arg(short, long)]
        optional: bool,
    },

    /// Edit a requirement
    Edit {
        /// Recipe name, id or identifying pattern
        recipe_clue: String,

        /// Ingredient name, id or identifying pattern
        ingredient_clue: String,

        /// Change the required quantity
        #[arg(short, long)]
        quantity: Option<String>,

        /// Change the optional status
        #[arg(short, long)]
        optional: Option<bool>,
    },

    /// Delete a requirement
    Delete {
        /// Recipe name, id or identifying pattern
        recipe_clue: String,

        /// Ingredient name, id or identifying pattern
        ingredient_clue: String,
    },
}

/// Manage a recipe's dependencies
#[derive(Subcommand)]
pub enum DependencySubCommands {
    /// Create a dependency
    Create {
        /// Recipe name, id or identifying pattern
        recipe_clue: String,

        /// Recipe name, id or identifying pattern
        required_clue: String,

        /// Required quantity
        quantity: Option<String>,

        /// Optional status
        #[arg(short, long)]
        optional: bool,
    },

    /// Edit a dependency
    Edit {
        /// Recipe name, id or identifying pattern
        recipe_clue: String,

        /// Recipe name, id or identifying pattern
        required_clue: String,

        /// Change the required quantity
        #[arg(short, long)]
        quantity: Option<String>,

        /// Change the optional status
        #[arg(short, long)]
        optional: Option<bool>,
    },

    /// Delete a dependency
    Delete {
        /// Recipe name, id or identifying pattern
        recipe_clue: String,

        /// Recipe name, id or identifying pattern
        required_clue: String,
    },
}

/// Manage a recipe's tags
#[derive(Subcommand)]
pub enum TagSubCommands {
    /// Add a tag
    Add {
        /// Recipe name, id or identifying pattern
        recipe_clue: String,

        /// Label name
        label_name: String,
    },

    /// Delete a tag
    Delete {
        /// Recipe name, id or identifying pattern
        recipe_clue: String,

        /// Label name, id or identifying pattern
        label_clue: String,
    },
}

pub async fn requirement_actions(
    origin: &str,
    cmd: RequirementSubCommands,
) -> Result<(), Box<dyn error::Error>> {
    match cmd {
        RequirementSubCommands::Create {
            recipe_clue,
            ingredient_clue,
            quantity,
            optional,
        } => requirement_add(origin, &recipe_clue, &ingredient_clue, &quantity, optional).await,
        RequirementSubCommands::Edit {
            recipe_clue,
            ingredient_clue,
            quantity,
            optional,
        } => {
            requirement_update(
                origin,
                &recipe_clue,
                &ingredient_clue,
                quantity.as_deref(),
                optional,
            )
            .await
        }
        RequirementSubCommands::Delete {
            recipe_clue,
            ingredient_clue,
        } => requirement_delete(origin, &recipe_clue, &ingredient_clue).await,
    }
}

pub async fn dependency_actions(
    origin: &str,
    cmd: DependencySubCommands,
) -> Result<(), Box<dyn error::Error>> {
    match cmd {
        DependencySubCommands::Create {
            recipe_clue,
            required_clue,
            quantity,
            optional,
        } => {
            dependency_create(
                origin,
                &recipe_clue,
                &required_clue,
                quantity.as_deref(),
                optional,
            )
            .await
        }
        DependencySubCommands::Edit {
            recipe_clue,
            required_clue,
            quantity,
            optional,
        } => {
            dependency_edit(
                origin,
                &recipe_clue,
                &required_clue,
                quantity.as_deref(),
                optional,
            )
            .await
        }
        DependencySubCommands::Delete {
            recipe_clue,
            required_clue,
        } => dependency_delete(origin, &recipe_clue, &required_clue).await,
    }
}

pub async fn tag_actions(origin: &str, cmd: TagSubCommands) -> Result<(), Box<dyn error::Error>> {
    match cmd {
        TagSubCommands::Add {
            recipe_clue,
            label_name,
        } => recipe_tag(origin, &recipe_clue, &label_name).await,
        TagSubCommands::Delete {
            recipe_clue,
            label_clue,
        } => recipe_untag(origin, &recipe_clue, &label_clue).await,
    }
}

pub async fn actions(origin: &str, cmd: RecipeSubCommands) -> Result<(), Box<dyn error::Error>> {
    match cmd {
        RecipeSubCommands::List { pattern } => recipe_list(origin, pattern.as_deref()).await,
        RecipeSubCommands::Show { clue } => recipe_show(origin, &clue).await,
        RecipeSubCommands::Create {
            name,
            author,
            directions,
            information,
        } => {
            recipe_create(
                origin,
                &name,
                author.as_deref(),
                directions.as_deref(),
                information.as_deref(),
            )
            .await
        }
        RecipeSubCommands::Edit {
            clue,
            name,
            author,
            directions,
            information,
        } => {
            recipe_edit(
                origin,
                &clue,
                name.as_deref(),
                author.as_deref(),
                directions.as_deref(),
                information.as_deref(),
            )
            .await
        }
        RecipeSubCommands::Delete { id } => recipe_delete(origin, &id).await,
        RecipeSubCommands::Requirement { requirement } => {
            requirement_actions(origin, requirement).await
        }
        RecipeSubCommands::Dependency { dependency } => {
            dependency_actions(origin, dependency).await
        }
        RecipeSubCommands::Tag { tag } => tag_actions(origin, tag).await,
    }
}

async fn recipe_list(origin: &str, pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let mut recipes = ladle::recipe_index(origin, pattern.unwrap_or("")).await?;
    recipes.sort_by(|lhs, rhs| unidecode(&lhs.name).cmp(&unidecode(&rhs.name)));
    recipes
        .iter()
        .map(|x| println!("{}\t{}", x.name, x.id))
        .for_each(drop);

    Ok(())
}

async fn recipe_show(origin: &str, recipe_clue: &str) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;
    let recipe_data = ladle::recipe_get(origin, &recipe.id).await?;

    println!("{}", serde_json::to_string(&recipe_data)?);
    Ok(())
}

async fn recipe_create(
    origin: &str,
    name: &str,
    author: Option<&str>,
    directions: Option<&str>,
    information: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    ladle::recipe_create(
        origin,
        name,
        author.unwrap_or(""),
        directions.unwrap_or(""),
        information.unwrap_or(""),
    )
    .await?;
    Ok(())
}

async fn recipe_edit(
    origin: &str,
    recipe_clue: &str,
    name: Option<&str>,
    author: Option<&str>,
    directions: Option<&str>,
    information: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;

    ladle::recipe_update(origin, &recipe.id, name, author, directions, information).await?;
    Ok(())
}

async fn recipe_delete(origin: &str, id: &str) -> Result<(), Box<dyn error::Error>> {
    ladle::recipe_delete(origin, id).await
}

async fn requirement_add(
    origin: &str,
    recipe_clue: &str,
    ingredient_clue: &str,
    quantity: &str,
    optional: bool,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;
    let ingredient = ingredient_identify(origin, ingredient_clue, false).await?;

    ladle::requirement_create(origin, &recipe.id, &ingredient.id, quantity, optional).await
}

async fn requirement_update(
    origin: &str,
    recipe_clue: &str,
    ingredient_clue: &str,
    quantity: Option<&str>,
    optional: Option<bool>,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;
    let ingredient = ingredient_identify(origin, ingredient_clue, false).await?;

    ladle::requirement_update(origin, &recipe.id, &ingredient.id, quantity, optional).await
}

async fn requirement_delete(
    origin: &str,
    recipe_clue: &str,
    ingredient_clue: &str,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;
    let ingredient = ingredient_identify(origin, ingredient_clue, false).await?;

    ladle::requirement_delete(origin, &recipe.id, &ingredient.id).await
}

async fn dependency_create(
    origin: &str,
    recipe_clue: &str,
    required_clue: &str,
    quantity: Option<&str>,
    optional: bool,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;
    let required = recipe_identify(origin, required_clue).await?;

    ladle::dependency_create(
        origin,
        &recipe.id,
        &required.id,
        &quantity.unwrap_or(""),
        optional,
    )
    .await
}

async fn dependency_edit(
    origin: &str,
    recipe_clue: &str,
    required_clue: &str,
    quantity: Option<&str>,
    optional: Option<bool>,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;
    let required = recipe_identify(origin, required_clue).await?;

    ladle::dependency_edit(origin, &recipe.id, &required.id, quantity, optional).await
}

async fn dependency_delete(
    origin: &str,
    recipe_clue: &str,
    required_clue: &str,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;
    let required = recipe_identify(origin, required_clue).await?;

    ladle::dependency_delete(origin, &recipe.id, &required.id).await
}

async fn recipe_tag(
    origin: &str,
    recipe_clue: &str,
    label: &str,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;

    ladle::recipe_tag(origin, &recipe.id, &label).await
}

async fn recipe_untag(
    origin: &str,
    recipe_clue: &str,
    label_clue: &str,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;
    let label = label_identify(origin, label_clue, false).await?;

    ladle::recipe_untag(origin, &recipe.id, &label.id).await
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
