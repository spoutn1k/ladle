use crate::BASE_URL;
use futures::executor::block_on;
use ladle::models::Label;
use std::collections::HashMap;
use std::error;

pub fn label_actions(matches: &clap::ArgMatches) -> Result<(), Box<dyn error::Error>> {
    match matches.subcommand() {
        ("list", Some(sub_m)) => label_list(sub_m.value_of("pattern")),
        ("show", Some(sub_m)) => label_show(sub_m.value_of("id")),
        ("create", Some(sub_m)) => label_create(sub_m.value_of("name")),
        ("edit", Some(sub_m)) => label_edit(sub_m.value_of("id"), sub_m.value_of("name")),
        ("delete", Some(sub_m)) => label_delete(sub_m.value_of("id")),
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

fn label_list(pattern: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::label_index(BASE_URL, pattern.unwrap_or("")))?
        .iter()
        .map(|x| println!("{}\t{}", x.id, x.name))
        .for_each(drop);
    Ok(())
}

fn label_show(_id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let Label {
        id: _,
        name: _,
        tagged_recipes,
    } = block_on(ladle::label_get(BASE_URL, _id.unwrap()))?;
    tagged_recipes
        .iter()
        .map(|r| {
            println!("{}\t{}", r.id, r.name);
        })
        .for_each(drop);

    Ok(())
}

fn label_create(name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::label_create(BASE_URL, name.unwrap()))?;
    Ok(())
}

fn label_edit(id: Option<&str>, name: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let mut params = HashMap::new();

    if let Some(value) = name {
        params.insert("name", value);
    }

    block_on(ladle::label_update(BASE_URL, id.unwrap(), params))?;
    Ok(())
}

fn label_delete(id: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    block_on(ladle::label_delete(BASE_URL, id.unwrap()))?;
    Ok(())
}
