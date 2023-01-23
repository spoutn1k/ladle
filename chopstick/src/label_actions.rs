use crate::error::ChopstickError;
use clap::Subcommand;
use ladle::models::{Label, LabelIndex};
use std::error;

/// Label fetching and edition family of commands
#[derive(Subcommand)]
pub enum LabelSubCommands {
    List {
        /// Label name pattern to match in list
        pattern: Option<String>,
    },

    Show {
        /// Label name, id or identifying pattern
        clue: String,
    },

    /// Create a label
    Create {
        /// Label name
        name: String,
    },

    /// Edit a label
    Edit {
        /// Label name, id or identifying pattern
        clue: String,

        #[arg(short, long)]
        name: Option<String>,
    },

    /// Delete label
    Delete {
        /// Label id
        id: String,
    },
}
pub async fn actions(origin: &str, cmd: LabelSubCommands) -> Result<(), Box<dyn error::Error>> {
    match cmd {
        LabelSubCommands::List { pattern } => label_list(origin, pattern.as_deref()).await,
        LabelSubCommands::Show { clue } => label_show(origin, &clue).await,
        LabelSubCommands::Create { name } => label_create(origin, &name).await,
        LabelSubCommands::Edit { clue, name } => label_edit(origin, &clue, name.as_deref()).await,
        LabelSubCommands::Delete { id } => label_delete(origin, &id).await,
    }
}

async fn label_list(origin: &str, pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    ladle::label_index(origin, pattern.unwrap_or(""))
        .await?
        .iter()
        .map(|x| println!("{}\t{}", x.id, x.name))
        .for_each(drop);
    Ok(())
}

async fn label_show(origin: &str, label_clue: &str) -> Result<(), Box<dyn error::Error>> {
    let label = label_identify(origin, label_clue, false).await?;

    let Label {
        id: _,
        name: _,
        tagged_recipes,
    } = ladle::label_get(origin, &label.id).await?;

    tagged_recipes
        .iter()
        .map(|r| {
            println!("{}\t{}", r.id, r.name);
        })
        .for_each(drop);

    Ok(())
}

async fn label_create(origin: &str, name: &str) -> Result<(), Box<dyn error::Error>> {
    ladle::label_create(origin, name).await?;
    Ok(())
}

async fn label_edit(
    origin: &str,
    label_clue: &str,
    name: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let label = label_identify(origin, label_clue, false).await?;

    ladle::label_update(origin, &label.id, name.unwrap()).await?;
    Ok(())
}

async fn label_delete(origin: &str, label_clue: &str) -> Result<(), Box<dyn error::Error>> {
    let label = label_identify(origin, label_clue, false).await?;

    ladle::label_delete(origin, &label.id).await
}

pub async fn label_identify(
    url: &str,
    clue: &str,
    create: bool,
) -> Result<LabelIndex, Box<dyn error::Error>> {
    if let Ok(Label {
        name,
        id,
        tagged_recipes: _,
    }) = ladle::label_get(url, clue).await
    {
        return Ok(LabelIndex { id, name });
    }

    let matches = ladle::label_index(url, clue).await?;

    if matches.len() == 1 {
        let label = matches.first().unwrap();
        if label.name != clue {
            log::info!("Identified label `{}` from `{}`", label.name, clue);
        }
        return Ok(label.to_owned());
    }

    for indice in matches.iter() {
        if indice.name == clue {
            return Ok(indice.to_owned());
        }
    }

    if create {
        ladle::label_create(url, clue).await
    } else {
        Err(Box::new(ChopstickError(format!(
            "Failed to identify label from: `{}`",
            clue
        ))))
    }
}
