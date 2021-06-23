use crate::schema::{users, devices};
use diesel::prelude::*;

#[derive(Insertable, Queryable, Serialize, Deserialize, Debug)]
#[table_name="users"]
pub struct NewUser {
    pub name: String,
    pub points: i32,
    pub is_admin: i32,
}

#[derive(Insertable, Queryable, Serialize, Deserialize, Debug)]
#[table_name="users"]
pub struct User {
    pub user_id: i32,
    pub name: String,
    pub points: i32,
    pub is_admin: i32,
}

#[derive(Insertable, Queryable, Serialize, Deserialize, Debug, Clone)]
#[table_name="devices"]
pub struct NewDevice {
    pub user_id: i32,
    pub nickname: String,
    pub addr_mac: String,
    pub addr_ip: String,
    pub is_watching: i32,
    pub is_blocked: i32,
    pub is_tracked: i32,
    pub last_checked: i32,
    pub last_last_checked: i32,
    pub manufacturer_name: Option<String>,
}

#[derive(Insertable, Queryable, Serialize, Deserialize, Debug, Clone)]
#[table_name="devices"]
pub struct Device {
    pub id: i32,
    pub user_id: i32,
    pub nickname: String,
    pub addr_mac: String,
    pub addr_ip: String,
    pub is_watching: i32,
    pub is_blocked: i32,
    pub is_tracked: i32,
    pub last_checked: i32,
    pub last_last_checked: i32,
    pub manufacturer_name: Option<String>,
}
