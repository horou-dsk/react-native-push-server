pub mod index;
pub mod update;

use rocket::{Request, Response, response::Responder, http::ContentType};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ResultOk<T> {
    code: u16,
    data: T,
}

impl<T> ResultOk<T> {
    fn new(data: T) -> Self {
        ResultOk { code: 200, data }
    }
}

#[derive(Serialize, Deserialize)]
struct ResultErr<'r> {
    code: u16,
    err_msg: &'r str,
}

impl<'r> ResultErr<'r> {
    fn new(code: u16, err_msg: &'r str) -> Self {
        ResultErr { code, err_msg }
    }
}

//
// pub struct ResultJson;
//
// impl ResultJson {
//     fn ok<T>(data: T) -> ResultOk<T> {
//         ResultOk::new(data)
//     }
//
//     fn err(code: u16, err_msg: &str) -> ResultErr {
//         ResultErr::new(code, err_msg)
//     }
// }


#[derive(Debug)]
pub struct ApiResponse {
    json_str: String,
}

impl ApiResponse {
    #[inline]
    pub fn new<T: ?Sized + Serialize>(json: &T) -> Self {
        Self {
            json_str: serde_json::to_string(json).unwrap()
        }
    }

    #[inline]
    pub fn json<T: Serialize>(json: T) -> Self {
        Self::new(&json)
    }

    #[inline]
    pub fn ok<T: ?Sized + Serialize>(data: &T) -> Self {
        Self::json(ResultOk::new(data))
    }

    #[inline]
    pub fn err(code: u16, err_msg: &str) -> Self {
        Self::json(ResultErr::new(code, err_msg))
    }
}

impl<'r> Responder<'r, 'r> for ApiResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        Response::build_from(self.json_str.respond_to(&request).unwrap())
            .header(ContentType::JSON)
            .ok()
    }
}
