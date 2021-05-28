use crate::errors::AppError;
use crate::schema::{users, devices};
use diesel::prelude::*;

type Result<T> = std::result::Result<T, AppError>;

#[derive(Queryable, Identifiable, Serialize, Debug, PartialEq)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub points: i32,
    pub is_admin: i32,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub username: String,
    pub points: i32,
    pub is_admin: i32,
}

//#[derive(Queryable, Identifiable, Serialize, Debug, PartialEq)]
//pub struct Devices {
//    pub id: i32, 
//    pub username: String,
//    pub mac: String,
//    pub name: String,
//}

pub fn create_user(conn: &SqliteConnection, username: String, is_admin: i32) -> Result<User> {
    let new_user = NewUser
    {
        username,
        points: 0,
        is_admin,
    };

    conn.transaction(|| {
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)?;

        users::table
            .order(users::id.desc())
            .select((users::id, users::username, users::points, users::is_admin))
            .first(conn)
            .map_err(Into::into)
    })
}
