use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub enum Category {
    Dining,
    Grocery,
    Travel,
    Merchandise,
    Entertainment,
    Other,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let print = match *self {
            Self::Dining => "Dining",
            Self::Grocery => "Grocery",
            Self::Travel => "Travel",
            Self::Merchandise => "Merchandise",
            Self::Entertainment => "Entertainment",
            Self::Other => "Other",
        };
        write!(f, "{}", print)
    }
}
