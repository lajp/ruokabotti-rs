use chrono::{Duration, NaiveDate};
use lopdf::Document;
use regex::Regex;
use tracing::info;

pub async fn parse_lykeion(link: Option<String>) -> Result<Vec<(NaiveDate, String)>, ()> {
    let r_client = reqwest::Client::builder().build().unwrap();
    let url;
    let body;
    if link == None {
        body = r_client
            .get("https://www.teese.fi/ruokalistat-2/")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        url = Regex::new(r"https://.*lukio.*\.pdf")
            .unwrap()
            .find_iter(&body)
            .into_iter()
            .last()
            .unwrap()
            .as_str()
            .to_string();
        info!("Found the link to the latest pdf: {}", &url);
    } else {
        url = link.unwrap();
        info!("Was provided with the following link: {}", &url);
    }
    let foodlist = r_client
        .get(url)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

    let document = Document::load_mem(&foodlist).unwrap();
    let mut content = document.extract_text(&[1]).unwrap();
    content.retain(|c| c != '\n');
    let mut listavec = Vec::new();
    while content.contains("VIIKKO") {
        let firstindex = content.find("VIIKKO").unwrap();
        let secondindex = match content[firstindex + 1..].find("VIIKKO") {
            Some(i) => i,
            None => content.len() - 1 - firstindex,
        };
        let week = &content[firstindex..secondindex + firstindex + 1];
        let week = &week.split_whitespace().collect::<Vec<&str>>().join(" "); // trim excess whitespace
        let pvm_str = week.split(' ').collect::<Vec<&str>>()[2];
        let sunpvm =
            NaiveDate::parse_from_str(pvm_str.split('-').collect::<Vec<&str>>()[1], "%d.%m.%Y")
                .unwrap();
        let mut pvm = chrono::offset::Local::today().naive_local();
        let mut foodvec = Vec::new();
        for item in week.split(' ').collect::<Vec<&str>>() {
            match item {
                "Maanantai" => {
                    pvm = sunpvm - Duration::days(6);
                    foodvec.clear();
                }
                "Tiistai" => {
                    listavec.push((pvm, foodvec.join(" ")));
                    foodvec.clear();
                    pvm = sunpvm - Duration::days(5);
                }
                "Keskiviikko" => {
                    listavec.push((pvm, foodvec.join(" ")));
                    foodvec.clear();
                    pvm = sunpvm - Duration::days(4);
                }
                "Torstai" => {
                    listavec.push((pvm, foodvec.join(" ")));
                    foodvec.clear();
                    pvm = sunpvm - Duration::days(3);
                }
                "Perjantai" => {
                    listavec.push((pvm, foodvec.join(" ")));
                    foodvec.clear();
                    pvm = sunpvm - Duration::days(2);
                }
                "Lauantai" => {
                    listavec.push((pvm, foodvec.join(" ")));
                    foodvec.clear();
                    pvm = sunpvm - Duration::days(1);
                }
                _ => {
                    foodvec.push(item);
                }
            }
        }
        listavec.push((pvm, foodvec.join(" ")));
        foodvec.clear();
        content = content[secondindex + firstindex..].to_string();
    }
    Ok(listavec)
}
