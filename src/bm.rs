// extern crate ini;
extern crate mosquitto_client as mosq;
use super::whois::Whois;
use super::wx::Weather;
use ini::Ini;
use log::{debug, error, info, warn};
use mosq::Mosquitto;

// const MQTT_SERVER_HOST: &str = "localhost";
// const MQTT_SERVER_PORT: u32 = 1883;
// const MQTT_TOPIC_INCOMING: &str = "Master/4601/Incoming/Message/#";
// const MQTT_TOPIC_OUTGOING: &str = "Master/4601/Outgoing/Message/460990/";

fn raw_byte_8to16(s8: &[u8]) -> &[u16] {
    unsafe { std::slice::from_raw_parts(s8.as_ptr() as *const u16, s8.len() / 2) }
}

enum TextEncoding {
    UTF8,
    UTF16LE,
}

pub struct MQTT {
    m: Mosquitto,
    mp: Mosquitto,
    mqtt_host: String,
    mqtt_port: u32,
    bmid: u32,
    serviceid: u32,
    text_encoding: TextEncoding,
    weater: Weather,
    whois: Whois,
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

        let text_encoding = match sec.get("text_encoding").unwrap() {
            "utf8" => TextEncoding::UTF8,
            _ => TextEncoding::UTF16LE,
        };

        let weater = Weather::new();
        let whois = Whois::init();

        MQTT {
            m,
            mp,
            mqtt_host,
            mqtt_port,
            bmid,
            serviceid,
            text_encoding,
            weater,
            whois,
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
                let text = match self.text_encoding {
                    TextEncoding::UTF16LE => {
                        String::from_utf16_lossy(raw_byte_8to16(msg.payload())).to_uppercase()
                    }
                    _ => msg.text().to_uppercase(),
                };
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
                        "HELP" => self.send_service_help(id),
                        "WX" => self.send_wx(id, cmd),
                        "WHOIS" => self.send_whois(id, cmd),
                        _ => self.send_service_help(id),
                    }
                }
            }
        });

        self.m.loop_until_disconnect(200).expect("broken loop");
    }

    fn send_service_help(&self, id: &str) {
        info!("Service help requested by #{}", id);
        let text = "BM4601 Service Help\nAvailable commands:\nwx <location>\nwhois <id/callsign>\nhelp\n";

        self.send_text(id, text.to_string());
    }

    fn send_wx_help(&self, id: &str) {
        debug!("{} -> help wx", id);
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

    fn send_whois_help(&self, id: &str) {
        debug!("{} -> help whois", id);
        let text = format!("Hi {},\nuse the format:\nwhois <id/callsign>", id);

        self.send_text(id, text);
    }

    fn send_whois(&self, id: &str, cmd: Vec<&str>) {
        if cmd.len() == 1 {
            self.send_whois_help(id);
            warn!("id/callsign paramater is missing.");
            return;
        }

        debug!("{} -> {}", id, cmd[1]);
        self.send_text(id, self.whois.query_text(cmd[1]));
    }

    fn send_text(&self, id: &str, text: String) {
        info!("Send Msg to ID->{}", id);
        info!("Msg->>>\n{}", text);

        let mut append: Vec<u8> = vec![];
        let payload = match self.text_encoding {
            TextEncoding::UTF16LE => {
                // trick to convert to UTF-16LE
                for c in text.as_bytes() {
                    append.push(*c);
                    append.push(0);
                }
                &append[..]
            }
            _ => text.as_bytes(),
        };
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
