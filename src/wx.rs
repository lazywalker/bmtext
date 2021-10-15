use ini::Ini;
use log::{debug, error};

pub struct Weather {
    data_url: String,
}

impl Weather {
    pub fn new() -> Self {
        let conf = Ini::load_from_file("config.ini").unwrap();
        let sec = conf.section(Some("wx")).unwrap();
        let data_url = format!(
            "https://devapi.qweather.com/s6/weather/now?key={}&lang=en&location=",
            sec.get("token").unwrap()
        );

        Weather { data_url }
    }

    pub fn get_wx_report(&self, city: &str) -> String {
        let url = format!("{}{}", self.data_url, city);
        let resp = reqwest::blocking::get(&*url);
        let text: String;

        match resp {
            Ok(r) => {
                let json = r.json::<serde_json::Value>().unwrap();
                let wx_data = &json["HeWeather6"][0];
                debug!("{:#?}", wx_data);

                let wx_basic = &wx_data["basic"];
                let wx_now = &wx_data["now"];
                let wx_status = &wx_data["status"];

                if wx_status != "ok" {
                    text = wx_status.to_string();
                } else {
                    let location = wx_basic["location"].to_string();
                    let mut wx_city: Vec<String> = vec![location];
                    if wx_basic["location"] != wx_basic["parent_city"] {
                        wx_city.push(wx_basic["parent_city"].to_string());
                    }
                    if wx_basic["parent_city"] != wx_basic["admin_area"] {
                        wx_city.push(wx_basic["admin_area"].to_string());
                    }

                    text = format!(
                        "WX Report - {}\nT:{}C H:{}% P:{}hPa\nWind:{}km/h\n{}",
                        wx_city.join("\n").to_uppercase(),
                        wx_now["tmp"],
                        wx_now["hum"],
                        wx_now["pres"],
                        wx_now["wind_spd"],
                        wx_now["cond_txt"]
                    );
                }
            }
            Err(e) => {
                error!("{:#?}", e);
                text = "ERROR".to_string();
            }
        };

        // trick to remove \" of json value
        return text.replace(r#"""#, "");
    }
}
