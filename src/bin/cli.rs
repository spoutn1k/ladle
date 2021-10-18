use chopstick::get;
use chopstick::models::Recipe;
#[macro_use]
extern crate clap;

fn main() {
    let matches = clap_app!(Chopstick =>
        (version: "0.0")
        (author: "JBS <jb.skutnik@gmail.com>")
        (about: "Get data from a knife server")
        (@subcommand recipe =>
            (about: "access recipes")
            (@subcommand list =>
                (about: "list recipes")
                (@arg pattern: "list recipes matching a pattern")
            )
        )
        (@subcommand tag =>
            (about: "access tags")
            (@subcommand list =>
                (about: "list tags")
            )
        )
    )
    .get_matches();

    // Same as before...
    match matches.subcommand() {
        ("recipe", Some(sub_m)) => recipe_actions(&sub_m),
        ("tag", Some(sub_m)) => tag_actions(&sub_m),
        _ => {}
    }
}

fn recipe_actions(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("list", Some(sub_m)) => recipe_list(sub_m.value_of("pattern")),
        _ => {}
    }
}

fn recipe_list(_pattern: Option<&str>) {
    let list: Vec<Recipe>;

    match get::<Vec<Recipe>>("http://localhost:8000/recipes") {
        Ok(recipes) => list = recipes,
        Err(e) => {
            eprintln!("{:?}", e);
            list = vec![]
        }
    }

    list.iter().map(|x| println!("{}", x.name)).for_each(drop);
}

fn tag_actions(matches: &clap::ArgMatches) {
    match matches.subcommand_name() {
        Some("list") => match get::<Vec<Recipe>>("http://localhost:8000/labels") {
            Ok(recipes) => {
                recipes
                    .iter()
                    .map(|x| println!("{}", x.name))
                    .for_each(drop);
            }
            Err(e) => eprintln!("{:?}", e),
        },

        _ => {
            println!("Dropped !")
        }
    }
}
