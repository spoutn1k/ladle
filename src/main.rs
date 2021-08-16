use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

fn none() -> String {
    "None".to_string()
}

#[derive(Debug, Deserialize)]
struct Element {
    id: String,
    name: String,
    #[serde(default = "none")]
    other: String,
}

#[derive(Debug, Deserialize)]
struct RecipeList {
    accept: bool,
    data: Vec<Element>,
}

fn get_list(url: &str) -> Result<RecipeList, Box<dyn Error>> {
    let client = Client::new();

    Ok(client.get(url).send()?.json()?)
}

fn main() {
    match get_list("http://localhost:8000/recipes") {
        Ok(data) => println!("{:?}", data),
        Err(e) => eprintln!("{:?}", e),
    }
}
