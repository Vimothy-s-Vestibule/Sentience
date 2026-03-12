diesel::table! {
    use diesel::sql_types::*;
    use pgvector::sql_types::Vector;

    messages (message_id) {
        message_id -> Text,
        username -> Text,
        user_id -> Text,
        content -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use pgvector::sql_types::Vector;

    vestibule_users (discord_user_id) {
        discord_user_id -> Text,
        discord_username -> Text,
        yt_username -> Nullable<Text>,
        yt_display_name -> Nullable<Text>,
        intro_message_id -> Nullable<Text>,
        honesty_humility -> Float8,
        emotionality -> Float8,
        extraversion -> Float8,
        agreeableness -> Float8,
        conscientiousness -> Float8,
        openness_to_experience -> Float8,
        agency -> Float8,
        communion -> Float8,
        self_direction -> Float8,
        stimulation -> Float8,
        hedonism -> Float8,
        achievement -> Float8,
        power -> Float8,
        security -> Float8,
        conformity -> Float8,
        tradition -> Float8,
        benevolence -> Float8,
        universalism -> Float8,
        intro_embedding -> Nullable<Vector>,
        intro_diagram -> Nullable<Bytea>,
        status -> Text,
        activities -> Array<Text>,
        domains -> Array<Text>,
    }
}

diesel::joinable!(vestibule_users -> messages (intro_message_id));

diesel::allow_tables_to_appear_in_same_query!(messages, vestibule_users,);
