use diesel::prelude::*;
use crate::schema::{users};
use diesel::prelude::*;

//#[derive(Queryable, Identifiable, Serialize, Debug, PartialEq)]
//pub struct SingleUser {
//    pub id: i32,
//    pub name: String,
//    pub points: i32,
//    pub is_admin: i32,
//}

#[derive(Insertable, Deserialize, Debug)]
#[table_name="users"]
pub struct NewUser {
    pub name: String,
    pub points: i32,
    pub is_admin: i32,
}

//#[derive(Queryable, Identifiable, Serialize, Debug, PartialEq)]
//pub struct Devices {
//    pub id: i32, 
//    pub name: String,
//    pub mac: String,
//    pub name: String,
//}
