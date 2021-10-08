use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use chrono::{Weekday::*, *};
use crate::database::*;
use tracing::info;
use tracing::error;

#[command]
pub async fn ruoka(ctx: &Context, msg: &Message, mut args:Args) -> CommandResult {
    let date = match args.single::<String>() {
        Ok(a) => {
            match chrono::NaiveDate::parse_from_str(&a, "%d/%m/%Y") {
                Ok(n) => n,
                Err(e) => {
                    msg.channel_id.say(&ctx.http, format!("Virheellinen päivämäärämuoto: `{}`, tulostetaan tämän päivän ruoka.", a)).await?;
                    error!("Date parse-error, invalid date format: {}", e);
                    chrono::offset::Local::today().naive_local()
                }
            }
        },
        Err(_) => chrono::offset::Local::today().naive_local(),
    };
    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    let ruoka = db.nouda_ruoka_by_date(date.to_string()).await?;
    let viikonpaiva = match date.weekday() {
        Mon => "Maanantai",
        Tue => "Tiistai",
        Wed => "Keskiviikko",
        Thu => "Torstai",
        Fri => "Perjantai",
        Sat => "Lauantai",
        Sun => "Sunnuntai"
    };
    info!("{}", ruoka);
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.color(serenity::utils::Color::GOLD);
            e.field(format!("{}: {}", viikonpaiva, date.format("%d/%m/%Y").to_string()), ruoka, false)
        })
    }).await?;
    Ok(())
}
