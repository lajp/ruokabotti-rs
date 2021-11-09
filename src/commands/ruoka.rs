use crate::commands::update_ruokadb::update_ruokadb;
use crate::database::*;
use crate::util::dayconvert::*;
use chrono::*;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::ReactionType;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::convert::TryInto;
use tracing::{error, info};

#[command]
pub async fn ruoka(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
        }
        Err(_) => {
            let mut duration = Duration::days(0);
            if chrono::offset::Local::now().naive_local().hour() >= 14 {
                msg.channel_id.say(&ctx.http, "Tämän päivän ruokailujen ollessa jo ruokailtu, tulostetaan huomisen ruoka.").await?;
                duration = Duration::days(1);
            }
            chrono::offset::Local::today().naive_local()+duration
        }
    };

    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    let ruoka = match db.nouda_ruoka_ja_id_by_date(date.to_string()).await? {
        Some(r) => r,
        None => match date.weekday().num_days_from_monday() {
            d if d > 4 => {
                msg.channel_id.say(&ctx.http, format!("Ei ruokaa päivälle `{}`! Koska kyseessä on viikonloppu, kokeillaan vielä seuraavan viikon maanantaita.", date.format("%d/%m/%Y"))).await?;
                info!("No food was found for {} which is on a weekend. Checking the monday of the following week!", date.to_string());
                let diff: i64 = d.try_into().unwrap();
                date += Duration::days(7 - diff);
                match db.nouda_ruoka_ja_id_by_date(date.to_string()).await? {
                    Some(r) => r,
                    None => {
                        update_ruokadb(ctx, None).await.ok();
                        match db
                            .nouda_ruoka_ja_id_by_date(date.to_string())
                            .await
                            .unwrap()
                        {
                            Some(r) => r,
                            None => {
                                msg.channel_id.say(&ctx.http, format!("Ei ruokaa päivälle `{}`! Jos tämä on mielestäsi bugi, ota yhteyttä ruokabotin kehittäjiin!",
                                        date.format("%d/%m/%Y"))).await?;
                                return Ok(());
                            }
                        }
                    }
                }
            }
            _ => {
                update_ruokadb(ctx, None).await.ok();
                match db
                    .nouda_ruoka_ja_id_by_date(date.to_string())
                    .await
                    .unwrap()
                {
                    Some(r) => r,
                    None => {
                        msg.channel_id.say(&ctx.http, format!("Ei ruokaa päivälle `{}`! Jos tämä on mielestäsi bugi, ota yhteyttä ruokabotin kehittäjiin!",
                        date.format("%d/%m/%Y"))).await?;
                        error!("A non-weekend date with no food was found: `{}`. This might mean that the foodlist needs to be updated!", date.to_string());
                        return Ok(());
                    }
                }
            }
        },
    };
    let viikonpaiva =
        num_to_paiva(date.weekday().num_days_from_monday().try_into().unwrap()).unwrap();
    let kuva: String = match db.ruokakuva_by_id(ruoka.RuokaID).await {
        Some(r) => r,
        _ => "".to_string(),
    };
    let maara;
    let keskiarvo;
    let positio;
    if let Ok(stats) = db
        .anna_ruoan_statistiikka(
            ruoka.KokoRuoka[..match ruoka.KokoRuoka.find(',') {
                Some(n) => n,
                None => ruoka.KokoRuoka.len(),
            }]
                .to_string(),
        )
        .await
    {
        keskiarvo = match stats.keskiarvo.as_ref() {
            Some(s) => s.round(2).to_string(),
            None => "N/A".to_string(),
        };
        maara = stats.maara.to_string();
        positio = stats.positio.to_string();
    } else {
        maara = "0".to_string();
        keskiarvo = "N/A".to_string();
        positio = "N/A".to_string();
    }
    let message = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(serenity::utils::Color::GOLD);
                e.field(
                    format!("{}: {}", viikonpaiva, date.format("%d/%m/%Y")),
                    format!(
                        "{} \n(**#{}** :star:{}, {} arvostelija(a))",
                        ruoka.KokoRuoka, positio, keskiarvo, maara
                    ),
                    false,
                );
                e.image(format!("http://ruoka.lajp.fi/{}", kuva))
            })
        })
        .await?;
    for rating in 1..6 {
        message
            .react(
                &ctx.http,
                ReactionType::Unicode(format!("{}\u{fe0f}\u{20e3}", rating)),
            )
            .await
            .unwrap();
    }
    Ok(())
}
