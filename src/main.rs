use std::env;
use axum::{
    extract::{Path, State}, http::StatusCode, routing::{get, post}, Router, response::IntoResponse,
};
use std::sync::OnceLock;
use config::{Config, Rule};
use std::sync::Arc;
pub mod config;
pub mod engine;

pub static CONFIG: OnceLock<Config> = OnceLock::new();

struct AppState {
    engine: engine::Engine,
}

async fn root() -> impl IntoResponse {  
    (StatusCode::OK, "Get the world!".to_string())
}

fn get_rules_for_path<'r>(conf: & 'r Config, path: &String) -> Option<Vec<& 'r Rule>>{
    let trigger1 = "post/".to_string() + path;
    let trigger2 = "put/".to_string() + path;
    let rules = conf.rules.iter().filter(|r| {r.trigger == trigger1 || r.trigger == trigger2}).collect();
    Some(rules)
}

async fn post_root(Path(path): Path<String>,
                   State(state): State<Arc<AppState>>,
                   body: String) -> impl IntoResponse {
    let x = Arc::clone(&state);
    let conf = CONFIG.get().unwrap();
    let engine = &x.engine;
    let rules = get_rules_for_path(conf, &path);

    println!("BODY {}", body);
    let mut found_rule = false;
    if let Some(rs) = rules {
        println!("Found rules:");
        for r in rs {
            // found_rule = true;
            println!("  Rule: {} {}", r.name, r.trigger);
            // TODO: Send to engine.
            let rr = engine.
                trigger(r.name.clone(), body.clone());
            match rr {
                Ok(_) => {
                    println!("Send to engine ok");
                    found_rule = true;
                },
                Err(err) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, format!("Cannot handle request '{}'", err));
                }                
            }
        }
    }

    if !found_rule {
        return (StatusCode::NOT_FOUND, format!("No rules match the path '{}'", path));
    }

    (StatusCode::OK, format!("post trigger {}", path))
}

fn get_path() -> String {
    let args: Vec<String> = env::args().collect();

    let mut path = ".".to_string();
    if args.len() > 1 {
        path =  args[1].clone();
    }
    return path;
}

#[tokio::main]
async fn main() {
    let conf_path = get_path();
    let conf = config::read_config(&conf_path).unwrap();
    CONFIG.set(conf).unwrap();
    println!("Global {:?}", CONFIG.get().unwrap().global);
    println!("Rules {:?}", CONFIG.get().unwrap().rules);

    // Start rule engine
    let engine = engine::start();

    let app_state: AppState = AppState {
        engine: engine,
    };

    let app_state_arc = Arc::new(app_state);
    let app = Router::new()
    .route("/", get(root))
    .route("/{trigger}", post(post_root).put(post_root))
    .with_state(app_state_arc)
    ;

    // run our app with hyper, listening globally on port 3000
    let port = CONFIG.get().unwrap().global.port;
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
