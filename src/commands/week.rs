use crate::rating::FoodStatistics;
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
#[aliases(viikko, tääviikko)]
pub async fn week(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
    let mut week = db
        .fetch_week(monday.to_string(), sunday.to_string())
        .await
        .unwrap();
    if week.is_empty() {
        update_ruokadb(ctx, None).await.ok();
        week = db
            .fetch_week(monday.to_string(), sunday.to_string())
            .await
            .unwrap();
        if week.is_empty() {
            msg.channel_id.say(&ctx.http, format!("Ei ruokia viikolle `{}-{}`! Jos tämä on mielestäsi bugi, ota yhteyttä ruokabotin kehittäjiin!",
                monday.format("%d/%m"), sunday.format("%d/%m/%Y"))).await?;
            return Ok(());
        }
    };
    let mut averages = Vec::new();
    let mut rating_counts = Vec::new();
    let mut rankings = Vec::new();
    for (_, food) in week.iter().enumerate() {
        let stat: FoodStatistics = match db
            .fetch_food_stats(
                food[..match food.find(',') {
                    Some(n) => n,
                    None => food.len(),
                }]
                    .to_string(),
            )
            .await
        {
            Ok(s) => s,
            Err(_) => {
                averages.push("N/A".to_string());
                rating_counts.push("0".to_string());
                rankings.push("N/A".to_string());
                continue;
            }
        };
        let average = match stat.average.as_ref() {
            Some(s) => s.round(2).to_string(),
            None => "N/A".to_string(),
        };
        averages.push(average);
        rating_counts.push(stat.rating_count.to_string());
        rankings.push(stat.ranking.to_string());
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
                for (day, food) in week.iter().enumerate() {
                    e.field(
                        format!(
                            "{}: {}",
                            num_to_day(day).unwrap(),
                            (monday + Duration::days(day.try_into().unwrap())).format("%d/%m/%Y")
                        ),
                        format!(
                            "{} \n(**#{}**: :star:{}, {} arvostelija(a))",
                            food, rankings[day], averages[day], rating_counts[day]
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
#[command]
#[aliases(ensviikko)]
pub async fn nextweek(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let currentdate = chrono::offset::Local::today().naive_local();
    let next_week = currentdate + Duration::days(7);
    week(
        ctx,
        msg,
        Args::new(&next_week.format("%d/%m/%Y").to_string(), &[]),
    )
    .await
    .unwrap();
    Ok(())
}
