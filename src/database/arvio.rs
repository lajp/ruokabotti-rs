use super::Database;
use sqlx::types::BigDecimal;

pub struct Statistiikka {
    pub keskiarvo: Option<BigDecimal>,
    pub maara: i64,
    pub positio: u64,
    pub id: i32,
}
pub struct KeskiarvoJaId {
    pub keskiarvo: Option<BigDecimal>,
    pub id: i32,
}
pub struct ArvioituRuoka {
    pub id: i32,
    pub arvio: Option<BigDecimal>,
}
pub struct KayttajaStatistiikka {
    pub keskiarvo: Option<BigDecimal>,
    pub maara: i64,
    pub paras: ArvioituRuoka,
    pub huonoin: ArvioituRuoka,
}

impl Database {
    pub async fn lisaa_arvio(&self, userid: u64, reaktio: i32, ruoka: String) {
        let ruokaid = self.nouda_ruoka_by_name(ruoka).await.unwrap().RuokaID;
        if self.hae_reaktio(userid, reaktio, ruokaid).await.is_ok() {
            return;
        };
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query!(
            "INSERT INTO Arvostelut (RuokaID, KayttajaID, Arvosana) VALUES( ? , ? , ?)",
            ruokaid,
            userid,
            reaktio
        )
        .execute(&mut conn)
        .await
        .unwrap();
    }
    pub async fn poista_arvio(&self, userid: u64, reaktio: i32, ruoka: String) {
        let ruokaid = self.nouda_ruoka_by_name(ruoka).await.unwrap().RuokaID;
        match self.hae_reaktio(userid, reaktio, ruokaid).await {
            Ok(_) => (),
            _ => return,
        };
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query!(
            "DELETE FROM Arvostelut WHERE KayttajaID = ? AND Arvosana = ? AND RuokaID = ?",
            userid,
            reaktio,
            ruokaid
        )
        .execute(&mut conn)
        .await
        .unwrap();
    }
    pub async fn hae_reaktio(&self, userid: u64, reaktio: i32, ruokaid: i32) -> Result<(), ()> {
        let mut conn = self.pool.acquire().await.unwrap();
        match sqlx::query!("SELECT Arvosana FROM Arvostelut WHERE KayttajaID = ? AND Arvosana = ? AND RuokaID = ? ",
            userid, reaktio, ruokaid).fetch_one(&mut conn).await {
            Ok(..) => Ok(()),
            _ => Err(()),
        }
    }
    pub async fn anna_ruoan_statistiikka(
        &self,
        ruoka: String,
    ) -> Result<Statistiikka, sqlx::Error> {
        let ruokaid = self.nouda_ruoka_by_name(ruoka).await.unwrap().RuokaID;
        let mut conn = self.pool.acquire().await.unwrap();
        sqlx::query_as!(Statistiikka, "WITH Ranking AS (SELECT RuokaID as id, AVG(Arvosana) AS keskiarvo, COUNT(DISTINCT(KayttajaID)) AS maara, RANK() OVER (ORDER BY AVG(Arvosana) DESC) AS positio FROM Arvostelut GROUP BY RuokaID) SELECT * FROM Ranking WHERE id = ?;", ruokaid).fetch_one(&mut conn).await
    }
    pub async fn anna_kayttajan_statistiikka(&self, userid: u64) -> Option<KayttajaStatistiikka> {
        let mut conn = self.pool.acquire().await.unwrap();
        let maara = sqlx::query!(
            "SELECT COUNT(DISTINCT(RuokaID)) AS maara FROM Arvostelut WHERE KayttajaID = ?",
            userid
        )
        .fetch_one(&mut conn)
        .await
        .unwrap()
        .maara;
        if maara == 0 {
            return None;
        }
        let keskiarvo = sqlx::query!(
            "SELECT AVG(Arvosana) AS keskiarvo FROM Arvostelut WHERE KayttajaID = ?",
            userid
        )
        .fetch_one(&mut conn)
        .await
        .unwrap()
        .keskiarvo;
        let paras = sqlx::query_as!(ArvioituRuoka, "SELECT RuokaID as id, AVG(Arvosana) as arvio FROM Arvostelut WHERE KayttajaID = ? GROUP BY RuokaID ORDER BY AVG(Arvosana) DESC",userid).fetch_one(&mut conn).await.unwrap();
        let huonoin = sqlx::query_as!(ArvioituRuoka, "SELECT RuokaID as id, AVG(Arvosana) as arvio FROM Arvostelut WHERE KayttajaID = ? GROUP BY RuokaID ORDER BY AVG(Arvosana) ASC",userid).fetch_one(&mut conn).await.unwrap();
        Some(KayttajaStatistiikka {
            keskiarvo,
            maara,
            paras,
            huonoin,
        })
    }
    pub async fn anna_parhaat_ruoat(&self, limit: Option<i32>) -> Vec<KeskiarvoJaId> {
        let mut conn = self.pool.acquire().await.unwrap();
        let ruoat = sqlx::query_as!(KeskiarvoJaId, "SELECT AVG(Arvosana) as keskiarvo, RuokaID as id FROM Arvostelut GROUP BY RuokaID ORDER BY AVG(Arvosana) DESC LIMIT ?;", limit.unwrap_or(5))
            .fetch_all(&mut conn).await.unwrap();
        ruoat
    }
    pub async fn anna_huonoimmat_ruoat(&self, limit: Option<i32>) -> Vec<KeskiarvoJaId> {
        let mut conn = self.pool.acquire().await.unwrap();
        let ruoat = sqlx::query_as!(KeskiarvoJaId, "SELECT AVG(Arvosana) as keskiarvo, RuokaID as id FROM Arvostelut GROUP BY RuokaID ORDER BY AVG(Arvosana) ASC LIMIT ?;", limit.unwrap_or(5))
            .fetch_all(&mut conn).await.unwrap();
        ruoat
    }
}
