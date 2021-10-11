use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::database::*;
use std::convert::TryInto;
use crate::arvio::KayttajaStatistiikka;

#[command]
pub async fn ruokastats(ctx: &Context, msg: &Message, mut args:Args) -> CommandResult {
    let userid;
    if msg.mentions.len() > 0 {
        userid = msg.mentions[0].id.0;
    }
    else {
        userid = msg.author.id.0;
    }
    let db = ctx.data.read().await.get::<Database>().unwrap().clone();
    let stats = match db.anna_kayttajan_statistiikka(userid).await {
        Some(s) => s,
        None => {
            msg.channel_id.say(&ctx.http, format!("Käyttäjällä {} ei ole yhtään arvosteltua ruokaa!", msg.author.name)).await.unwrap();
            return Ok(())
        }
    };
    let paras = db.nouda_ruoka_by_id(stats.paras.id.try_into().unwrap()).await.unwrap().RuokaName;
    let huonoin = db.nouda_ruoka_by_id(stats.huonoin.id.try_into().unwrap()).await.unwrap().RuokaName;
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.color(serenity::utils::Color::BLUE);
            e.title(format!("Käyttäjän {} ruokastatsit", msg.author.name));
            e.field("Arvostellut ruoat", stats.maara.to_string(), false);
            e.field("Keskimaarainen arvio", stats.keskiarvo.unwrap().to_string(), false);
            e.field("Lemppariruoka", format!("{}(:star:{})", paras, stats.paras.arvio.unwrap().round(2).to_string()), false);
            e.field("Inhokkiruoka", format!("{}(:star:{})", huonoin, stats.huonoin.arvio.unwrap().round(2).to_string()), false)
        })
    }).await?;
    Ok(())
}
