use rocket::{Route, FromForm, fs::TempFile, http::{ContentType}, get, post, form::Form};
use tokio::io;
use crate::router::ApiResponse;
use crate::service::update::UpdateProject;

#[derive(Debug, FromForm)]
struct UploadUpdate<'r> {
    pkg_version: &'r str,
    pkg_url: Option<String>,
    // #[field(validate = ext(ContentType::ZIP))]
    file: Option<TempFile<'r>>,
    #[field(default = false)]
    mandatory: bool,
    update_log: &'r str,
}

#[post("/upload/<project_name>", data = "<data>")]
async fn upload_update<'r>(project_name: &'r str, data: Form<UploadUpdate<'r>>) -> io::Result<ApiResponse> {
    if let Some(file) = &data.file {
        if let Err(errors) = rocket::form::validate::ext(file, ContentType::ZIP) {
            if let Some(error) = errors.last() {
                return Ok(ApiResponse::err(500, &error.kind.to_string()))
            }
            // log::error!("{:?}", error);
        }
    }
    let data = data.into_inner();
    let mut update_project = UpdateProject::new(project_name).await?;
    update_project.add(
        data.pkg_version,
        data.file,
        data.pkg_url,
        data.mandatory,
        data.update_log
    ).await?;
    update_project.save().await?;
    Ok(ApiResponse::ok("success！"))
}

#[get("/remove/<project_name>?<pkg_version>&<hot_version>")]
async fn remove(project_name: &str, pkg_version: &str, hot_version: u32) -> Result<ApiResponse, tokio::io::Error> {
    let mut update_project = UpdateProject::new(project_name).await?;
    update_project.remove(pkg_version, hot_version).await?;
    update_project.save().await?;
    Ok(ApiResponse::ok("Success!"))
}

#[get("/new_version/<project_name>?<pkg_version>&<hot_version>")]
async fn new_version(project_name: &str, pkg_version: &str, hot_version: u32) -> Result<ApiResponse, tokio::io::Error> {
    let update_project = UpdateProject::new(project_name).await?;
    if let Some(up) = update_project.get_new(pkg_version, hot_version) {
        let download_url = update_project.get_download_path(pkg_version, up.version);
        Ok(ApiResponse::ok(&serde_json::json!({
            "mandatory": up.mandatory,
            "version": up.version,
            "downloadUrl": download_url,
            "pkgUrl": up.pkg_url,
            "updateLog": up.update_log,
        })))
    } else {
        Ok(ApiResponse::err(404, "暂无更新！"))
    }
}

#[inline]
pub fn routes() -> Vec<Route> {
    rocket::routes![upload_update, remove, new_version]
}
