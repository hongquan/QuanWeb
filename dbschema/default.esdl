module default {
    scalar type DocFormat extending enum<Md, Rst>;

    type User {
        required property username -> str {
            constraint exclusive;
        }
        required property password -> str {
            constraint max_len_value(100);
        }
        property first_name -> str {
            constraint max_len_value(40);
        }
        property last_name -> str {
            constraint max_len_value(40);
        }
        required property email -> str {
            constraint exclusive;
            constraint max_len_value(200);
        }
        property is_active -> bool {
            default := true;
        }
        property is_superuser -> bool {
            default := false;
        }
        property old_id -> int16 {
            readonly := true;
            constraint exclusive;
        }
        index on (str_lower(.username));
        index on (str_lower(.email));
    }

    type BlogCategory {
        required property title -> str {
            constraint max_len_value(50);
        }
        required property slug -> str {
            constraint exclusive;
            constraint max_len_value(50);
        }
        property old_id -> int16 {
            readonly := true;
            constraint exclusive;
        }
        index on (str_lower(.slug));
    }

    type BlogPost {
        required property title -> str {
            constraint max_len_value(200);
        }
        required property slug -> str {
            constraint exclusive;
            constraint max_len_value(200);
        }
        property body -> str;
        property format -> DocFormat {
            default := DocFormat.Md;
        }
        property locale -> str {
            constraint max_len_value(6);
        }
        property excerpt -> str;
        property html -> str;
        property is_published -> bool {
            default := false;
        }
        property published_at -> datetime {
            default := datetime_current();
        };
        link author -> User {
            on target delete allow;
        }
        multi link categories -> BlogCategory;
        property seo_description -> str {
            constraint max_len_value(400);
        }
        multi property seo_keywords -> str {
            constraint max_len_value(40);
        }
        property og_image -> str {
            constraint max_len_value(200);
        }
        property created_at -> datetime {
            default := datetime_current();
        }
        property updated_at -> datetime {
            default := datetime_current();
        }
        property old_id -> int16 {
            readonly := true;
            constraint exclusive;
        }
        index on (str_lower(.slug));
    }

    type BookAuthor {
        required property name -> str {
            constraint exclusive;
        }
        property old_id -> int16 {
            readonly := true;
            constraint exclusive;
        }
    }

    type Book {
        required property title -> str {
            constraint max_len_value(200);
        }
        property download_url -> str;
        link author -> BookAuthor {
            on target delete allow;
        }
        property created_at -> datetime {
            default := datetime_current();
        }
        property updated_at -> datetime {
            default := datetime_current();
        }
        link created_by -> User;
        property old_id -> int16 {
            readonly := true;
            constraint exclusive;
        }
    }

    type Presentation {
        required property title -> str {
            constraint max_len_value(400);
        }
        required property url -> str {
            constraint max_len_value(400);
        }
        property event -> str {
            constraint max_len_value(200);
        }
        property old_id -> int16 {
            readonly := true;
            constraint exclusive;
        }
    }
}
