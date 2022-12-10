use reqwest::Client;
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

async fn get<'a, T: serde::de::DeserializeOwned>(url: &str) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    let response = client
        .get(url)
        .send()
        .await?
        .json::<models::Answer<T>>()
        .await?;

    match (response.accept, response.data) {
        (true, Some(object)) => Ok(object),
        (true, None) => Err(Box::new(KnifeError(String::from(
            "Internal error. This should not happen.",
        )))),
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

async fn post<'a, T: serde::de::DeserializeOwned>(
    url: &str,
    params: HashMap<&str, &str>,
) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    let response = client
        .post(url)
        .form(&params)
        .send()
        .await?
        .json::<models::Answer<T>>()
        .await?;

    match (response.accept, response.data) {
        (true, Some(object)) => Ok(object),
        (true, None) => Err(Box::new(KnifeError(String::from(
            "Internal error. This should not happen.",
        )))),
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

async fn put<'a, T: serde::de::DeserializeOwned + fmt::Debug>(
    url: &str,
    params: HashMap<&str, &str>,
) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    let response = client
        .put(url)
        .form(&params)
        .send()
        .await?
        .json::<models::Answer<T>>()
        .await?;

    match (response.accept, response.data) {
        (true, Some(object)) => Ok(object),
        (true, None) => Err(Box::new(KnifeError(String::from(
            "Internal error. This should not happen.",
        )))),
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

async fn delete(url: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let response = client
        .delete(url)
        .send()
        .await?
        .json::<models::Answer<()>>()
        .await?;

    match (response.accept, response.data) {
        (true, _) => Ok(()),
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

pub async fn recipe_index(url: &str, pattern: &str) -> Option<Vec<models::Recipe>> {
    let endpoint = format!("{}/recipes?name={}", url, pattern);
    let answer = get::<Vec<models::Recipe>>(&endpoint);

    match answer.await {
        Ok(recipes) => Some(recipes),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn recipe_get(url: &str, id: &str) -> Option<models::Recipe> {
    let endpoint = format!("{}/recipes/{}", url, id);
    let answer = get::<models::Recipe>(&endpoint);

    match answer.await {
        Ok(recipe) => Some(recipe),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn recipe_create(url: &str, name: &str) -> Option<models::Recipe> {
    let mut params = HashMap::new();
    params.insert("name", name);
    let endpoint = format!("{}/recipes/new", url);
    let answer = post::<models::Recipe>(&endpoint, params);

    match answer.await {
        Ok(recipe) => Some(recipe),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn recipe_update(
    url: &str,
    id: &str,
    data: HashMap<&str, &str>,
) -> Option<models::Recipe> {
    let endpoint = format!("{}/recipes/{}", url, id);
    let answer = put::<models::Recipe>(&endpoint, data);
    match answer.await {
        Ok(recipe) => Some(recipe),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn recipe_delete(url: &str, id: &str) -> Option<()> {
    let endpoint = format!("{}/recipes/{}", url, id);
    let answer = delete(&endpoint);
    match answer.await {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn recipe_link(url: &str, id: &str, required_id: &str) -> Option<()> {
    let mut params = HashMap::new();
    params.insert("required_id", required_id);
    let endpoint = format!("{}/recipes/{}/dependencies/add", url, id);
    let answer = post(&endpoint, params);
    match answer.await {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn recipe_unlink(url: &str, id: &str, required_id: &str) -> Option<()> {
    let endpoint = format!("{}/recipes/{}/dependencies/{}", url, id, required_id);
    let answer = delete(&endpoint);
    match answer.await {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn recipe_tag(url: &str, id: &str, label_name: &str) -> Option<()> {
    let mut params = HashMap::new();
    params.insert("name", label_name);
    let endpoint = format!("{}/recipes/{}/tags/add", url, id);
    let answer = post(&endpoint, params);
    match answer.await {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn recipe_untag(url: &str, id: &str, label_id: &str) -> Option<()> {
    let endpoint = format!("{}/recipes/{}/tags/{}", url, id, label_id);
    let answer = delete(&endpoint);
    match answer.await {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn ingredient_index(url: &str, pattern: &str) -> Option<Vec<models::Ingredient>> {
    let endpoint = format!("{}/ingredients?name={}", url, pattern);
    let answer = get::<Vec<models::Ingredient>>(&endpoint);
    match answer.await {
        Ok(ingredients) => Some(ingredients),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn ingredient_get(url: &str, id: &str) -> Option<models::Ingredient> {
    let endpoint = format!("{}/ingredients/{}", url, id);
    let answer = get::<models::Ingredient>(&endpoint);
    match answer.await {
        Ok(ingredient) => Some(ingredient),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn ingredient_create(url: &str, name: &str) -> Option<models::Ingredient> {
    let mut params = HashMap::new();
    params.insert("name", name);

    let endpoint = format!("{}/ingredients/new", url);
    let answer = post::<models::Ingredient>(&endpoint, params);
    match answer.await {
        Ok(ingredient) => Some(ingredient),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn ingredient_update(
    url: &str,
    id: &str,
    data: HashMap<&str, &str>,
) -> Option<models::Ingredient> {
    let endpoint = format!("{}/ingredients/{}", url, id);
    let answer = put::<models::Ingredient>(&endpoint, data);
    match answer.await {
        Ok(ingredient) => Some(ingredient),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn ingredient_delete(url: &str, id: &str) -> Option<()> {
    let endpoint = format!("{}/ingredients/{}", url, id);
    let answer = delete(&endpoint);
    match answer.await {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn label_index(url: &str, pattern: &str) -> Option<Vec<models::Label>> {
    let endpoint = format!("{}/labels?name={}", url, pattern);
    let answer = get::<Vec<models::Label>>(&endpoint);
    match answer.await {
        Ok(labels) => Some(labels),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn label_get(url: &str, id: &str) -> Option<models::Label> {
    let endpoint = format!("{}/labels/{}", url, id);
    let answer = get::<models::Label>(&endpoint);
    match answer.await {
        Ok(label) => Some(label),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn label_create(url: &str, name: &str) -> Option<models::Label> {
    let mut params = HashMap::new();
    params.insert("name", name);

    let endpoint = format!("{}/labels/new", url);
    let answer = post::<models::Label>(&endpoint, params);
    match answer.await {
        Ok(label) => Some(label),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn label_update(url: &str, id: &str, data: HashMap<&str, &str>) -> Option<models::Label> {
    let endpoint = format!("{}/labels/{}", url, id);
    let answer = put::<models::Label>(&endpoint, data);
    match answer.await {
        Ok(label) => Some(label),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}

pub async fn label_delete(url: &str, id: &str) -> Option<()> {
    let endpoint = format!("{}/labels/{}", url, id);
    let answer = delete(&endpoint);
    match answer.await {
        Ok(()) => Some(()),
        Err(e) => {
            eprintln!("{:?}", e);
            None
        }
    }
}
