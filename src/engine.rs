use std::{thread, time::Duration};
use std::sync::mpsc::{self, Receiver};

pub struct Engine {
    pub tx: mpsc::Sender<String>,        
    // rx: mpsc::Receiver<String>,        
}

impl Engine {

    pub fn trigger(&mut self, trigger: String) {
        let r = self.tx.send(trigger);
        match r {
            Ok(_) => println!("Send OK"),
            Err(e) => println!("Send failed {}", e)
       }
    }
}

pub fn start() -> Engine {
    let (tx, rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
    let e = Engine {
        tx: tx,
    };

    println!("Start engine");
    thread::spawn(|| {engine_runner(rx)});
    return e;
}

fn engine_runner(rx: mpsc::Receiver<String>) {
    // let mut count = 1;
    loop {
        let result = rx.recv();
        match result {
            Ok(t) => println!("Received trigger {}", t),
            Err(e) => println!("Receive failed {}", e),
        }
        // thread::sleep(Duration::from_secs(5));
        // println!("Runner {}", count);
        // count += 1;    
    }
}