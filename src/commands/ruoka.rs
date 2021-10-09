use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::database::*;
use tracing::{info, error};
use crate::util::dayconvert::*;
use chrono::*;
use std::convert::TryInto;

#[command]
pub async fn ruoka(ctx: &Context, msg: &Message, mut args:Args) -> CommandResult {
    let mut date = match args.single::<String>() {
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
    let ruoka = match db.nouda_ruoka_ja_id_by_date(date.to_string()).await? {
        Some(r) => r,
        None => {
            match date.weekday().num_days_from_monday() {
                d if d > 4 => {
                    msg.channel_id.say(&ctx.http, format!("Ei ruokaa päivälle `{}`! Koska kyseessä on viikonloppu, kokeillaan vielä seuraavan viikon maanantaita.", date.format("%d/%m/%Y"))).await?;
                    info!("No food was found for {} which is on a weekend. Checking the monday of the following week!", date.to_string());
                    let diff:i64 = d.try_into().unwrap();
                    date = date+Duration::days(7-diff);
                    db.nouda_ruoka_ja_id_by_date(date.to_string()).await.unwrap().unwrap()
                },
                _ => {
                    msg.channel_id.say(&ctx.http, format!("Ei ruokaa päivälle `{}`! Jos tämä on mielestäsi bugi, ota yhteyttä ruokabotin kehittäjiin!",
                        date.format("%d/%m/%Y"))).await?;
                    error!("A non-weekend date with no food was found: `{}`. This might mean that the foodlist needs to be updated!", date.to_string());
                    return Ok(())
                }
            }
        }
    };
    let viikonpaiva = num_to_paiva(date.weekday().num_days_from_monday().try_into().unwrap()).unwrap();
    let kuva:String = match db.ruokakuva_by_id(ruoka.RuokaID).await {
        Some(r) => r,
        _ => {
            "".to_string()
        },
    };
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.color(serenity::utils::Color::GOLD);
            e.field(format!("{}: {}", viikonpaiva, date.format("%d/%m/%Y").to_string()), ruoka.KokoRuoka, false);
            e.image(format!("http://ruoka.lajp.fi/{}", kuva))
        })
    }).await?;
    Ok(())
}
