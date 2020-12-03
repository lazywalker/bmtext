// extern crate ini;
extern crate mosquitto_client as mosq;
use super::wx::Weather;
use ini::Ini;
use log::{debug, info, warn};
use mosq::Mosquitto;

// const MQTT_SERVER_HOST: &str = "localhost";
// const MQTT_SERVER_PORT: u32 = 1883;
// const MQTT_TOPIC_INCOMING: &str = "Master/4601/Incoming/Message/#";
// const MQTT_TOPIC_OUTGOING: &str = "Master/4601/Outgoing/Message/460990/";

fn raw_byte_8to16(s8: &[u8]) -> &[u16] {
    unsafe { std::slice::from_raw_parts(s8.as_ptr() as *const u16, s8.len() / 2) }
}

pub struct MQTT {
    m: Mosquitto,
    mp: Mosquitto,
    mqtt_host: String,
    mqtt_port: u32,
    bmid: u32,
    serviceid: u32,
    weater: Weather,
}

impl MQTT {
    pub fn new() -> Self {
        let m = Mosquitto::new("bmsg");
        let mp = m.clone();
        let conf = Ini::load_from_file("config.ini").unwrap();
        let sec = conf.section(Some("bm")).unwrap();

        let mqtt_host = sec.get("mqtt_host").unwrap().to_string();
        let mqtt_port = sec.get("mqtt_port").unwrap().parse::<u32>().unwrap();
        let bmid = sec.get("bmid").unwrap().parse::<u32>().unwrap();
        let serviceid = sec.get("serviceid").unwrap().parse::<u32>().unwrap();

        let weater = Weather::new();

        MQTT {
            m,
            mp,
            mqtt_host,
            mqtt_port,
            bmid,
            serviceid,
            weater,
        }
    }

    pub fn serv(&mut self) {
        self.m
            .connect(&*self.mqtt_host, self.mqtt_port, 5)
            .expect("can't connect");

        let incoming = self
            .m
            .subscribe(&*format!("Master/{}/Incoming/Message/#", self.bmid), 1)
            .expect("can't subscribe to topic");

        let mut mc = self.m.callbacks(0);
        mc.on_message(|_, msg| {
            if incoming.matches(&msg) {
                let text = String::from_utf16_lossy(raw_byte_8to16(msg.payload())).to_lowercase();
                info!("topic {} text '{}'", msg.topic(), text);
                // get id from msg.topic()
                let mut src: Vec<&str> = msg.topic().split('/').collect();
                src.pop();
                let id: &str = src.pop().unwrap();

                // get cmd from msg.text()
                let cmd: Vec<&str> = text.split(' ').collect();
                debug!("{:#?}", cmd);
                if cmd.len() > 0 {
                    match cmd[0] {
                        "help" => self.send_service_help(id),
                        "wx" => self.send_wx(id, cmd),
                        _ => self.send_service_help(id),
                    }
                }
            }
        });

        self.m.loop_until_disconnect(200).expect("broken loop");
    }

    fn send_service_help(&self, id: &str) {
        info!("Service help requested by #{}", id);
        let text = "BM4601 Service Help\nAvailable commands:\nwx <location>\nhelp\n";

        self.send_text(id, text.to_string());
    }

    fn send_wx_help(&self, id: &str) {
        debug!("{} -> wxhelp", id);
        let text = format!(
            "Hi {},\nto receive the weather report from a town, use the format:\nwx <town>",
            id
        );

        self.send_text(id, text);
    }

    fn send_wx(&self, id: &str, cmd: Vec<&str>) {
        if cmd.len() == 1 {
            self.send_wx_help(id);
            warn!("Town paramater is missing.");
            return;
        }

        debug!("{} -> {}", id, cmd[1]);
        self.send_text(
            id,
            format!("Hi {},\n{}", id, self.weater.get_wx_report(cmd[1])),
        );
    }

    fn send_text(&self, id: &str, text: String) {
        info!("Send Msg to ID->{}", id);
        info!("Msg->>>\n{}", text);

        // trick to convert to UTF-16LE
        let mut append: Vec<u8> = vec![];
        for c in text.as_bytes() {
            append.push(*c);
            append.push(0);
        }
        let payload = &append[..];

        self.mp
            .publish(
                &*format!(
                    "Master/{}/Outgoing/Message/{}/{}",
                    self.bmid, self.serviceid, id
                ),
                payload,
                0,
                false,
            )
            .unwrap();
    }
}
