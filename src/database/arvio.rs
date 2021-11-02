use super::Database;
use sqlx::types::BigDecimal;

pub struct Statistiikka {
    pub keskiarvo: Option<BigDecimal>,
    pub maara: i64,
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
        match self.hae_reaktio(userid, reaktio, ruokaid).await {
            Ok(_) => return,
            _ => (),
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
    pub async fn anna_ruoan_statistiikka(&self, ruoka: String) -> Statistiikka {
        let ruokaid = self.nouda_ruoka_by_name(ruoka).await.unwrap().RuokaID;
        let mut conn = self.pool.acquire().await.unwrap();
        let stats = sqlx::query_as!(Statistiikka, "SELECT AVG(Arvosana) AS keskiarvo, COUNT(DISTINCT(KayttajaID)) AS maara FROM Arvostelut WHERE RuokaID = ?" ,ruokaid).fetch_one(&mut conn).await.unwrap();
        stats
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
        match maara {
            0 => return None,
            _ => (),
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
}
