use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

/// Element of a recipe listing
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialOrd, Ord, Eq)]
pub struct RecipeIndex {
    pub id: String,
    pub name: String,
}

/// Element of an ingredient listing
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialOrd, Ord, Eq)]
pub struct IngredientIndex {
    pub id: String,
    pub name: String,
}

/// Element of a label listing
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialOrd, Ord, Eq)]
pub struct LabelIndex {
    pub id: String,
    pub name: String,
}

/// Label metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq)]
pub struct Label {
    pub id: String,
    pub name: String,

    /// List of recipe indexes tagged with this label
    #[serde(default)]
    pub tagged_recipes: BTreeSet<RecipeIndex>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Classifications {
    pub dairy: bool,
    pub meat: bool,
    pub gluten: bool,
    pub animal_product: bool,
}

/// Ingredient metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq)]
pub struct Ingredient {
    pub id: String,
    pub name: String,

    #[serde(default)]
    pub classifications: Classifications,

    #[serde(default)]
    pub used_in: BTreeSet<RecipeIndex>,
}

/// Requirement metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialOrd, Ord, Eq)]
pub struct Requirement {
    pub ingredient: IngredientIndex,
    pub quantity: String,
    #[serde(default)]
    pub optional: bool,
}

/// Dependency metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialOrd, Ord, Eq)]
pub struct Dependency {
    pub recipe: RecipeIndex,
    pub quantity: String,
    #[serde(default)]
    pub optional: bool,
}

/// Recipe metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq)]
pub struct Recipe {
    pub id: String,
    pub name: String,

    #[serde(default)]
    pub author: String,

    #[serde(default)]
    pub directions: String,

    #[serde(default)]
    pub information: String,

    #[serde(default)]
    pub classifications: Classifications,

    /// List of requirements. Contains ingredient indexes
    #[serde(default)]
    pub requirements: BTreeSet<Requirement>,

    /// List of dependencies. Contains recipe indexes
    #[serde(default)]
    pub dependencies: BTreeSet<Dependency>,

    /// List of tags. Contains label indexes
    #[serde(default)]
    pub tags: BTreeSet<LabelIndex>,
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

impl PartialEq for LabelIndex {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Label {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for IngredientIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for IngredientIndex {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Ingredient {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Ingredient {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for RecipeIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for RecipeIndex {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Recipe {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Recipe {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Requirement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ingredient.hash(state);
    }
}

impl PartialEq for Requirement {
    fn eq(&self, other: &Self) -> bool {
        self.ingredient == other.ingredient
    }
}

impl Hash for Dependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.recipe.hash(state);
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.recipe == other.recipe
    }
}
