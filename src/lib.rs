use reqwest::blocking::Client;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub mod models {

    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Label {
        pub id: String,
        pub name: String,

        #[serde(default)]
        pub tagged_recipes: Vec<Recipe>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Ingredient {
        pub id: String,
        pub name: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Requirement {
        pub ingredient: Ingredient,
        pub quantity: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Dependency {
        pub id: String,
        pub name: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
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

fn post<'a, T: serde::de::DeserializeOwned>(
    url: &str,
    params: HashMap<&str, &str>,
) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    let response = client
        .post(url)
        .form(&params)
        .send()?
        .json::<models::Answer<T>>()?;

    match (response.accept, response.data) {
        (true, Some(object)) => Ok(object),
        (true, None) => Err(Box::new(KnifeError(String::from(
            "Internal error. This should not happen.",
        )))),
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

fn put<'a, T: serde::de::DeserializeOwned + fmt::Debug>(
    url: &str,
    params: HashMap<&str, &str>,
) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    let response = client
        .put(url)
        .form(&params)
        .send()?
        .json::<models::Answer<T>>()?;

    match (response.accept, response.data) {
        (true, Some(object)) => Ok(object),
        (true, None) => Err(Box::new(KnifeError(String::from(
            "Internal error. This should not happen.",
        )))),
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

fn delete(url: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let response = client.delete(url).send()?.json::<models::Answer<()>>()?;

    match (response.accept, response.data) {
        (true, _) => Ok(()),
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

pub fn recipe_index(url: &str, pattern: &str) -> Option<Vec<models::Recipe>> {
    match get::<Vec<models::Recipe>>(&format!("{}/recipes?name={}", url, pattern)) {
        Ok(recipes) => Some(recipes),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn recipe_get(url: &str, id: &str) -> Option<models::Recipe> {
    match get::<models::Recipe>(&format!("{}/recipes/{}", url, id)) {
        Ok(recipe) => Some(recipe),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn recipe_create(url: &str, name: &str) -> Option<models::Recipe> {
    let mut params = HashMap::new();
    params.insert("name", name);

    match post::<models::Recipe>(&format!("{}/recipes/new", url), params) {
        Ok(recipe) => Some(recipe),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn recipe_update(url: &str, id: &str, data: HashMap<&str, &str>) -> Option<models::Recipe> {
    match put::<models::Recipe>(&format!("{}/recipes/{}", url, id), data) {
        Ok(recipe) => Some(recipe),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn recipe_delete(url: &str, id: &str) -> Option<()> {
    match delete(&format!("{}/recipes/{}", url, id)) {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn recipe_link(url: &str, id: &str, required_id: &str) -> Option<()> {
    let mut params = HashMap::new();
    params.insert("required_id", required_id);
    match post(&format!("{}/recipes/{}/dependencies/add", url, id), params) {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn recipe_unlink(url: &str, id: &str, required_id: &str) -> Option<()> {
    match delete(&format!(
        "{}/recipes/{}/dependencies/{}",
        url, id, required_id
    )) {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn recipe_tag(url: &str, id: &str, label_name: &str) -> Option<()> {
    let mut params = HashMap::new();
    params.insert("name", label_name);
    match post(&format!("{}/recipes/{}/tags/add", url, id), params) {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn recipe_untag(url: &str, id: &str, label_id: &str) -> Option<()> {
    match delete(&format!("{}/recipes/{}/tags/{}", url, id, label_id)) {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn ingredient_index(url: &str, pattern: &str) -> Option<Vec<models::Ingredient>> {
    match get::<Vec<models::Ingredient>>(&format!("{}/ingredients?name={}", url, pattern)) {
        Ok(ingredients) => Some(ingredients),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn ingredient_get(url: &str, id: &str) -> Option<models::Ingredient> {
    match get::<models::Ingredient>(&format!("{}/ingredients/{}", url, id)) {
        Ok(ingredient) => Some(ingredient),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn ingredient_create(url: &str, name: &str) -> Option<models::Ingredient> {
    let mut params = HashMap::new();
    params.insert("name", name);

    match post::<models::Ingredient>(&format!("{}/ingredients/new", url), params) {
        Ok(ingredient) => Some(ingredient),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn ingredient_update(
    url: &str,
    id: &str,
    data: HashMap<&str, &str>,
) -> Option<models::Ingredient> {
    match put::<models::Ingredient>(&format!("{}/ingredients/{}", url, id), data) {
        Ok(ingredient) => Some(ingredient),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn ingredient_delete(url: &str, id: &str) -> Option<()> {
    match delete(&format!("{}/ingredients/{}", url, id)) {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn label_index(url: &str, pattern: &str) -> Option<Vec<models::Label>> {
    match get::<Vec<models::Label>>(&format!("{}/labels?name={}", url, pattern)) {
        Ok(labels) => Some(labels),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn label_get(url: &str, id: &str) -> Option<models::Label> {
    match get::<models::Label>(&format!("{}/labels/{}", url, id)) {
        Ok(label) => Some(label),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn label_create(url: &str, name: &str) -> Option<models::Label> {
    let mut params = HashMap::new();
    params.insert("name", name);

    match post::<models::Label>(&format!("{}/labels/new", url), params) {
        Ok(label) => Some(label),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn label_update(url: &str, id: &str, data: HashMap<&str, &str>) -> Option<models::Label> {
    match put::<models::Label>(&format!("{}/labels/{}", url, id), data) {
        Ok(label) => Some(label),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub fn label_delete(url: &str, id: &str) -> Option<()> {
    match delete(&format!("{}/labels/{}", url, id)) {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}
