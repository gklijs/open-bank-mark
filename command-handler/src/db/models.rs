use crate::db::schema::balancer;
use crate::db::schema::cacr;
use crate::db::schema::cmtr;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Queryable, Identifiable)]
#[primary_key(balance_id)]
#[table_name = "balancer"]
pub struct Balance {
    pub balance_id: i32,
    pub iban: String,
    pub token: String,
    pub amount: i64,
    pub type_: String,
    pub lmt: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "balancer"]
pub struct NewBalance<'a> {
    pub iban: &'a str,
    pub token: &'a str,
    pub amount: i64,
    pub type_: &'a str,
    pub lmt: i64,
}

#[derive(Clone, Debug, Queryable)]
pub struct Cac {
    pub uuid: Uuid,
    pub iban: Option<String>,
    pub token: Option<String>,
    pub type_: Option<String>,
    pub reason: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "cacr"]
pub struct NewCac<'a> {
    pub uuid: Uuid,
    pub iban: Option<&'a str>,
    pub token: Option<&'a str>,
    pub type_: Option<&'a str>,
    pub reason: Option<&'a str>,
}

#[derive(Debug, Queryable)]
pub struct Cmt {
    pub uuid: Uuid,
    pub reason: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "cmtr"]
pub struct NewCmt<'a> {
    pub uuid: Uuid,
    pub reason: Option<&'a str>,
}
