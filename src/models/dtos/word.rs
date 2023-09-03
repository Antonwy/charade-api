use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct NewWordDto {
    #[validate(length(min = 3, max = 30, message = "Wrong word length"))]
    pub word: String,
}
