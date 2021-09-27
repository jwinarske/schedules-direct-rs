table! {
    credits (programID) {
        personID -> Nullable<Integer>,
        programID -> Text,
        role -> Nullable<Text>,
    }
}

table! {
    image_cache (row) {
        row -> Integer,
        item -> Text,
        md5 -> Text,
        height -> Text,
        width -> Text,
        #[sql_name = "type"]
        type_ -> Text,
    }
}

table! {
    lineups (row) {
        row -> Integer,
        lineup -> Text,
        modified -> Nullable<Text>,
        json -> Nullable<Text>,
    }
}

table! {
    messages (row) {
        row -> Integer,
        id -> Text,
        date -> Nullable<Text>,
        message -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Nullable<Text>,
        modified -> Timestamp,
    }
}

table! {
    people (personID) {
        personID -> Nullable<Integer>,
        name -> Nullable<Text>,
    }
}

table! {
    program_genres (programID) {
        programID -> Text,
        relevance -> Text,
        genre -> Text,
    }
}

table! {
    program_ratings (programID) {
        programID -> Text,
        system -> Text,
        rating -> Nullable<Text>,
    }
}

table! {
    programs (row) {
        row -> Integer,
        programID -> Text,
        md5 -> Text,
        modified -> Timestamp,
        json -> Text,
    }
}

table! {
    schedules (md5) {
        stationID -> Text,
        md5 -> Text,
    }
}

table! {
    settings (key) {
        key -> Text,
        value -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    credits,
    image_cache,
    lineups,
    messages,
    people,
    program_genres,
    program_ratings,
    programs,
    schedules,
    settings,
);
