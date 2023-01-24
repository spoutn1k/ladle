use crate::error::MatchingError;
use crate::helpers::display_classifications;
use crate::ingredient_actions::ingredient_identify;
use crate::label_actions::label_identify;
use clap::Subcommand;
use ladle::models::RecipeIndex;
use std::error;
use std::io::Write;
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
        directions: bool,

        /// Recipe's information
        #[arg(short, long)]
        information: bool,
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
        directions: bool,

        /// Change recipe information
        #[arg(short, long)]
        information: bool,
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
        } => recipe_create(origin, &name, author.as_deref(), directions, information).await,
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
                directions,
                information,
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

    let name_field_width = recipes
        .iter()
        .map(|r| unidecode(&r.name).len())
        .max()
        .unwrap_or(10);

    let mut term = console::Term::buffered_stdout();

    for index in recipes.iter() {
        write!(
            term,
            "{}    {}\n",
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

async fn recipe_show(origin: &str, recipe_clue: &str) -> Result<(), Box<dyn error::Error>> {
    let recipe_index = recipe_identify(origin, recipe_clue).await?;
    let recipe_tree = ladle::recipe_tree(origin, &recipe_index.id).await?;
    let recipe = &recipe_tree[0];

    let mut term = console::Term::buffered_stdout();

    write!(
        term,
        "{} by {}\n",
        console::style(&recipe.name).bold(),
        recipe.author
    )?;

    let terms = display_classifications(&recipe.classifications)?;
    if terms.len() > 0 {
        write!(
            term,
            "Contient: {}.\n",
            console::style(terms.join(", ")).italic()
        )?;
    }
    write!(term, "\n")?;

    write!(term, "{}\n\n", console::style("Ingrédients").bold())?;
    for recipe in recipe_tree.iter().rev() {
        write!(term, "{}:\n", console::style(&recipe.name).underlined())?;
        for req in recipe.requirements.iter() {
            write!(term, "  ")?;
            if req.optional {
                write!(
                    term,
                    " - {}, {} (optionnel)\n",
                    req.ingredient.name, req.quantity
                )?;
            } else {
                write!(term, " - {}, {}\n", req.ingredient.name, req.quantity)?;
            }
        }
        write!(term, "\n")?;
    }

    write!(term, "{}\n", console::style("Instructions").bold())?;
    for recipe in recipe_tree.iter().rev() {
        write!(
            term,
            "\n{}: {}\n",
            console::style(&recipe.name).underlined(),
            recipe.directions
        )?;
    }

    let tags = recipe
        .tags
        .iter()
        .map(|t| format!("#{}", t.name))
        .collect::<Vec<_>>()
        .join(" ");
    write!(term, "\n{}\n", console::style(tags).italic())?;

    term.flush()?;
    Ok(())
}

async fn recipe_create(
    origin: &str,
    name: &str,
    author: Option<&str>,
    directions: bool,
    information: bool,
) -> Result<(), Box<dyn error::Error>> {
    let directions_str = if directions {
        dialoguer::Editor::new()
            .edit("Enter recipe directions")
            .unwrap()
    } else {
        None
    };

    let information_str = if information {
        dialoguer::Editor::new()
            .edit("Enter recipe information")
            .unwrap()
    } else {
        None
    };

    ladle::recipe_create(
        origin,
        name,
        author.unwrap_or(""),
        &directions_str.unwrap_or(String::default()),
        &information_str.unwrap_or(String::default()),
    )
    .await?;
    Ok(())
}

async fn recipe_edit(
    origin: &str,
    recipe_clue: &str,
    name: Option<&str>,
    author: Option<&str>,
    directions: bool,
    information: bool,
) -> Result<(), Box<dyn error::Error>> {
    let recipe = recipe_identify(origin, recipe_clue).await?;
    let old_recipe = ladle::recipe_get(origin, &recipe.id).await?;

    let directions_str = if directions {
        dialoguer::Editor::new()
            .edit(&old_recipe.directions)
            .unwrap()
    } else {
        None
    };

    let information_str = if information {
        dialoguer::Editor::new()
            .edit(&old_recipe.information)
            .unwrap()
    } else {
        None
    };

    ladle::recipe_update(
        origin,
        &recipe.id,
        name,
        author,
        directions_str.as_deref(),
        information_str.as_deref(),
    )
    .await?;
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

    Err(Box::new(MatchingError(
        format!("Failed to identify recipe from: `{}`", clue),
        matches.iter().map(|r| r.name.clone()).collect(),
    )))
}
