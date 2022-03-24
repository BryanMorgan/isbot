use isbot::Bots;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::status::Forbidden;
use rocket::State;

const USER_AGENT: &str = "user-agent";

#[macro_use]
extern crate rocket;

struct AppState {
    bots: Bots,
}

struct UserAgent(String);
struct Browser;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgent {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, ()> {
        Outcome::Success(UserAgent(
            request
                .headers()
                .get_one(USER_AGENT)
                .unwrap_or("")
                .to_string(),
        ))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Browser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, ()> {
        if let Some(user_agent) = request.headers().get_one(USER_AGENT) {
            if request
                .rocket()
                .state::<AppState>()
                .map_or(false, |state| state.bots.is_bot(user_agent))
            {
                return Outcome::Failure((Status::Forbidden, ()));
            }
        }

        Outcome::Success(Browser)
    }
}

/// Only respond to real browsers. Bots get a 403 (Forbidden).
#[get("/")]
fn index(_browser: Browser) -> &'static str {
    "Home"
}

/// Check if the user-agent is a bot and if so return a 403 (Forbidden)
#[get("/login")]
fn login(
    state: &State<AppState>,
    user_agent: UserAgent,
) -> Result<&'static str, Forbidden<&'static str>> {
    if state.bots.is_bot(&user_agent.0) {
        return Err(Forbidden(Some("Bots not allowed")));
    }

    Ok("Login")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(AppState {
            bots: Bots::default(),
        })
        .mount("/", routes![index, login])
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::http::Header;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    const VALID_BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36";
    const KNOWN_BOT_USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 9_1 like Mac OS X) AppleWebKit/601.1.46 (KHTML, like Gecko) Version/9.0 Mobile/13B143 Safari/601.1 (compatible; AdsBot-Google-Mobile; +http://www.google.com/mobile/adsbot.html)";
    const CHROME_LIGHTHOUSE_BOT_USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 7.0; Moto G (4)) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/84.0.4143.7 Mobile Safari/537.36 Chrome-Lighthouse";

    #[test]
    fn test_valid_browser() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let mut response = client
            .get("/")
            .header(Header::new(USER_AGENT, VALID_BROWSER_USER_AGENT))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Home");

        response = client
            .get("/login")
            .header(Header::new(USER_AGENT, VALID_BROWSER_USER_AGENT))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Login");
    }

    #[test]
    fn test_known_bot() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let mut response = client
            .get("/")
            .header(Header::new(USER_AGENT, KNOWN_BOT_USER_AGENT))
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);

        response = client
            .get("/login")
            .header(Header::new(USER_AGENT, KNOWN_BOT_USER_AGENT))
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);
    }

    #[test]
    fn test_exclude_known_bot() {
        let mut bots = Bots::default();
        bots.remove(&["Chrome-Lighthouse"]); // Example: remove Chrome Lighthouse for performance testing

        let rocket = rocket::build()
            .manage(AppState { bots })
            .mount("/", routes![index, login]);

        let client = Client::tracked(rocket).expect("valid rocket instance");
        let mut response = client
            .get("/")
            .header(Header::new(USER_AGENT, CHROME_LIGHTHOUSE_BOT_USER_AGENT))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Home");

        response = client
            .get("/login")
            .header(Header::new(USER_AGENT, KNOWN_BOT_USER_AGENT))
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);
    }

    #[test]
    fn test_post_known_bot() {
        #[post("/")]
        fn index_post(_browser: Browser) -> &'static str {
            "Post Home"
        }

        let rocket = rocket::build()
            .manage(AppState {
                bots: Bots::default(),
            })
            .mount("/", routes![index_post]);

        let client = Client::tracked(rocket).expect("valid rocket instance");

        let mut response = client
            .post("/")
            .header(Header::new(USER_AGENT, CHROME_LIGHTHOUSE_BOT_USER_AGENT))
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);

        response = client
            .post("/")
            .header(Header::new(USER_AGENT, VALID_BROWSER_USER_AGENT))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Post Home");
    }
}
