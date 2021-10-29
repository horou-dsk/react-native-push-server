use std::path::{Path, PathBuf};
use rocket::{get, Route};
use rocket::fs::NamedFile;
use rocket::http::ContentType;
use rocket::response::status::NotFound;

#[get("/cargo.txt")]
async fn abc() -> (ContentType, &'static [u8]) {
    (ContentType::Bytes, "我日你个仙人".as_bytes())
}

#[get("/update/file/<file..>")]
async fn files(file: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new("static/").join(file);
    NamedFile::open(&path).await.map_err(|e| NotFound(e.to_string()))
}

#[inline]
pub fn routes() -> Vec<Route> {
    rocket::routes![abc, files]
}
