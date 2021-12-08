#![allow(dead_code)]
use super::Database;
use chrono::NaiveDate;
use tracing::info;

impl Database {
    pub async fn add_image_to_food(
        &self,
        id: i32,
        image: String,
    ) -> Result<sqlx::mysql::MySqlQueryResult, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query!(
            "UPDATE Ruoat SET ImageName = ? WHERE RuokaID = ?",
            image,
            id
        )
        .execute(&mut conn)
        .await
    }
    pub async fn add_new_food(&self, name: String) -> Result<i32, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let res = sqlx::query!("INSERT INTO Ruoat (RuokaName) VALUES ( ? )", name)
            .execute(&mut conn)
            .await?;
        match res.rows_affected() {
            r if r > 0 => Ok(self.fetch_food_by_name(name).await.unwrap().id),
            _ => Err(sqlx::error::Error::RowNotFound),
        }
    }
    pub async fn add_foods_to_list(&self, list: Vec<(NaiveDate, String)>) -> Result<(), ()> {
        let mut conn = self.pool.acquire().await.unwrap();
        let mut errcount = 0;
        for item in list {
            info!("Item: {:?}", item);
            let foodname = &item.1[..item.1.find(",").unwrap_or(item.1.len())].to_string();
            let foodid = match self.fetch_food_by_name(foodname.to_string()).await {
                Ok(r) => r.id,
                Err(_) => self.add_new_food(foodname.to_string()).await.unwrap(),
            };
            let res = sqlx::query!(
                "INSERT INTO Ruokalista (PVM, RuokaID, KokoRuoka) VALUES( ? , ? , ?)",
                &item.0.to_string(),
                &foodid,
                &item.1
            )
            .execute(&mut conn)
            .await;
            match res {
                Ok(_) => continue,
                Err(_) => {
                    if errcount == 5 {
                        return Err(());
                    }
                    errcount += 1;
                }
            }
        }
        Ok(())
    }
}
