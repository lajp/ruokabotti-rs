use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::commands::update_ruokadb::*;

pub async fn handle_admin_message(ctx: Context, msg: Message) -> Result<(), ()> {
    if msg.content.starts_with("!update") {
        let link = match msg.content.len() {
            7 => None,
            _ => Some(msg.content[msg.content.find(" ").unwrap()+1..].to_owned())
        };
        match update_ruokadb(&ctx, link).await {
            Ok(_) => {
                msg.channel_id.say(&ctx.http, "Ruokalista päivitetty!").await.unwrap();
                return Ok(())
            },
            Err(_) => {
                msg.channel_id.say(&ctx.http, "Ruokalistan päivittäminen epäonnistui. Ehkä se on jo ajan tasalla?").await.unwrap();
                return Ok(())
            }
        };
    }
    unimplemented!()
}
