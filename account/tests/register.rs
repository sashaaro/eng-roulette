#[cfg(test)]
mod tests {
    use account::api::app::create_app;
    use account::api::routes::RegisterResponse;
    use account::infra::db;
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
        let body = |user, pwd: &str| -> String {
            format!(r#"{{"name": "{}", "password": "{}"}}"#, user, pwd)
        };
        let req = test::TestRequest::post()
            .insert_header(ContentType::json())
            .set_payload(body("alex", "123"))
            .uri("/register")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 200);

        let req = test::TestRequest::post()
            .insert_header(ContentType::json())
            .set_payload(body("alex", "wrong password"))
            .uri("/login")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400);
        assert_eq!(
            std::str::from_utf8(&to_bytes(resp.into_body()).await.unwrap()).unwrap(),
            "wrong password"
        );

        let req = test::TestRequest::post()
            .insert_header(ContentType::json())
            .set_payload(body("noexistuser", "wrong password"))
            .uri("/login")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400);
        assert_eq!(
            std::str::from_utf8(&to_bytes(resp.into_body()).await.unwrap()).unwrap(),
            "user not found"
        );

        let req = test::TestRequest::post()
            .insert_header(ContentType::json())
            .set_payload(body("alex", "123"))
            .uri("/login")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 200);

        let b = to_bytes(resp.into_body()).await.unwrap();
        let b = std::str::from_utf8(&b).unwrap();

        let dto: RegisterResponse = serde_json::from_str(b).expect("Failed to parse json");
        assert!(!dto.token.is_empty());

        let req = test::TestRequest::get()
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", dto.token),
            ))
            .uri("/me")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 200);

        let req = test::TestRequest::get()
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", dto.token + "wrong"),
            ))
            .uri("/me")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400);
    }
}
