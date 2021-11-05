use crate::database::*;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::convert::TryInto;

#[command]
pub async fn ruokastats(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let userid;
    if !msg.mentions.is_empty() {
        userid = msg.mentions[0].id.0;
    } else {
        userid = msg.author.id.0;
    }
    let user = &mut ctx.http.get_user(userid).await.unwrap();
    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    let stats = match db.anna_kayttajan_statistiikka(userid).await {
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
        .nouda_ruoka_by_id(stats.paras.id.try_into().unwrap())
        .await
        .unwrap()
        .RuokaName;
    let huonoin = db
        .nouda_ruoka_by_id(stats.huonoin.id.try_into().unwrap())
        .await
        .unwrap()
        .RuokaName;
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(serenity::utils::Color::BLUE);
                e.title(format!("Käyttäjän {} ruokastatsit", user.name));
                e.field("Arvostellut ruoat", stats.maara.to_string(), false);
                e.field(
                    "Keskimääräinen arvio",
                    stats.keskiarvo.unwrap().to_string(),
                    false,
                );
                e.field(
                    "Lemppariruoka",
                    format!(
                        "{}(:star:{})",
                        paras,
                        stats.paras.arvio.unwrap().round(2).to_string()
                    ),
                    false,
                );
                e.field(
                    "Inhokkiruoka",
                    format!(
                        "{}(:star:{})",
                        huonoin,
                        stats.huonoin.arvio.unwrap().round(2).to_string()
                    ),
                    false,
                )
            })
        })
        .await?;
    Ok(())
}

#[command]
pub async fn parhaat(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    let topfive = db.anna_parhaat_ruoat(Some(5)).await;
    let mut ruokanimet = Vec::new();
    for ruoka in &topfive {
        ruokanimet.push(
            db.nouda_ruoka_by_id(ruoka.id.try_into().unwrap())
                .await
                .unwrap()
                .RuokaName,
        );
    }
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(serenity::utils::Color::GOLD);
                e.title("Top 5 ruoat");
                for (index, ruoka) in topfive.iter().enumerate() {
                    e.field(
                        format!("#{}", index + 1),
                        format!(
                            "{} (:star:{})",
                            ruokanimet[index],
                            ruoka.keskiarvo.as_ref().unwrap().to_string()
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
#[aliases(subipäivät)]
pub async fn huonoimmat(ctx: &Context, msg: &Message, __args: Args) -> CommandResult {
    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    let botfive = db.anna_huonoimmat_ruoat(Some(5)).await;
    let mut ruokanimet = Vec::new();
    for ruoka in &botfive {
        ruokanimet.push(
            db.nouda_ruoka_by_id(ruoka.id.try_into().unwrap())
                .await
                .unwrap()
                .RuokaName,
        );
    }
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(serenity::utils::Color::DARK_RED);
                e.title("Top 5 paskimmat ruoat");
                for (index, ruoka) in botfive.iter().enumerate() {
                    e.field(
                        format!("#{}", index + 1),
                        format!(
                            "{} (:star:{})",
                            ruokanimet[index],
                            ruoka.keskiarvo.as_ref().unwrap().to_string()
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
