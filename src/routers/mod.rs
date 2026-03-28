use salvo::prelude::*;

use crate::hoops;

mod hello;

pub fn root() -> Router {
    let router = Router::new()
        .hoop(Logger::new())
        .get(hello::hello_get)
        .hoop(hoops::cors::cors_hoop());

    let doc = OpenApi::new("SalvoApi Template", "0.0.1").merge_router(&router);
    router
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(Scalar::new("/api-doc/openapi.json").into_router("scalar"))
}
