use isbot::Bots;

use actix_web::{
    dev::Service,
    http::header::{HeaderMap, USER_AGENT},
    web, App, HttpRequest, HttpResponse, HttpServer,
};
use futures::{future, future::Either, future::FutureExt};

struct AppState {
    bots: Bots,
}

async fn index(_: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body("Home")
}

async fn login(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    if let Some(user_agent) = get_user_agent(req.headers()) {
        if data.bots.is_bot(user_agent) {
            return HttpResponse::Forbidden().body("Bots not allowed");
        }
    }
    HttpResponse::Ok().body("Login")
}

fn get_user_agent(header_map: &HeaderMap) -> Option<&str> {
    header_map.get(USER_AGENT)?.to_str().ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(AppState {
                bots: Bots::default(),
            })
            .wrap_fn(|sreq, srv| {
                // Example middleware wrapper to exclude bots from all routes
                if let Some(data) = sreq.app_data::<web::Data<AppState>>() {
                    if let Some(user_agent) = get_user_agent(sreq.headers()) {
                        if data.bots.is_bot(user_agent) {
                            // Return a 403 indicating bots aren't allowed
                            return Either::Right(future::ready(Ok(sreq.into_response(
                                HttpResponse::Forbidden().body("Bots not allowed"),
                            ))));
                        }
                    }
                }
                Either::Left(srv.call(sreq).map(|res| res))
            })
            .route("/", web::get().to(index))
            .route("/login", web::get().to(login))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, web, App};
    use bytes::Bytes;

    const VALID_BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36";
    const KNOWN_BOT_USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 9_1 like Mac OS X) AppleWebKit/601.1.46 (KHTML, like Gecko) Version/9.0 Mobile/13B143 Safari/601.1 (compatible; AdsBot-Google-Mobile; +http://www.google.com/mobile/adsbot.html)";
    const CHROME_LIGHTHOUSE_BOT_USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 7.0; Moto G (4)) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/84.0.4143.7 Mobile Safari/537.36 Chrome-Lighthouse";

    /// Alternative approach where the handler function calls `is_bot()` directly,
    /// instead of using middleware
    async fn account(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
        if let Some(user_agent) = get_user_agent(&req.headers()) {
            if data.bots.is_bot(user_agent) {
                return HttpResponse::Forbidden().body("Bots not allowed");
            }
        }
        HttpResponse::Ok().body("Account")
    }

    #[actix_rt::test]
    async fn test_valid_browser() {
        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    bots: Bots::default(),
                }))
                .route("/account", web::get().to(account)),
        )
        .await;
        let req = test::TestRequest::with_uri("/account")
            .insert_header((USER_AGENT, VALID_BROWSER_USER_AGENT))
            .to_request();
        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());

        let result = test::read_body(res).await;
        assert_eq!(result, Bytes::from_static(b"Account"))
    }

    #[actix_rt::test]
    async fn test_known_bot() {
        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    bots: Bots::default(),
                }))
                .route("/account", web::get().to(account)),
        )
        .await;
        let req = test::TestRequest::with_uri("/account")
            .insert_header((USER_AGENT, KNOWN_BOT_USER_AGENT))
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_rt::test]
    async fn test_exclude_known_bot() {
        let mut bots = Bots::default();
        bots.remove(&["Chrome-Lighthouse"]); // Example: remove Chrome Lighthouse for performance testing

        let data = web::Data::new(AppState { bots });
        let mut app = test::init_service(
            App::new()
                .app_data(data.clone())
                .route("/account", web::get().to(account)),
        )
        .await;
        let req = test::TestRequest::with_uri("/account")
            .insert_header((USER_AGENT, CHROME_LIGHTHOUSE_BOT_USER_AGENT))
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_middleware_known_bot() {
        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    bots: Bots::default(),
                }))
                .wrap_fn(|sreq, srv| {
                    if let Some(data) = sreq.app_data::<web::Data<AppState>>() {
                        if let Some(user_agent) = get_user_agent(sreq.headers()) {
                            if data.bots.is_bot(user_agent) {
                                return Either::Right(future::ready(Ok(sreq.into_response(
                                    HttpResponse::Forbidden().body("Bots not allowed"),
                                ))));
                            }
                        }
                    }
                    Either::Left(srv.call(sreq).map(|res| res))
                })
                .route("/account", web::get().to(account)),
        )
        .await;
        let req = test::TestRequest::with_uri("/account")
            .insert_header((USER_AGENT, KNOWN_BOT_USER_AGENT))
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_rt::test]
    async fn test_post_known_bot() {
        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    bots: Bots::default(),
                }))
                .route("/account", web::post().to(account)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/account")
            .insert_header((USER_AGENT, KNOWN_BOT_USER_AGENT))
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
