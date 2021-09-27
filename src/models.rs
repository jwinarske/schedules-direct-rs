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
