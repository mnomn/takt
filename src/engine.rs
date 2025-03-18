use std::thread;
use std::sync::mpsc;
use crate::config::{Config, Rule, Action};
use crate::CONFIG;

pub struct Engine {
    tx: mpsc::Sender<String>,        
}

impl Engine {
    pub fn trigger(&self, trigger: String) -> Result<(), mpsc::SendError<String>>{
        self.tx.send(trigger)
    }
}

pub fn start() -> Engine {
    let (tx, rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
    let e = Engine {
        tx: tx,
    };

    println!("Start engine");
    thread::spawn(move || {engine_runner(rx)});
    return e;
}

fn take_action(cfg: &Config, rule_name: String, ac: Vec<String>) -> Option<String>
{
    println!("Take action {:?}", ac);
    let actions: Vec<&Action> = cfg.actions
        .iter()
        .filter(|a| ac.contains(&a.name))
        .collect();

    println!("Actions len {:}", actions.len());
    for action in actions {
        // let mm = action.method.as_ref();
        println!("REST {:?} Action {:}", 
        rule_name, 
        action.target);
        let client = reqwest::blocking::Client::new();
        let result = client
        .post(action.target.clone())
        .send();
        // let result = reqwest::blocking::post(action.target.clone());
        match result {
            Ok(res) => {
                println!("Action OK: {:},{:?}", res.status(), res.text() ); 
            },
            Err(e) => println!("Action ERR: {:?}", e),
        }
    }
    
    Some(rule_name)
}

fn engine_runner( rx: mpsc::Receiver<String>) {
    let cfg = CONFIG.get().unwrap();

    loop {
        let result = rx.recv();
        match result {
            Ok(rule_name) => {
                println!("Engine {}", rule_name);
                let ru: Vec<&Rule>= cfg.rules.iter().filter(|r| r.name == rule_name).collect();
                for rr in ru {
                    let acts = rr.actions.clone();
                    take_action(cfg, rr.name.clone(), acts);
                }
            },
            Err(e) => println!("Receive failed {}", e),
        }
    }
}