use crate::database::*;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::convert::TryInto;

#[command]
#[aliases(ruokastats)]
pub async fn foodstats(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let userid;
    if !msg.mentions.is_empty() {
        userid = msg.mentions[0].id.0;
    } else {
        userid = msg.author.id.0;
    }
    let user = &mut ctx.http.get_user(userid).await.unwrap();
    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    let stats = match db.fetch_user_stats(userid).await {
        Some(s) => s,
        None => {
            msg.channel_id
                .say(
                    &ctx.http,
                    format!(
                        "Käyttäjällä {} ei ole yhtään arvosteltua ruokaa!",
                        user.name
                    ),
                )
                .await
                .unwrap();
            return Ok(());
        }
    };
    let paras = db
        .fetch_food_by_id(stats.best.id.try_into().unwrap())
        .await
        .unwrap()
        .name;
    let huonoin = db
        .fetch_food_by_id(stats.worst.id.try_into().unwrap())
        .await
        .unwrap()
        .name;
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(serenity::utils::Color::BLUE);
                e.title(format!("Käyttäjän {} ruokastatsit", user.name));
                e.field("Arvostellut ruoat", stats.rating_count, false);
                e.field("Keskimääräinen arvio", stats.average.unwrap(), false);
                e.field(
                    "Lemppariruoka",
                    format!("{}(:star:{})", paras, stats.best.rating.unwrap().round(2)),
                    false,
                );
                e.field(
                    "Inhokkiruoka",
                    format!(
                        "{}(:star:{})",
                        huonoin,
                        stats.worst.rating.unwrap().round(2)
                    ),
                    false,
                )
            })
        })
        .await?;
    Ok(())
}

#[command]
#[aliases(parhaat)]
pub async fn best(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    let topfive = db.fetch_best_foods(Some(5)).await;
    let mut foodnames = Vec::new();
    for food in &topfive {
        foodnames.push(
            db.fetch_food_by_id(food.id.try_into().unwrap())
                .await
                .unwrap()
                .name,
        );
    }
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(serenity::utils::Color::GOLD);
                e.title("Top 5 ruoat");
                for (index, food) in topfive.iter().enumerate() {
                    e.field(
                        format!("#{}", index + 1),
                        format!(
                            "{} (:star:{})",
                            foodnames[index],
                            food.average.as_ref().unwrap()
                        ),
                        false,
                    );
                }
                e
            })
        })
        .await
        .unwrap();
    Ok(())
}

#[command]
#[aliases(huonoimmat, subipäivät)]
pub async fn worst(ctx: &Context, msg: &Message, __args: Args) -> CommandResult {
    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    let botfive = db.fetch_worst_foods(Some(5)).await;
    let mut foodnames = Vec::new();
    for food in &botfive {
        foodnames.push(
            db.fetch_food_by_id(food.id.try_into().unwrap())
                .await
                .unwrap()
                .name,
        );
    }
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(serenity::utils::Color::DARK_RED);
                e.title("Top 5 paskimmat ruoat");
                for (index, food) in botfive.iter().enumerate() {
                    e.field(
                        format!("#{}", index + 1),
                        format!(
                            "{} (:star:{})",
                            foodnames[index],
                            food.average.as_ref().unwrap()
                        ),
                        false,
                    );
                }
                e
            })
        })
        .await
        .unwrap();
    Ok(())
}
