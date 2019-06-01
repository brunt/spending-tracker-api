use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub enum Category {
    Dining,
    Travel,
    Merchandise,
    Entertainment,
    Other,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let print = match *self {
            Category::Dining => "Dining",
            Category::Travel => "Travel",
            Category::Merchandise => "Merchandise",
            Category::Entertainment => "Entertainment",
            Category::Other => "Other",
        };
        write!(f, "{}", print)
    }
}
