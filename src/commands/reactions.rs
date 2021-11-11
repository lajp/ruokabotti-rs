use crate::database::*;
use serenity::builder::CreateEmbed;
use serenity::model::channel::Reaction;
use serenity::prelude::*;
use tracing::info;

pub async fn food_reaction(ctx: &Context, reaction: &Reaction, add: bool) {
    if (1..6).contains(&reaction.emoji.to_string()[..1].parse::<i32>().unwrap()) {
        let message = &mut ctx
            .http
            .get_message(reaction.channel_id.0, reaction.message_id.0)
            .await
            .unwrap();
        let wholefood = &message.embeds[0].fields[0].value;
        let food = &wholefood[..match wholefood.find(',') {
            Some(n) => n,
            None => wholefood.len(),
        }];
        let db = ctx.data.read().await.get::<Database>().unwrap().clone();
        if add {
            db.add_rating(
                reaction.user_id.unwrap().0,
                reaction.emoji.to_string()[..1].parse::<i32>().unwrap(),
                food.to_string(),
            )
            .await;
            info!(
                "Reaction {} added by user `{}` to food: {}",
                &reaction.emoji,
                reaction.user_id.unwrap().0,
                food
            );
        } else {
            db.remove_rating(
                reaction.user_id.unwrap().0,
                reaction.emoji.to_string()[..1].parse::<i32>().unwrap(),
                food.to_string(),
            )
            .await;
            info!(
                "Reaction {} removed by user `{}` from food: {}",
                &reaction.emoji,
                reaction.user_id.unwrap().0,
                food
            );
        }
        let rating_count;
        let average;
        let ranking;
        if let Ok(stats) = db.fetch_food_stats(food.to_string()).await {
            average = match stats.average.as_ref() {
                Some(s) => s.round(2).to_string(),
                None => "N/A".to_string(),
            };
            rating_count = stats.rating_count.to_string();
            ranking = stats.ranking.to_string();
        } else {
            rating_count = "0".to_string();
            average = "N/A".to_string();
            ranking = "N/A".to_string();
        }
        let mut orig_embed = message.embeds[0].clone();
        let orig_foodstring = &orig_embed.fields[0].value;
        let foodstring = format!(
            "{}{}",
            &orig_foodstring[..orig_foodstring.find('(').unwrap()],
            format!(
                "(**#{}** :star:{}, {} arvostelija(a))",
                ranking, average, rating_count
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
