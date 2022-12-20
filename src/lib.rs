use reqwest::Client;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub mod models {

    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
    pub struct Label {
        pub id: String,
        pub name: String,

        #[serde(default)]
        pub tagged_recipes: Vec<Recipe>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
    pub struct Ingredient {
        pub id: String,
        pub name: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
    pub struct Requirement {
        pub ingredient: Ingredient,
        pub quantity: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
    pub struct Dependency {
        pub id: String,
        pub name: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
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

#[derive(Debug)]
struct LadleError(String);

impl fmt::Display for LadleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Internal error: {}", self.0)
    }
}

impl Error for LadleError {}

async fn get<'a, T: serde::de::DeserializeOwned>(url: &str) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    log::debug!("GET {}", url);

    let response = client
        .get(url)
        .send()
        .await?
        .json::<models::Answer<T>>()
        .await?;

    match (response.accept, response.data) {
        (true, Some(object)) => Ok(object),
        (true, None) => Err(Box::new(LadleError(String::from(
            "Failed to interpret the server's response",
        )))),
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

async fn post<'a, T: serde::de::DeserializeOwned + Any + Default>(
    url: &str,
    params: HashMap<&str, &str>,
) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    log::debug!("POST {} {:?}", url, params);

    let response = client
        .post(url)
        .form(&params)
        .send()
        .await?
        .json::<models::Answer<T>>()
        .await?;

    match (response.accept, response.data) {
        (true, Some(object)) => Ok(object),
        (true, None) => {
            if TypeId::of::<T>() == TypeId::of::<()>() {
                Ok(T::default())
            } else {
                Err(Box::new(LadleError(String::from(
                    "Failed to interpret the server's response",
                ))))
            }
        }
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

async fn put<'a, T: serde::de::DeserializeOwned + Any + Default>(
    url: &str,
    params: HashMap<&str, &str>,
) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    log::debug!("PUT {} {:?}", url, params);

    let response = client
        .put(url)
        .form(&params)
        .send()
        .await?
        .json::<models::Answer<T>>()
        .await?;

    match (response.accept, response.data) {
        (true, Some(object)) => Ok(object),
        (true, None) => {
            if TypeId::of::<T>() == TypeId::of::<()>() {
                Ok(T::default())
            } else {
                Err(Box::new(LadleError(String::from(
                    "Failed to interpret the server's response",
                ))))
            }
        }
        _ => Err(Box::new(KnifeError(response.error))),
    }
}

async fn delete(url: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    log::debug!("DELETE {}", url);

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

pub async fn recipe_index(url: &str, pattern: &str) -> Result<Vec<models::Recipe>, Box<dyn Error>> {
    let endpoint = format!("{}/recipes?name={}", url, pattern);
    let answer = get::<Vec<models::Recipe>>(&endpoint);

    answer.await
}

pub async fn recipe_get(url: &str, id: &str) -> Result<models::Recipe, Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}", url, id);
    let answer = get::<models::Recipe>(&endpoint);

    answer.await
}

pub async fn recipe_create(url: &str, name: &str) -> Result<models::Recipe, Box<dyn Error>> {
    let mut params = HashMap::new();
    params.insert("name", name);
    let endpoint = format!("{}/recipes/new", url);
    let answer = post::<models::Recipe>(&endpoint, params);

    answer.await
}

pub async fn recipe_update(
    url: &str,
    id: &str,
    data: HashMap<&str, &str>,
) -> Result<models::Recipe, Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}", url, id);
    let answer = put::<models::Recipe>(&endpoint, data);

    answer.await
}

pub async fn recipe_delete(url: &str, id: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}", url, id);
    let answer = delete(&endpoint);

    answer.await
}

pub async fn recipe_link(url: &str, id: &str, required_id: &str) -> Result<(), Box<dyn Error>> {
    let mut params = HashMap::new();
    params.insert("required_id", required_id);
    let endpoint = format!("{}/recipes/{}/dependencies/add", url, id);
    let answer = post(&endpoint, params);

    answer.await
}

pub async fn recipe_unlink(url: &str, id: &str, required_id: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}/dependencies/{}", url, id, required_id);
    let answer = delete(&endpoint);

    answer.await
}

pub async fn recipe_tag(url: &str, id: &str, label_name: &str) -> Result<(), Box<dyn Error>> {
    let mut params = HashMap::new();
    params.insert("name", label_name);
    let endpoint = format!("{}/recipes/{}/tags/add", url, id);
    let answer = post(&endpoint, params);

    answer.await
}

pub async fn recipe_untag(url: &str, id: &str, label_id: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}/tags/{}", url, id, label_id);
    let answer = delete(&endpoint);

    answer.await
}

pub async fn ingredient_index(
    url: &str,
    pattern: &str,
) -> Result<Vec<models::Ingredient>, Box<dyn Error>> {
    let endpoint = format!("{}/ingredients?name={}", url, pattern);
    let answer = get::<Vec<models::Ingredient>>(&endpoint);

    answer.await
}

pub async fn ingredient_get(url: &str, id: &str) -> Result<models::Ingredient, Box<dyn Error>> {
    let endpoint = format!("{}/ingredients/{}", url, id);
    let answer = get::<models::Ingredient>(&endpoint);

    answer.await
}

pub async fn ingredient_create(
    url: &str,
    name: &str,
) -> Result<models::Ingredient, Box<dyn Error>> {
    let mut params = HashMap::new();
    params.insert("name", name);

    let endpoint = format!("{}/ingredients/new", url);
    let answer = post::<models::Ingredient>(&endpoint, params);

    answer.await
}

pub async fn ingredient_update(
    url: &str,
    id: &str,
    data: HashMap<&str, &str>,
) -> Result<models::Ingredient, Box<dyn Error>> {
    let endpoint = format!("{}/ingredients/{}", url, id);
    let answer = put::<models::Ingredient>(&endpoint, data);

    answer.await
}

pub async fn ingredient_delete(url: &str, id: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/ingredients/{}", url, id);
    let answer = delete(&endpoint);

    answer.await
}

pub async fn label_index(url: &str, pattern: &str) -> Result<Vec<models::Label>, Box<dyn Error>> {
    let endpoint = format!("{}/labels?name={}", url, pattern);
    let answer = get::<Vec<models::Label>>(&endpoint);

    answer.await
}

pub async fn label_get(url: &str, id: &str) -> Result<models::Label, Box<dyn Error>> {
    let endpoint = format!("{}/labels/{}", url, id);
    let answer = get::<models::Label>(&endpoint);

    answer.await
}

pub async fn label_create(url: &str, name: &str) -> Result<models::Label, Box<dyn Error>> {
    let mut params = HashMap::new();
    params.insert("name", name);

    let endpoint = format!("{}/labels/new", url);
    let answer = post::<models::Label>(&endpoint, params);

    answer.await
}

pub async fn label_update(
    url: &str,
    id: &str,
    data: HashMap<&str, &str>,
) -> Result<models::Label, Box<dyn Error>> {
    let endpoint = format!("{}/labels/{}", url, id);
    let answer = put::<models::Label>(&endpoint, data);

    answer.await
}

pub async fn label_delete(url: &str, id: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/labels/{}", url, id);
    let answer = delete(&endpoint);

    answer.await
}

pub async fn requirement_create(
    url: &str,
    recipe_id: &str,
    ingredient_id: &str,
    quantity: &str,
) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}/requirements/add", url, recipe_id);
    let mut params = HashMap::new();
    params.insert("quantity", quantity);
    params.insert("ingredient_id", ingredient_id);

    post::<()>(&endpoint, params).await
}

pub async fn requirement_update(
    url: &str,
    recipe_id: &str,
    ingredient_id: &str,
    quantity: &str,
) -> Result<(), Box<dyn Error>> {
    let endpoint = format!(
        "{}/recipes/{}/requirements/{}",
        url, recipe_id, ingredient_id
    );
    let mut params = HashMap::new();
    params.insert("quantity", quantity);

    put::<()>(&endpoint, params).await
}

pub async fn requirement_delete(
    url: &str,
    recipe_id: &str,
    ingredient_id: &str,
) -> Result<(), Box<dyn Error>> {
    let endpoint = format!(
        "{}/recipes/{}/requirements/{}",
        url, recipe_id, ingredient_id
    );

    delete(&endpoint).await
}

pub async fn requirement_create_from_ingredient_name(
    url: &str,
    recipe_id: &str,
    ingredient: &str,
    quantity: &str,
) -> Result<(), Box<dyn Error>> {
    let sanitized_name = unidecode::unidecode(ingredient.to_lowercase().as_str());
    let endpoint = format!("{}/ingredients?name={}", url, sanitized_name.as_str());
    let lookup = get::<Vec<models::Ingredient>>(&endpoint).await?;

    let exact_matches = lookup
        .iter()
        .filter(|i| unidecode::unidecode(i.name.to_lowercase().as_str()) == sanitized_name)
        .collect::<Vec<&models::Ingredient>>();

    let ingredient_id: String;
    log::debug!(
        "Comparing {} to {:?}",
        ingredient,
        lookup
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<&str>>()
    );

    if exact_matches.len() == 1 {
        ingredient_id = exact_matches.first().unwrap().id.clone();
    } else {
        let mut params = HashMap::new();
        params.insert("name", ingredient);
        let endpoint = format!("{}/ingredients/new", url);
        let ingredient = post::<models::Ingredient>(&endpoint, params).await?;
        ingredient_id = ingredient.id;
    };

    let endpoint = format!("{}/recipes/{}/requirements/add", url, recipe_id);
    let mut params = HashMap::new();
    params.insert("quantity", quantity);
    params.insert("ingredient_id", ingredient_id.as_str());

    post::<()>(&endpoint, params).await
}
