use std::env;
use axum::{
    extract::{Path, State}, http::StatusCode, routing::{get, post}, Router, response::IntoResponse,
};
use std::sync::OnceLock;
use config::{Config, Trigger, Rule};

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

fn get_triggers_for_path<'t>(conf: & 't Config, path: &String) -> Option<Vec<& 't Trigger>> {
    let triggers: &Vec<Trigger> = &conf.triggers;
    let triggers = triggers.iter().filter(|tr| tr.value == *path).collect();
    Some(triggers)

}

fn get_rules_for_triggers<'r>(conf: & 'r Config, triggers: Vec<&Trigger>) -> Option<Vec<& 'r Rule>>{
    let trs = triggers;
    let rs = conf.rules.iter().filter(|r| trs.iter().any(|t| t.name == r.trigger)).collect();
    Some(rs)
}

fn get_rules_for_path<'r>(conf: & 'r Config, path: &String) -> Option<Vec<& 'r Rule>>{
    let triggers = get_triggers_for_path(conf, path)?;
    let rules = get_rules_for_triggers(conf, triggers);
    rules
}

async fn post_root(Path(path): Path<String>,  State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let x = Arc::clone(&state);
    let conf = CONFIG.get().unwrap();
    let engine = &x.engine;
    let rules = get_rules_for_path(conf, &path);

    let mut found_rule = false;
    if let Some(rs) = rules {
        println!("Found rules:");
        for r in rs {
            // found_rule = true;
            println!("  Rule: {} {}", r.name, r.trigger);
            // TODO: Send to engine.
            let rr = engine.trigger(r.name.clone());
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
    .route("/{trigger}", post(post_root))
    .with_state(app_state_arc)
    ;

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
