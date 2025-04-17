#[cfg(test)]
mod tests {
    use crate::api::app::create_app;
    use crate::api::routes::RegisterResponse;
    use crate::infra::db;
    use actix_web::body::to_bytes;
    use actix_web::http::header::ContentType;
    use actix_web::{test, App};
    use sqlx::Executor;

    const SECRET_KEY: &str = "53b65289550252052c61406f0f3dad24";

    #[actix_web::test]
    async fn test_index_get() {
        let pool = db::pg().await;

        let app =
            test::init_service(App::new().configure(create_app(pool.clone(), SECRET_KEY))).await;

        let req = test::TestRequest::post().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 404);
    }

    #[actix_web::test]
    async fn test_register_and_login() {
        let pool = db::pg().await;

        let app =
            test::init_service(App::new().configure(create_app(pool.clone(), SECRET_KEY))).await;

        pool.execute("truncate users cascade").await.unwrap();

        let req = test::TestRequest::post()
            .insert_header(ContentType::json())
            .set_payload("{\"name\": \"alex\", \"password\": \"123\"}")
            .uri("/register")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 200);

        let req = test::TestRequest::post()
            .insert_header(ContentType::json())
            .set_payload("{\"name\": \"alex\", \"password\": \"123\"}")
            .uri("/login")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 200);

        let b = to_bytes(resp.into_body()).await.unwrap();
        let b = std::str::from_utf8(&b).unwrap();

        let dto: RegisterResponse = serde_json::from_str(b).expect("Failed to parse json");
        assert_eq!(true, dto.token.len() > 0);
    }
}
