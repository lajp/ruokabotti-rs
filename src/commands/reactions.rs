use crate::database::*;
use serenity::model::channel::Reaction;
use serenity::builder::CreateEmbed;
use serenity::prelude::*;
use tracing::info;

pub async fn food_reaction(ctx: &Context, reaction: &Reaction, add: bool) {
    if (1..6).contains(&reaction.emoji.to_string()[..1].parse::<i32>().unwrap()) {
        let message = &mut ctx
            .http
            .get_message(reaction.channel_id.0, reaction.message_id.0)
            .await
            .unwrap();
        let kokoruoka = &message.embeds[0].fields[0].value;
        let ruoka = &kokoruoka[..match kokoruoka.find(',') {
            Some(n) => n,
            None => kokoruoka.len(),
        }];
        let db = ctx.data.read().await.get::<Database>().unwrap().clone();
        if add {
            db.lisaa_arvio(
                reaction.user_id.unwrap().0,
                reaction.emoji.to_string()[..1].parse::<i32>().unwrap(),
                ruoka.to_string(),
                )
                .await;
            info!(
                "Reaction {} added by user `{}` to food: {}",
                &reaction.emoji,
                reaction.user_id.unwrap().0,
                ruoka
                );
        } else {
            db.poista_arvio(
                reaction.user_id.unwrap().0,
                reaction.emoji.to_string()[..1].parse::<i32>().unwrap(),
                ruoka.to_string(),
            )
            .await;
            info!(
                "Reaction {} removed by user `{}` from food: {}",
                &reaction.emoji,
                reaction.user_id.unwrap().0,
                ruoka
                );
        }
        let maara;
        let keskiarvo;
        let positio;
        if let Ok(stats) = db.anna_ruoan_statistiikka(ruoka.to_string()).await {
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
        let mut orig_embed = message.embeds[0].clone();
        let orig_foodstring = &orig_embed.fields[0].value;
        let foodstring = format!(
            "{}{}",
            &orig_foodstring[..orig_foodstring.find('(').unwrap()],
            format!(
                "(**#{}** :star:{}, {} arvostelija(a))",
                positio, keskiarvo, maara
                )
            .as_str()
            );
        orig_embed.fields[0].value = foodstring;
        message
            .edit(&ctx.http, |m| m.set_embed(CreateEmbed::from(orig_embed)))
            .await
            .unwrap();
    }
}
