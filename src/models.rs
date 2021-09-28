use super::schema::*;

#[derive(Queryable)]
pub struct Settings {
    pub key: String,
    pub value: String,
}

#[derive(Insertable)]
#[table_name = "settings"]
pub struct NewSetting<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Queryable)]
pub struct Lineup {
    pub id: String,
    pub modified: String,
    pub uri: String,
    pub is_deleted: bool,
}

#[derive(Insertable)]
#[table_name = "lineups"]
pub struct NewLineup<'a> {
    pub id: &'a str,
    pub modified: &'a str,
    pub uri: &'a str,
    pub is_deleted: bool,
}
