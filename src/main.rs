
mod bm;
mod wx;

use log4rs;
use bm::MQTT;

fn main() {
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    let mut mq = MQTT::new();

    mq.serv();
}
