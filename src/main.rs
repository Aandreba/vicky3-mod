pub mod data;
pub(crate) mod utils;

pub(crate) type Str = Box<str>;
pub type Result<T> = ::core::result::Result<T, jomini::Error>;

#[tokio::main]
async fn main () {

}