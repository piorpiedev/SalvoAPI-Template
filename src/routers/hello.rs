use salvo::{oapi::extract::QueryParam, prelude::*};

use crate::AppResult;

#[endpoint]
pub(super) async fn hello_get(name: QueryParam<String, false>) -> AppResult<String> {
    let name = name.as_deref().unwrap_or("World");
    Ok(format!("Hello {}", name))
}

#[cfg(test)]
mod tests {
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};

    use crate::config;

    #[tokio::test]
    async fn test_hello_get() {
        config::init();

        let service = Service::new(crate::routers::root());

        let content = TestClient::get(format!(
            "http://{}",
            config::get().listen_addr.replace("0.0.0.0", "127.0.0.1")
        ))
        .send(&service)
        .await
        .take_string()
        .await
        .unwrap();
        assert_eq!(content, "Hello World");
    }
}
