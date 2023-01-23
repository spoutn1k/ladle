use ladle::models::Classifications;
use std::error;

pub fn display_classifications(
    class: &Classifications,
) -> Result<Vec<String>, Box<dyn error::Error>> {
    let mut terms = vec![];

    if class.animal_product && !class.meat {
        terms.push("produits d'origine animale".to_string());
    }

    if class.meat {
        terms.push("viande".to_string());
    }

    if class.dairy {
        terms.push("produits laitiers".to_string());
    }

    if class.gluten {
        terms.push("gluten".to_string());
    }

    Ok(terms)
}
