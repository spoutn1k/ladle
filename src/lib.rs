use reqwest::{Client, StatusCode};
use std::any::Any;
use std::collections::HashMap;
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
async fn post<
    'a,
    P: serde::Serialize + fmt::Debug,
    T: serde::de::DeserializeOwned + Any + Default,
>(
    url: &str,
    params: P,
) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    let response = client.post(url).form(&params).send().await?;
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

async fn put<
    'a,
    P: serde::Serialize + fmt::Debug,
    T: serde::de::DeserializeOwned + Any + Default,
>(
    url: &str,
    params: P,
) -> Result<T, Box<dyn Error>> {
    let client = Client::new();

    let response = client.put(url).form(&params).send().await?;
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
    params: HashMap<&str, &str>,
) -> Result<models::Recipe, Box<dyn Error>> {
    let endpoint = format!("{}/recipes/new", url);
    let answer = post(&endpoint, params);

    answer.await
}

pub async fn recipe_update(
    url: &str,
    id: &str,
    data: HashMap<&str, &str>,
) -> Result<models::Recipe, Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}", url, id);
    let answer = put(&endpoint, data);

    answer.await
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
    let params = serde_json::json!({
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
    let params = match (quantity, optional) {
        (Some(qt), Some(op)) => serde_json::json!({"quantity":qt, "optional": op}),
        (Some(qt), None) => serde_json::json!({ "quantity": qt }),
        (None, Some(op)) => serde_json::json!({ "optional": op }),
        (None, None) => serde_json::json!({}),
    };

    let endpoint = format!("{}/recipes/{}/dependencies/{}", url, id, required_id);
    let answer = put(&endpoint, params);

    answer.await
}

pub async fn dependency_delete(
    url: &str,
    id: &str,
    required_id: &str,
) -> Result<(), Box<dyn Error>> {
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

pub async fn recipe_get_requirements(
    url: &str,
    id: &str,
) -> Result<Vec<models::Requirement>, Box<dyn Error>> {
    let endpoint = format!("{}/recipes/{}/requirements", url, id);
    let answer = get::<Vec<models::Requirement>>(&endpoint);

    answer.await
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
) -> Result<models::IngredientIndex, Box<dyn Error>> {
    let params = HashMap::from([("name", name)]);
    let endpoint = format!("{}/ingredients/new", url);

    post(&endpoint, params).await
}

pub async fn ingredient_update(
    url: &str,
    id: &str,
    data: HashMap<&str, &str>,
) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/ingredients/{}", url, id);

    put(&endpoint, data).await
}

pub async fn ingredient_delete(url: &str, id: &str) -> Result<(), Box<dyn Error>> {
    let endpoint = format!("{}/ingredients/{}", url, id);
    let answer = delete(&endpoint);

    answer.await
}

pub async fn label_index(
    url: &str,
    pattern: &str,
) -> Result<Vec<models::LabelIndex>, Box<dyn Error>> {
    let endpoint = format!("{}/labels?name={}", url, pattern);
    let answer = get(&endpoint);

    answer.await
}

pub async fn label_get(url: &str, id: &str) -> Result<models::Label, Box<dyn Error>> {
    let endpoint = format!("{}/labels/{}", url, id);
    let answer = get(&endpoint);

    answer.await
}

pub async fn label_create(url: &str, name: &str) -> Result<models::LabelIndex, Box<dyn Error>> {
    let mut params = HashMap::new();
    params.insert("name", name);

    let endpoint = format!("{}/labels/new", url);
    let answer = post(&endpoint, params);

    answer.await
}

pub async fn label_update(
    url: &str,
    id: &str,
    data: HashMap<&str, &str>,
) -> Result<models::LabelIndex, Box<dyn Error>> {
    let endpoint = format!("{}/labels/{}", url, id);
    let answer = put(&endpoint, data);

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
    let params = HashMap::from([("quantity", quantity), ("ingredient_id", ingredient_id)]);

    post(&endpoint, params).await
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
        let ingredient = post::<_, models::IngredientIndex>(&endpoint, params).await?;
        ingredient_id = ingredient.id;
    };

    let endpoint = format!("{}/recipes/{}/requirements/add", url, recipe_id);
    let mut params = HashMap::new();
    params.insert("quantity", quantity);
    params.insert("ingredient_id", ingredient_id.as_str());

    post(&endpoint, params).await
}
