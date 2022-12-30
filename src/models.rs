use serde::Deserialize;
use serde::Serialize;
use std::hash::{Hash, Hasher};

/// Element of a recipe listing
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct RecipeIndex {
    pub id: String,
    pub name: String,
}

/// Element of an ingredient listing
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct IngredientIndex {
    pub id: String,
    pub name: String,
}

/// Element of a label listing
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct LabelIndex {
    pub id: String,
    pub name: String,
}

/// Label metadata
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct Label {
    pub id: String,
    pub name: String,

    /// List of recipe indexes tagged with this label
    #[serde(default)]
    pub tagged_recipes: Vec<RecipeIndex>,
}

/// Ingredient metadata
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct Ingredient {
    pub id: String,
    pub name: String,

    #[serde(default)]
    pub used_in: Vec<RecipeIndex>,
}

/// Requirement metadata
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct Requirement {
    pub ingredient: IngredientIndex,
    pub quantity: String,
}

/// Recipe metadata
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct Recipe {
    pub id: String,
    pub name: String,

    #[serde(default)]
    pub author: String,

    #[serde(default)]
    pub directions: String,

    /// List of requirements. Contains ingredient indexes
    #[serde(default)]
    pub requirements: Vec<Requirement>,

    /// List of dependencies. Contains recipe indexes
    #[serde(default)]
    pub dependencies: Vec<RecipeIndex>,

    /// List of tags. Contains label indexes
    #[serde(default)]
    pub tags: Vec<LabelIndex>,
}

#[derive(Debug, Deserialize)]
pub struct Answer<T> {
    pub accept: bool,

    #[serde(default)]
    pub error: String,

    pub data: Option<T>,
}

impl Hash for LabelIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Hash for Label {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Hash for IngredientIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Hash for Ingredient {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Hash for RecipeIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Hash for Recipe {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
