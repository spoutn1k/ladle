use crate::ChopstickError;
use ladle::models::{Label, LabelIndex};
use std::collections::HashMap;
use std::error;

pub async fn label_actions(
    origin: &str,
    matches: &clap::ArgMatches<'static>,
) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("list", Some(sub_m)) => label_list(origin, sub_m.value_of("pattern")).await,
        ("show", Some(sub_m)) => label_show(origin, sub_m.value_of("label")).await,
        ("create", Some(sub_m)) => label_create(origin, sub_m.value_of("name")).await,
        ("edit", Some(sub_m)) => {
            label_edit(origin, sub_m.value_of("label"), sub_m.value_of("name")).await
        }
        ("delete", Some(sub_m)) => label_delete(origin, sub_m.value_of("label")).await,
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
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

async fn label_show(origin: &str, label_clue: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let label = label_identify(origin, label_clue.unwrap(), false).await?;

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

async fn label_create(origin: &str, name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    ladle::label_create(origin, name.unwrap()).await?;
    Ok(())
}

async fn label_edit(
    origin: &str,
    label_clue: Option<&str>,
    name: Option<&str>,
) -> Result<(), Box<dyn error::Error>> {
    let mut params = HashMap::new();

    if let Some(value) = name {
        params.insert("name", value);
    }

    let label = label_identify(origin, label_clue.unwrap(), false).await?;

    ladle::label_update(origin, &label.id, params).await?;
    Ok(())
}

async fn label_delete(origin: &str, label_clue: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let label = label_identify(origin, label_clue.unwrap(), false).await?;

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
