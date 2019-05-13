pub mod models;
pub mod schema;
pub mod util;

use crate::db::models::Balance;
use crate::db::models::*;
use crate::db::util::*;
use log::warn;
use avro_rs::types::Value;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use uuid::Uuid;
use r2d2;
use diesel::r2d2::ConnectionManager;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn connect() -> Pool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool")
}

fn create_balance<'a>(
    conn: &PgConnection,
    iban: &'a str,
    token: &'a str,
    type_: &'a str,
) -> Balance {
    use crate::db::schema::balancer;

    let new_balance = NewBalance {
        iban,
        token,
        amount: 0,
        type_,
        lmt: -50000,
    };

    diesel::insert_into(balancer::table)
        .values(&new_balance)
        .get_result(conn)
        .expect("Error saving new balance")
}

fn get_balance_by_iban(conn: &PgConnection, ib: &str) -> Option<Balance> {
    use crate::db::schema::balancer::dsl::*;
    balancer
        .filter(iban.eq(ib))
        .first::<Balance>(conn)
        .optional()
        .unwrap()
}

pub fn get_cac(conn: &PgConnection, id: Uuid, tp: &str) -> Cac {
    use crate::db::schema::cacr::dsl::*;
    match cacr.find(id).first::<Cac>(conn).optional() {
        Ok(Some(v)) => v,
        Ok(None) => create_cac(conn, id, tp),
        Err(e) => panic!(
            "Error trying to get cac with uuid: {:?} and error: {}",
            id, e
        ),
    }
}

pub fn get_cmt(
    conn: &PgConnection,
    id: Uuid,
    values: &[(String, Value)],
) -> (Cmt, Option<Balance>, Option<Balance>) {
    use crate::db::schema::cmtr::dsl::*;
    match cmtr.find(id).first::<Cmt>(conn).optional() {
        Ok(Some(v)) => (v, None, None),
        Ok(None) => (create_cmt(conn, id, values)),
        Err(e) => panic!(
            "Error trying to get cac with uuid: {:?} and error: {}",
            id, e
        ),
    }
}

fn create_cac(conn: &PgConnection, uuid: Uuid, tp: &str) -> Cac {
    use crate::db::schema::cacr;
    let iban = new_iban();
    let reason = match get_balance_by_iban(conn, &iban) {
        Some(_v) => Option::from("generated iban already exists, try again"),
        None => None,
    };
    let token = match reason {
        Some(_v) => String::new(),
        None => new_token(),
    };

    if reason == None {
        create_balance(conn, &iban, &token, tp);
    };

    let new_cac = NewCac {
        uuid,
        iban: Option::from(iban.as_ref()),
        token: Option::from(token.as_ref()),
        type_: Option::from(tp),
        reason,
    };

    diesel::insert_into(cacr::table)
        .values(&new_cac)
        .get_result(conn)
        .expect("Error saving new cac")
}

fn create_cmt(
    conn: &PgConnection,
    uuid: Uuid,
    values: &[(String, Value)],
) -> (Cmt, Option<Balance>, Option<Balance>) {
    use crate::db::schema::cmtr;

    let from = match values[3] {
        (ref _from, Value::String(ref v)) => v,
        _ => panic!("Not a string value, while that was expected"),
    };
    let to = match values[4] {
        (ref _to, Value::String(ref v)) => v,
        _ => panic!("Not a string value, while that was expected"),
    };

    let (reason, b_from, b_to) = if invalid_from(from) {
        (Option::from("from is invalid"), None, None)
    } else if from == to {
        (
            Option::from("from and to can't be same for transfer"),
            None,
            None,
        )
    } else {
        transfer(conn, values, from, to)
    };

    let new_cac = NewCmt { uuid, reason };

    let cmt = diesel::insert_into(cmtr::table)
        .values(&new_cac)
        .get_result(conn)
        .expect("Error saving new balance");
    (cmt, b_from, b_to)
}

fn transfer(
    conn: &PgConnection,
    values: &[(String, Value)],
    from: &str,
    to: &str,
) -> (Option<&'static str>, Option<Balance>, Option<Balance>) {
    use crate::db::schema::balancer::dsl::*;
    let am = match values[2] {
        (ref _amount, Value::Long(ref v)) => v,
        _ => panic!("Not a Long value, while that was expected"),
    };
    let (reason, b_from) = if valid_open_iban(from)  {
        match get_balance_by_iban(conn, from) {
            Some(v) => {
                let tn = match values[1] {
                    (ref _token, Value::String(ref v)) => v,
                    _ => panic!("Not a String value, while that was expected"),
                };
                if &v.token != tn {
                    (Option::from("invalid token"), None)
                } else if v.amount - am < v.lmt {
                    (Option::from("insufficient funds"), None)
                } else {
                    let b_from = match diesel::update(&v)
                        .set(amount.eq(amount - am))
                        .get_result::<Balance>(conn)
                        {
                            Ok(v) => Option::from(v),
                            Err(e) => panic!("error updating balance with iban: {}, error: {}", to, e),
                        };
                    (None, b_from)
                }
            }
            None => {
                warn!("Valid open iban {} not found", from);
                (None, None)
            }
        }
    } else {
        (None, None)
    };
    let b_to = match reason{
        None => {
            if valid_open_iban(to){
                match get_balance_by_iban(conn, to) {
                    Some(v) => match diesel::update(&v)
                        .set(amount.eq(amount + am))
                        .get_result::<Balance>(conn)
                        {
                            Ok(v) => Option::from(v),
                            Err(e) => panic!("error updating balance with iban: {}, error: {}", to, e),
                        },
                    None => {
                        warn!("Valid open iban {} not found", from);
                        None
                    }
                }
            } else{
                None
            }
        },
        Some(_) => None
    };
    (reason, b_from, b_to)
}
