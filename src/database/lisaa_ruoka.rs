#![allow(dead_code)]
use super::Database;

impl Database {
    pub async fn lisaa_kuva_ruokaan(&self, id: i32, kuva: String) -> Result<sqlx::mysql::MySqlQueryResult, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query!("UPDATE Ruoat SET ImageName = ? WHERE RuokaID = ?", kuva, id)
            .execute(&mut conn).await
    }
}
