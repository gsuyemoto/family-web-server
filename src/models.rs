use crate::schema::{users, devices};
use diesel::prelude::*;

#[derive(Insertable, Queryable, Deserialize, Debug)]
#[table_name="users"]
pub struct NewUser {
    pub name: String,
    pub points: i32,
    pub is_admin: i32,
}

#[derive(Insertable, Queryable, Deserialize, Debug)]
#[table_name="devices"]
pub struct NewDevice {
    pub user_id: i32,
    pub nickname: String,
    pub addr_mac: String,
    pub addr_ip: String,
    pub is_watching: i32,
    pub is_blocked: i32,
    pub is_tracked: i32,
}
