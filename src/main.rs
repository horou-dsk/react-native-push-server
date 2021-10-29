use react_native_push_server::router::{index, update};
use react_native_push_server::setup_logger;

#[rocket::main]
async fn main() -> Result<(), rocket::error::Error> {
    setup_logger().expect("初始化日志系统失败！");
    rocket::build()
        .mount("/", index::routes())
        .mount("/update", update::routes())
        .launch()
        .await
}
