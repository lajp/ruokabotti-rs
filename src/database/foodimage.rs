#![allow(dead_code, non_snake_case)]
use super::Database;
use tracing::info;

pub struct Food {
    image_name: Option<String>,
}

impl Database {
    pub async fn fetch_image_by_name(&self, name: String) -> Option<String> {
        let mut conn = self.pool.acquire().await.unwrap();
        let mut foodimage = match sqlx::query!(
            "SELECT ImageName as image_name FROM Ruoat WHERE RuokaName = ?",
            name
        )
        .fetch_one(&mut conn)
        .await
        {
            Ok(r) => r.image_name,
            Err(e) => {
                info!("Error while querying for food image `{}`: {}", name, e);
                None
            }
        };

        foodimage = match foodimage {
            Some(s) => Some(s),
            None => {
                info!("{} has non image in the database", name);
                None
            }
        };
        foodimage
    }
    pub async fn fetch_image_by_id(&self, id: i32) -> Option<String> {
        let mut conn = self.pool.acquire().await.unwrap();
        let mut foodimage = match sqlx::query!(
            "SELECT ImageName as image_name FROM Ruoat WHERE RuokaID = ?",
            id
        )
        .fetch_one(&mut conn)
        .await
        {
            Ok(r) => r.image_name,
            Err(e) => {
                info!("Error while querying for food image ID:`{}`: {}", id, e);
                None
            }
        };

        foodimage = match foodimage {
            Some(s) => Some(s),
            None => {
                info!("ID:{} has non image in the database", id);
                None
            }
        };
        foodimage
    }
    pub async fn fetch_images_by_query(&self, query: String) -> Option<Vec<String>> {
        let mut conn = self.pool.acquire().await.unwrap();
        let foodimages = match sqlx::query!(
            "SELECT ImageName as image_name FROM Ruoat WHERE LOWER(ImageName) LIKE LOWER( ? )",
            format!("%{}%", query)
        )
        .fetch_all(&mut conn)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                info!("Error while querying for food image `{}`: {}", query, e);
                Vec::new()
            }
        };
        Some(
            foodimages
                .iter()
                .map(|r| r.image_name.as_ref().unwrap().clone())
                .collect(),
        )
    }
}
