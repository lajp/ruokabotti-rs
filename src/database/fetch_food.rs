#![allow(dead_code, non_snake_case)]
use super::Database;
use tracing::info;

pub struct Food {
    pub id: i32,
    pub name: String,
}
pub struct FoodListEntry {
    whole_food: String,
}

impl Database {
    pub async fn fetch_food_by_name(&self, name: String) -> Result<Food, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let food: Food = sqlx::query_as!(
            Food,
            "SELECT RuokaID as id, RuokaName as name FROM Ruoat WHERE RuokaName = ?",
            name
        )
        .fetch_one(&mut conn)
        .await?;
        Ok(food)
    }
    pub async fn fetch_food_by_name_case_insensitive(
        &self,
        name: String,
    ) -> Result<Food, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let food: Food = sqlx::query_as!(
            Food,
            "SELECT RuokaID as id, RuokaName as name FROM Ruoat WHERE LOWER(RuokaName) = LOWER(?)",
            name
        )
        .fetch_one(&mut conn)
        .await?;
        Ok(food)
    }
    pub async fn fetch_food_by_id(&self, id: u32) -> Result<Food, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let food: Food = sqlx::query_as!(
            Food,
            "SELECT RuokaID as id, RuokaName as name FROM Ruoat WHERE RuokaID = ?",
            id
        )
        .fetch_one(&mut conn)
        .await?;
        Ok(food)
    }
    pub async fn fetch_food_by_date(&self, date: String) -> Result<Option<String>, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        match sqlx::query!(
            "SELECT KokoRuoka as whole_food FROM Ruokalista WHERE PVM = ?",
            date
        )
        .fetch_one(&mut conn)
        .await
        {
            Ok(r) => Ok(Some(r.whole_food)),
            Err(_) => Ok(None),
        }
    }
    pub async fn fetch_food_and_id_by_date(
        &self,
        date: String,
    ) -> Result<Option<Food>, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        match sqlx::query_as!(
            Food,
            "SELECT RuokaID as id, KokoRuoka as name FROM Ruokalista WHERE PVM = ?",
            date
        )
        .fetch_one(&mut conn)
        .await
        {
            Ok(r) => Ok(Some(r)),
            Err(_) => Ok(None),
        }
    }
    pub async fn fetch_week(&self, start: String, end: String) -> Option<Vec<String>> {
        let mut conn = self.pool.acquire().await.unwrap();
        let week = match sqlx::query_as!(
            FoodListEntry,
            "SELECT KokoRuoka as whole_food FROM Ruokalista WHERE PVM BETWEEN ? AND ?",
            start,
            end
        )
        .fetch_all(&mut conn)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                info!(
                    "Error while querying for foods between `{}` and `{}`: {}",
                    start, end, e
                );
                return None;
            }
        };
        Some(week.iter().map(|r| r.whole_food.clone()).collect())
    }
}
