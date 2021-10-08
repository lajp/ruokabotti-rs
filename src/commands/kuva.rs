use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::error;
use crate::database::*;
use std::boxed::Box;

#[command]
pub async fn kuva(ctx: &Context, msg: &Message, mut args:Args) -> CommandResult {
    match args.single::<String>() {
        Ok(_) => {
            let query: String = args.raw().collect::<Vec<&str>>().join(" ");
            let db = ctx.data.read().await.get::<Database>().unwrap().clone();
            match db.ruokakuvat_by_query(query.clone()).await {
                Some(names) => {
                    msg.channel_id.say(&ctx.http, format!("http://ruoka.lajp.fi/{}", names.join("\nhttp://ruoka.lajp.fi/"))).await?;
                    Ok(())
                },
                None => {
                    msg.channel_id.say(&ctx.http, format!("Ei kuvia hakusanalle {}!", query)).await?;
                    Ok(())
                }
            }
        },
        Err(e) => {
            error!("!kuva was executed without a search term");
            msg.channel_id.say(&ctx.http, "Syötä hakusana! `Käyttöohje: \"!kuva <hakusana>\"`").await?;
            Err(Box::new(e))
        }
    }
}
