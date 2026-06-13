
use diesel::table;
use diesel::joinable;
use diesel::allow_tables_to_appear_in_same_query;
table! {
    employees (id) {
        id -> Int8,
        name -> Text,
        email -> Text,
        phone -> Text,
    }
}