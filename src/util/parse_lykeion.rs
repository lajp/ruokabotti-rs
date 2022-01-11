use chrono::NaiveDate;
use regex::Regex;
use rss::Channel;

lazy_static::lazy_static! {
    static ref FOOD_REGEX: Regex = Regex::new(r"Lounas\s:\s([\W\w]+)?<br>").unwrap();
    static ref ALLERGEENI_REGEX: Regex = Regex::new(r"\s\(.*?\)").unwrap();
}

pub async fn parse_lykeion() -> Result<Vec<(NaiveDate, String)>, anyhow::Error> {
    let r_client = reqwest::Client::builder().build().unwrap();
    let mut retvec = Vec::new();
    for i in 1..=7 {
        let resp = r_client
            .get(format!("https://aromimenu.cgisaas.fi/VaasaAromieMenus/fi-FI/Default/_/CampusLykeion/Rss.aspx?Id=0c9160c7-bedb-4b60-9ee1-188bf43a02b3&DateMode={}", i))
            .send()
            .await?
            .bytes()
            .await?;
        let channel = Channel::read_from(&resp[..])?;
        for item in channel.items {
            let date_str = item.title.unwrap();
            let date_str = &date_str[date_str.find(' ').unwrap()+1..];
            let date = NaiveDate::parse_from_str(date_str, "%d.%m.%Y").unwrap();
            let foodstr = item.description.unwrap();
            let caps = FOOD_REGEX.captures(&foodstr).unwrap();
            let caps = caps.get(1).unwrap().as_str();
            let caps = ALLERGEENI_REGEX.replace_all(caps, ",");
            let caps = caps.replace("M,G,V", "");
            let caps = caps.replace("., ", "");
            let caps = caps.replace(" ,", ",");
            let caps = caps.replace(",,", ",");
            let caps = caps.replace("  ", " ");
            retvec.push((date, caps.trim_end_matches(&[',', ' ', 'M']).to_string()))
        }
    }
    Ok(retvec)
}
