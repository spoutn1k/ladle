use reqwest::{Client, StatusCode};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::any::Any;
use std::error::Error;
use std::fmt;

pub mod models;

#[derive(Debug)]
struct KnifeError(StatusCode, String);

impl fmt::Display for KnifeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Server returned an error: [{}] {}", self.0, self.1)
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

    let response = client.get(url).send().await?;
    let status_code = response.status();

    log::debug!("GET {} -> {}", url, status_code);

    let answer = response.json::<models::Answer<T>>().await?;

    match (status_code, answer.data) {
        (StatusCode::OK, Some(object)) => Ok(object),
        (StatusCode::OK, None) => Err(Box::new(LadleError(String::from(
            "Failed to interpret the server's response",
        )))),
        (status, _) => Err(Box::new(KnifeError(status, answer.error))),
    }
}

/// Send a POST request to a knife server. Hijack the 409 CONFLICT status to get info on existing
/// data
async fn post<'a, P: Serialize + fmt::Debug, T: serde::de::DeserializeOwned + Any + Default>(
    url: &str,
    params: P,
) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    let response = client.post(url).json(&params).send().await?;
    let status_code = response.status();

    log::debug!("POST {} {:?} -> {}", url, params, status_code);

    let answer = response.json::<models::Answer<T>>().await?;

    match (status_code, answer.data) {
        (StatusCode::OK, Some(object))
        | (StatusCode::CREATED, Some(object))
        | (StatusCode::CONFLICT, Some(object)) => Ok(object),
        (StatusCode::OK, None) | (StatusCode::CREATED, None) => Ok(T::default()),
        (status, _) => Err(Box::new(KnifeError(status, answer.error))),
    }
}

async fn put<'a, P: Serialize + fmt::Debug, T: serde::de::DeserializeOwned + Any + Default>(
    url: &str,
    params: P,
) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    let response = client.put(url).json(&params).send().await?;
    let status_code = response.status();

    log::debug!("PUT {} {:?} -> {}", url, params, status_code);

    let answer = response.json::<models::Answer<T>>().await?;

    match (status_code, answer.data) {
        (StatusCode::OK, Some(object))
        | (StatusCode::CREATED, Some(object))
        | (StatusCode::ACCEPTED, Some(object))
        | (StatusCode::CONFLICT, Some(object)) => Ok(object),
        (StatusCode::OK, None) | (StatusCode::CREATED, None) | (StatusCode::ACCEPTED, None) => {
            Ok(T::default())
        }
        (status, _) => Err(Box::new(KnifeError(status, answer.error))),
    }
}

async fn delete(url: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let response = client.delete(url).send().await?;
    let status_code = response.status();

    log::debug!("DELETE {} -> {}", url, status_code);

    let answer = response.json::<models::Answer<()>>().await?;

    match (status_code, answer.data) {
        (StatusCode::OK, _) => Ok(()),
        (status, _) => Err(Box::new(KnifeError(status, answer.error))),
    }
}

pub async fn recipe_index(
    url: &str,
    pattern: &str,
) -> Result<Vec<models::RecipeIndex>, Box<dyn Error>> {
    let endpoint = format!("{}/recipes?name={}", url, pattern);
    let answer = get::<Vec<models::RecipeIndex>>(&endpoint);

    answer.await
}

pub async fn recipe_get(url: &str, id: &str) -> Result<models::Recipe, Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}", url, id);
    let answer = get::<models::Recipe>(&endpoint);

    answer.await
}

pub async fn recipe_create(
    url: &str,
    name: &str,
    author: &str,
    directions: &str,
    information: &str,
) -> Result<models::Recipe, Box<dyn Error>> {
    let params = json!({
        "name": name,
        "author": author,
        "directions": directions,
        "information": information
    });
    let endpoint = format!("{}/recipes/new", url);
    post(&endpoint, params).await
}

pub async fn recipe_update(
    url: &str,
    id: &str,
    name: Option<&str>,
    author: Option<&str>,
    directions: Option<&str>,
    information: Option<&str>,
) -> Result<models::Recipe, Box<dyn Error>> {
    let mut params = Value::Object(Map::default());
    if let Some(value) = name {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("name"), Value::String(String::from(value)));
    }
    if let Some(value) = author {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("author"), Value::String(String::from(value)));
    }
    if let Some(value) = directions {
        params.as_object_mut().unwrap().insert(
            String::from("directions"),
            Value::String(String::from(value)),
        );
    }
    if let Some(value) = information {
        params.as_object_mut().unwrap().insert(
            String::from("information"),
            Value::String(String::from(value)),
        );
    }

    let endpoint = format!("{}/recipes/{}", url, id);
    put(&endpoint, params).await
}

pub async fn recipe_delete(url: &str, id: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}", url, id);
    let answer = delete(&endpoint);

    answer.await
}

pub async fn dependency_create(
    url: &str,
    id: &str,
    required_id: &str,
    quantity: &str,
    optional: bool,
) -> Result<(), Box<dyn Error>> {
    let params = json!({
        "requisite": required_id,
        "quantity": quantity,
        "optional": optional,
    });
    let endpoint = format!("{}/recipes/{}/dependencies/add", url, id);
    let answer = post(&endpoint, params);

    answer.await
}

pub async fn dependency_edit(
    url: &str,
    id: &str,
    required_id: &str,
    quantity: Option<&str>,
    optional: Option<bool>,
) -> Result<(), Box<dyn Error>> {
    let mut params = Value::Object(Map::default());
    if let Some(value) = quantity {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("quantity"), Value::String(String::from(value)));
    }
    if let Some(value) = optional {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("optional"), Value::Bool(value));
    }

    let endpoint = format!("{}/recipes/{}/dependencies/{}", url, id, required_id);
    put(&endpoint, params).await
}

pub async fn dependency_delete(
    url: &str,
    id: &str,
    required_id: &str,
) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}/dependencies/{}", url, id, required_id);
    delete(&endpoint).await
}

pub async fn recipe_tag(url: &str, id: &str, label_name: &str) -> Result<(), Box<dyn Error>> {
    let params = json!({ "name": label_name });
    let endpoint = format!("{}/recipes/{}/tags/add", url, id);
    post(&endpoint, params).await
}

pub async fn recipe_untag(url: &str, id: &str, label_id: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}/tags/{}", url, id, label_id);
    delete(&endpoint).await
}

pub async fn recipe_get_requirements(
    url: &str,
    id: &str,
) -> Result<Vec<models::Requirement>, Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}/requirements", url, id);
    get::<Vec<models::Requirement>>(&endpoint).await
}

pub async fn ingredient_index(
    url: &str,
    pattern: &str,
) -> Result<Vec<models::IngredientIndex>, Box<dyn Error>> {
    let endpoint = format!("{}/ingredients?name={}", url, pattern);
    get(&endpoint).await
}

pub async fn ingredient_get(url: &str, id: &str) -> Result<models::Ingredient, Box<dyn Error>> {
    let endpoint = format!("{}/ingredients/{}", url, id);
    get(&endpoint).await
}

pub async fn ingredient_create(
    url: &str,
    name: &str,
    dairy: bool,
    meat: bool,
    gluten: bool,
    animal_product: bool,
) -> Result<models::IngredientIndex, Box<dyn Error>> {
    let params = json!({
        "name": name,
        "dairy": dairy,
        "meat": meat,
        "gluten": gluten,
        "animal_product": animal_product
    });
    let endpoint = format!("{}/ingredients/new", url);

    post(&endpoint, params).await
}

pub async fn ingredient_update(
    url: &str,
    id: &str,
    name: Option<&str>,
    dairy: Option<bool>,
    meat: Option<bool>,
    gluten: Option<bool>,
    animal_product: Option<bool>,
) -> Result<(), Box<dyn Error>> {
    let mut params = Value::Object(Map::default());
    if let Some(value) = name {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("name"), Value::String(String::from(value)));
    }

    if let Some(value) = dairy {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("dairy"), Value::Bool(value));
    }

    if let Some(value) = meat {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("meat"), Value::Bool(value));
    }

    if let Some(value) = gluten {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("gluten"), Value::Bool(value));
    }

    if let Some(value) = animal_product {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("animal_product"), Value::Bool(value));
    }

    let endpoint = format!("{}/ingredients/{}", url, id);

    put(&endpoint, params).await
}

pub async fn ingredient_delete(url: &str, id: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/ingredients/{}", url, id);
    delete(&endpoint).await
}

pub async fn label_index(
    url: &str,
    pattern: &str,
) -> Result<Vec<models::LabelIndex>, Box<dyn Error>> {
    let endpoint = format!("{}/labels?name={}", url, pattern);
    get(&endpoint).await
}

pub async fn label_get(url: &str, id: &str) -> Result<models::Label, Box<dyn Error>> {
    let endpoint = format!("{}/labels/{}", url, id);
    get(&endpoint).await
}

pub async fn label_create(url: &str, name: &str) -> Result<models::LabelIndex, Box<dyn Error>> {
    let params = json!({ "name": name });

    let endpoint = format!("{}/labels/new", url);
    post(&endpoint, params).await
}

pub async fn label_update(
    url: &str,
    id: &str,
    name: &str,
) -> Result<models::LabelIndex, Box<dyn Error>> {
    let params = json!({ "name": name });
    let endpoint = format!("{}/labels/{}", url, id);
    put(&endpoint, params).await
}

pub async fn label_delete(url: &str, id: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/labels/{}", url, id);
    delete(&endpoint).await
}

pub async fn requirement_create(
    url: &str,
    recipe_id: &str,
    ingredient_id: &str,
    quantity: &str,
    optional: bool,
) -> Result<(), Box<dyn Error>> {
    let params = json!({
        "quantity": quantity,
        "optional": optional,
        "ingredient_id": ingredient_id,
    });
    let endpoint = format!("{}/recipes/{}/requirements/add", url, recipe_id);
    post(&endpoint, params).await
}

pub async fn requirement_update(
    url: &str,
    recipe_id: &str,
    ingredient_id: &str,
    quantity: Option<&str>,
    optional: Option<bool>,
) -> Result<(), Box<dyn Error>> {
    let mut params = Value::Object(Map::default());
    if let Some(value) = quantity {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("quantity"), Value::String(String::from(value)));
    }
    if let Some(value) = optional {
        params
            .as_object_mut()
            .unwrap()
            .insert(String::from("optional"), Value::Bool(value));
    }
    let endpoint = format!(
        "{}/recipes/{}/requirements/{}",
        url, recipe_id, ingredient_id
    );

    put(&endpoint, params).await
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
