table! {
    board_postnum (board_id) {
        board_id -> Int4,
        postnum -> Int4,
    }
}

table! {
    boards (id) {
        id -> Int4,
        name -> Text,
        description -> Nullable<Text>,
    }
}

table! {
    images (id) {
        id -> Int4,
        filepath -> Text,
        thumbpath -> Text,
    }
}

table! {
    posts (board_id, post_num) {
        board_id -> Int4,
        post_num -> Int4,
        reply_to -> Nullable<Int4>,
        image -> Nullable<Int4>,
        time -> Timestamptz,
        comment -> Text,
    }
}

joinable!(board_postnum -> boards (board_id));
joinable!(posts -> boards (board_id));
joinable!(posts -> images (image));

allow_tables_to_appear_in_same_query!(
    board_postnum,
    boards,
    images,
    posts,
);
