use reqwest::blocking::Client;
use std::error::Error;
use std::fmt;

pub mod models {

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Label {
        pub id: String,
        pub name: String,

        #[serde(default)]
        pub tagged_recipes: Vec<Recipe>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Ingredient {
        pub id: String,
        pub name: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Requirement {
        pub ingredient: Ingredient,
        pub quantity: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Dependency {
        pub id: String,
        pub name: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Recipe {
        pub id: String,
        pub name: String,

        #[serde(default)]
        pub author: String,

        #[serde(default)]
        pub directions: String,

        #[serde(default)]
        pub requirements: Vec<Requirement>,

        #[serde(default)]
        pub dependencies: Vec<Dependency>,

        #[serde(default)]
        pub tags: Vec<Label>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Answer<T> {
        pub accept: bool,

        #[serde(default)]
        pub error: String,

        pub data: Option<T>,
    }
}

#[derive(Debug)]
struct KnifeError(String);

impl fmt::Display for KnifeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Server returned an error: {}", self.0)
    }
}

impl Error for KnifeError {}

pub fn get<'a, T: serde::de::DeserializeOwned>(url: &str) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    let response = client.get(url).send()?.json::<models::Answer<T>>()?;

    match (response.accept, response.data) {
        (true, Some(object)) => Ok(object),
        (true, None) => Err(Box::new(KnifeError(String::from(
            "Internal error. This should not happen.",
        )))),
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

pub fn list_recipes(url: &str, pattern: &str) -> Option<Vec<models::Recipe>> {
    match get::<Vec<models::Recipe>>(&format!("{}/recipes?name={}", url, pattern)) {
        Ok(recipes) => Some(recipes),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn list_ingredients(url: &str, pattern: &str) -> Option<Vec<models::Ingredient>> {
    match get::<Vec<models::Ingredient>>(&format!("{}/ingredients?name={}", url, pattern)) {
        Ok(ingredients) => Some(ingredients),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn list_labels(url: &str, pattern: &str) -> Option<Vec<models::Label>> {
    match get::<Vec<models::Label>>(&format!("{}/labels?name={}", url, pattern)) {
        Ok(labels) => Some(labels),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}
