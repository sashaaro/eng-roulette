#[cfg(test)]
mod tests {
    use crate::config_app;
    use crate::infra::db;
    use crate::infra::routes::RegisterResp;
    use actix_web::body::to_bytes;
    use actix_web::http::header::ContentType;
    use actix_web::{test, App};
    use sqlx::Executor;

    #[actix_web::test]
    async fn test_index_get() {
        let pool = db::pg().await;

        let app = test::init_service(App::new().configure(config_app(pool.clone()))).await;

        let req = test::TestRequest::post().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 404);
    }

    #[actix_web::test]
    async fn test_register_and_login() {
        let pool = db::pg().await;

        let app = test::init_service(App::new().configure(config_app(pool.clone()))).await;

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

        let dto: RegisterResp = serde_json::from_str(b).expect("Failed to parse json");
        assert_eq!(true, dto.token.len() > 0);
    }
}
