use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// Element of a recipe listing
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RecipeIndex {
    pub id: String,
    pub name: String,
}

/// Element of an ingredient listing
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct IngredientIndex {
    pub id: String,
    pub name: String,
}

/// Element of a label listing
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LabelIndex {
    pub id: String,
    pub name: String,
}

/// Label metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Label {
    pub id: String,
    pub name: String,

    /// List of recipe indexes tagged with this label
    #[serde(default)]
    pub tagged_recipes: Vec<RecipeIndex>,
}

/// Ingredient metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Ingredient {
    pub id: String,
    pub name: String,

    #[serde(default)]
    pub used_in: Vec<RecipeIndex>,
}

/// Requirement metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Requirement {
    pub ingredient: IngredientIndex,
    pub quantity: String,
}

/// Dependency metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Dependency {
    pub recipe: RecipeIndex,
    pub quantity: String,
    pub optional: bool,
}

/// Recipe metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Recipe {
    pub id: String,
    pub name: String,

    #[serde(default)]
    pub author: String,

    #[serde(default)]
    pub directions: String,

    /// List of requirements. Contains ingredient indexes
    #[serde(default)]
    pub requirements: HashSet<Requirement>,

    /// List of dependencies. Contains recipe indexes
    #[serde(default)]
    pub dependencies: HashSet<Dependency>,

    /// List of tags. Contains label indexes
    #[serde(default)]
    pub tags: HashSet<LabelIndex>,
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

impl Eq for LabelIndex {}

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

impl Eq for Label {}

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

impl Eq for IngredientIndex {}

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

impl Eq for Ingredient {}

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

impl Eq for RecipeIndex {}

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

impl Eq for Recipe {}

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

impl Eq for Requirement {}

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

impl Eq for Dependency {}
