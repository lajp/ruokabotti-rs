use crate::arvio::Statistiikka;
use crate::commands::update_ruokadb::update_ruokadb;
use crate::database::*;
use crate::util::dayconvert::*;
use chrono::{Datelike, Duration};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::convert::TryInto;
use tracing::error;
use tracing::info;

#[command]
pub async fn viikko(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let date = match args.single::<String>() {
        Ok(a) => {
            match chrono::NaiveDate::parse_from_str(&a, "%d/%m/%Y") {
                Ok(n) => n,
                Err(e) => {
                    msg.channel_id.say(&ctx.http, format!("Virheellinen päivämäärämuoto: `{}`, tulostetaan tämän viikon ruoka.", a)).await?;
                    error!("Date parse-error, invalid date format: {}", e);
                    chrono::offset::Local::today().naive_local()
                }
            }
        }
        Err(_) => chrono::offset::Local::today().naive_local(),
    };
    let weekday = date.weekday();
    let difftomon: i64 = weekday.num_days_from_monday().into();
    let monday = date + Duration::days(-difftomon);
    let sunday = date + Duration::days(6 - difftomon);
    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    let mut viikko = db
        .nouda_viikko(monday.to_string(), sunday.to_string())
        .await
        .unwrap();
    match viikko.len() {
        0 => {
            update_ruokadb(ctx, None).await.ok();
            viikko = db
                .nouda_viikko(monday.to_string(), sunday.to_string())
                .await
                .unwrap();
            if viikko.len() == 0 {
                msg.channel_id.say(&ctx.http, format!("Ei ruokia viikolle `{}-{}`! Jos tämä on mielestäsi bugi, ota yhteyttä ruokabotin kehittäjiin!",
                    monday.format("%d/%m"), sunday.format("%d/%m/%Y"))).await?;
                return Ok(());
            }
        }
        _ => viikko = viikko,
    };
    let mut keskiarvot = Vec::new();
    let mut maarat = Vec::new();
    for (_, ruoka) in viikko.iter().enumerate() {
        let stat: Statistiikka = db
            .anna_ruoan_statistiikka(
                ruoka[..match ruoka.find(",") {
                    Some(n) => n,
                    None => ruoka.len(),
                }]
                    .to_string(),
            )
            .await;
        let keskiarvo = match stat.keskiarvo.as_ref() {
            Some(s) => s.round(2).to_string(),
            None => "N/A".to_string(),
        };
        keskiarvot.push(keskiarvo);
        maarat.push(stat.maara);
    }
    info!(
        "Sending week `{}-{}`",
        monday.to_string(),
        sunday.to_string()
    );
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!(
                    "Viikko {}{}",
                    monday.format("%W: %d/%m"),
                    sunday.format("-%d/%m/%Y")
                ));
                e.color(serenity::utils::Color::PURPLE);
                for (paiva, ruoka) in viikko.iter().enumerate() {
                    e.field(
                        format!(
                            "{}: {}",
                            num_to_paiva(paiva).unwrap(),
                            (monday + Duration::days(paiva.try_into().unwrap())).format("%d/%m/%Y")
                        ),
                        format!(
                            "{} \n(:star:{}, {} arvostelija(a))",
                            ruoka, keskiarvot[paiva], maarat[paiva]
                        ),
                        false,
                    );
                }
                e
            })
        })
        .await?;
    Ok(())
}
