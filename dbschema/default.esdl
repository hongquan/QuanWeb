module default {
    scalar type DocFormat extending enum<Md, Rst>;

    type User {
        required username: str {
            constraint exclusive;
        }
        required password: str {
            constraint max_len_value(100);
        }
        first_name: str {
            constraint max_len_value(40);
        }
        last_name: str {
            constraint max_len_value(40);
        }
        required email: str {
            constraint exclusive;
            constraint max_len_value(200);
        }
        is_active: bool {
            default := true;
        }
        is_superuser: bool {
            default := false;
        }
        old_id: int16 {
            readonly := true;
            constraint exclusive;
        }
        index on (str_lower(.username));
        index on (str_lower(.email));
    }

    type BlogCategory {
        required title: str {
            constraint max_len_value(50);
        }
        title_vi: str {
            constraint max_len_value(50);
        }
        required slug: str {
            constraint exclusive;
            constraint max_len_value(50);
        }
        old_id: int16 {
            readonly := true;
            constraint exclusive;
        }
        index on (str_lower(.slug));
    }

    type BlogPost {
        required title: str {
            constraint max_len_value(200);
        }
        required slug: str {
            constraint exclusive;
            constraint max_len_value(200);
        }
        body: str;
        format: DocFormat {
            default := DocFormat.Md;
        }
        locale: str {
            constraint max_len_value(6);
        }
        excerpt: str;
        html: str;
        is_published: bool {
            default := false;
        }
        published_at: datetime {
            rewrite update using (datetime_of_statement() if __specified__.is_published and .is_published else __old__.published_at);
        }
        link author: User {
            on target delete allow;
        }
        multi link categories: BlogCategory {
            on target delete allow;
        }
        seo_description: str {
            constraint max_len_value(400);
        }
        multi seo_keywords: str {
            constraint max_len_value(40);
        }
        og_image: str {
            constraint max_len_value(200);
        }
        created_at: datetime {
            default := datetime_current();
        }
        updated_at: datetime {
            default := datetime_current();
            rewrite update using (
                datetime_of_statement()
                if not __specified__.updated_at
                else .updated_at
            )
        }
        old_id: int16 {
            readonly := true;
            constraint exclusive;
        }
        index on (str_lower(.slug));
        index on (str_lower(.title));
    }

    type BookAuthor {
        required name: str {
            constraint exclusive;
        }
        old_id: int16 {
            readonly := true;
            constraint exclusive;
        }
    }

    type Book {
        required title: str {
            constraint max_len_value(200);
        }
        download_url: str;
        link author: BookAuthor {
            on target delete allow;
        }
        created_at: datetime {
            default := datetime_current();
        }
        updated_at: datetime {
            default := datetime_current();
        }
        link created_by: User;
        old_id: int16 {
            readonly := true;
            constraint exclusive;
        }
    }

    type Presentation {
        required title: str {
            constraint max_len_value(400);
        }
        required url: str {
            constraint max_len_value(400);
        }
        event: str {
            constraint max_len_value(200);
        }
        old_id: int16 {
            readonly := true;
            constraint exclusive;
        }
    }
}
