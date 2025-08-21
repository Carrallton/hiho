use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {  
    pub name: String,     
    pub username: String, 
    pub password: String, 
}