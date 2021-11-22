use super::Database;
use sqlx::types::BigDecimal;

pub struct FoodStatistics {
    pub average: Option<BigDecimal>,
    pub rating_count: i64,
    pub ranking: u64,
    pub id: i32,
}
pub struct AverageAndId {
    pub average: Option<BigDecimal>,
    pub id: i32,
}
pub struct FoodRating {
    pub id: i32,
    pub rating: Option<BigDecimal>,
}
pub struct UserStatistics {
    pub average: Option<BigDecimal>,
    pub rating_count: i64,
    pub best: FoodRating,
    pub worst: FoodRating,
}

impl Database {
    pub async fn add_rating(&self, userid: u64, reaction: i32, food: String) {
        let foodid = self.fetch_food_by_name(food).await.unwrap().id;
        if self.fetch_reaction(userid, reaction, foodid).await.is_ok() {
            return;
        };
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query!(
            "INSERT INTO Arvostelut (RuokaID, KayttajaID, Arvosana) VALUES( ? , ? , ?)",
            foodid,
            userid,
            reaction
        )
        .execute(&mut conn)
        .await
        .unwrap();
    }
    pub async fn remove_rating(&self, userid: u64, reaction: i32, food: String) {
        let foodid = self.fetch_food_by_name(food).await.unwrap().id;
        match self.fetch_reaction(userid, reaction, foodid).await {
            Ok(_) => (),
            _ => return,
        };
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query!(
            "DELETE FROM Arvostelut WHERE KayttajaID = ? AND Arvosana = ? AND RuokaID = ?",
            userid,
            reaction,
            foodid
        )
        .execute(&mut conn)
        .await
        .unwrap();
    }
    pub async fn fetch_reaction(&self, userid: u64, reaction: i32, foodid: i32) -> Result<(), ()> {
        let mut conn = self.pool.acquire().await.unwrap();
        match sqlx::query!("SELECT Arvosana FROM Arvostelut WHERE KayttajaID = ? AND Arvosana = ? AND RuokaID = ? ",
            userid, reaction, foodid).fetch_one(&mut conn).await {
            Ok(..) => Ok(()),
            _ => Err(()),
        }
    }
    pub async fn fetch_food_stats(&self, food: String) -> Result<FoodStatistics, sqlx::Error> {
        let foodid = self.fetch_food_by_name(food).await.unwrap().id;
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query_as!(FoodStatistics, "WITH Ranking AS (SELECT RuokaID as id, AVG(Arvosana) AS average, COUNT(DISTINCT(KayttajaID)) AS rating_count, RANK() OVER (ORDER BY AVG(Arvosana) DESC) AS ranking FROM Arvostelut GROUP BY RuokaID) SELECT * FROM Ranking WHERE id = ?;", foodid).fetch_one(&mut conn).await
    }
    pub async fn fetch_user_stats(&self, userid: u64) -> Option<UserStatistics> {
        let mut conn = self.pool.acquire().await.unwrap();
        let rating_count = sqlx::query!(
            "SELECT COUNT(DISTINCT(RuokaID)) AS rating_count FROM Arvostelut WHERE KayttajaID = ?",
            userid
        )
        .fetch_one(&mut conn)
        .await
        .unwrap()
        .rating_count;
        if rating_count == 0 {
            return None;
        }
        let average = sqlx::query!(
            "SELECT AVG(Arvosana) AS average FROM Arvostelut WHERE KayttajaID = ?",
            userid
        )
        .fetch_one(&mut conn)
        .await
        .unwrap()
        .average;
        let best = sqlx::query_as!(FoodRating, "SELECT RuokaID as id, AVG(Arvosana) as rating FROM Arvostelut WHERE KayttajaID = ? GROUP BY RuokaID ORDER BY AVG(Arvosana) DESC",userid).fetch_one(&mut conn).await.unwrap();
        let worst = sqlx::query_as!(FoodRating, "SELECT RuokaID as id, AVG(Arvosana) as rating FROM Arvostelut WHERE KayttajaID = ? GROUP BY RuokaID ORDER BY AVG(Arvosana) ASC",userid).fetch_one(&mut conn).await.unwrap();
        Some(UserStatistics {
            average,
            rating_count,
            best,
            worst,
        })
    }
    pub async fn fetch_best_foods(&self, limit: Option<i32>) -> Vec<AverageAndId> {
        let mut conn = self.pool.acquire().await.unwrap();
        let foods = sqlx::query_as!(AverageAndId, "SELECT AVG(Arvosana) as average, RuokaID as id FROM Arvostelut GROUP BY RuokaID ORDER BY AVG(Arvosana) DESC LIMIT ?;", limit.unwrap_or(5))
            .fetch_all(&mut conn).await.unwrap();
        foods
    }
    pub async fn fetch_worst_foods(&self, limit: Option<i32>) -> Vec<AverageAndId> {
        let mut conn = self.pool.acquire().await.unwrap();
        let foods = sqlx::query_as!(AverageAndId, "SELECT AVG(Arvosana) as average, RuokaID as id FROM Arvostelut GROUP BY RuokaID ORDER BY AVG(Arvosana) ASC LIMIT ?;", limit.unwrap_or(5))
            .fetch_all(&mut conn).await.unwrap();
        foods
    }
}
