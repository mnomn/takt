use std::env;
use axum::{
    extract::{Path, State}, http::StatusCode, routing::{get, post}, Router
};
use config::Config;
use engine::Engine;
use std::sync::Arc;
pub mod config;
pub mod engine;

struct AppState {
    config: config::Config,
    engine: engine::Engine,
}

async fn root(State(state): State<Arc<AppState>>) -> &'static str {
    let x = Arc::clone(&state);
    let gg = &x.config.global;
    match gg {
        Some(gp) => println!("golbal some: {:?}", gp),
        None => println!("None")
    }
    "Get the world!"
}

async fn post_root(Path(trigger): Path<String>,
                   State(app_state): State<Arc<AppState>>
                   ) -> (StatusCode, String) {
    println!("PAT  {}", trigger);
    // let rr = format!("Hello, post World! {}", trigger);
 //   let en = Arc::clone(&engine_state);
//    let etx = &en.tx;

    let app_state = Arc::clone(&app_state);
    let conf = &app_state.config;
    let eng = &app_state.engine;
    match &conf.triggers {
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "No triggers configured".to_string()),
        Some(v) => {
            for t in v.iter() {
                println!("COMPARE {} and {}", trigger, t.name);
                if t.name == trigger {
                    eng.tx.send(trigger);
                    return (StatusCode::OK, "Found trigger".to_string());
                }
            }
        } 
    }

    (StatusCode::NOT_FOUND, "Cannot find trigger".to_string())
}



#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();
    // dbg!(args);

    let mut path = ".";
    if args.len() > 1 {
        path =  args[1].as_str();
        // let ast = a1.as_mut_str();
    }
    let conf = config::read_config(path).unwrap();
    println!("Global {:?}", conf.global);

    // Start rule engine
    let engine = engine::start();

    let app_state = AppState {
        config: conf,
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
