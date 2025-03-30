use std::ops::Index;
use std::thread;
use std::sync::mpsc;
use reqwest::blocking;

use crate::config::{Config, Rule, Action};
use crate::CONFIG;

pub struct Engine {
    tx: mpsc::Sender<PostTrigger>,
}

pub struct PostTrigger {
    rule: String,
    body: String
}

impl Engine {
    pub fn trigger(&self, rule: String, body: String) -> Result<(), mpsc::SendError<PostTrigger>>{
        let req = PostTrigger{
            rule: rule,
            body: body
        };

        self.tx.send(req)
    }
}

pub fn start() -> Engine {
    let (tx, rx): (mpsc::Sender<PostTrigger>, mpsc::Receiver<PostTrigger>) = mpsc::channel();
    let e = Engine {
        tx: tx,
    };

    println!("Start engine");
    thread::spawn(move || {engine_runner(rx)});
    return e;
}

fn take_action(cfg: &Config, trig: &PostTrigger, ac: Vec<String>) -> Option<String>
{
    println!("Take action {:?}", ac);
    let actions: Vec<&Action> = cfg.actions
        .iter()
        .filter(|a| ac.contains(&a.name))
        .collect();

    println!("Actions len {:}", actions.len());
    for action in actions {
        // let mm = action.method.as_ref();
        let client = reqwest::blocking::Client::new();

        // let result = client
        // .post(action.target.clone())
        // .send();

        let mut builder: blocking::RequestBuilder;

        let m = action.method.clone().unwrap_or ("POST".to_string());

        let m = m.to_lowercase();
        if m == "post" {
            println!("POST");
            builder = client.post(action.target.clone());
        } else if m == "put" {
            println!("PUT");
            builder = client.put(action.target.clone());
        } else {
            println!("Invalid method {:}", m);
            return None;
        }

        if let Some(headers) = action.headers.clone() {
            for h in headers {
                let mut parts = h.split(":");
                let Some(key) = parts.next() else {
                    continue;
                };
                let Some(val) = parts.next() else {
                    continue;
                };
                builder = builder.header( key, val);
            }
        }

        let res_json = serde_json::from_str::<serde_json::Value>(&trig.body.as_str());
        if let Ok(j) = res_json {
            builder = builder.json(&j);
        }

        let result = builder.send();
        // let result = reqwest::blocking::post(action.target.clone());
        match result {
            Ok(res) => {
                println!("Action OK: {:},{:?}", res.status(), res.text() ); 
            },
            Err(e) => println!("Action ERR: {:?}", e),
        }
    }
    
    Some("OK".to_string())
}

fn engine_runner( rx: mpsc::Receiver<PostTrigger>) {
    let cfg = CONFIG.get().unwrap();

    loop {
        let result = rx.recv();
        match result {
            Ok(trig) => {
                println!("Engine {}", trig.rule);
                let ru: Vec<&Rule>= cfg.rules.iter().filter(|r| r.name == trig.rule).collect();
                for rr in ru {
                    let acts = rr.actions.clone();
                    take_action(cfg, &trig, acts);
                }
            },
            Err(e) => println!("Receive failed {}", e),
        }
    }
}