CREATE MIGRATION m16xh577hkoqtuq6q77nq5xfexd3zww4z5wng7p6sbo2jyfmhcmesq
    ONTO m1mpwidjkrhccm3wqxsaadqy27otoodkvsisf25lfoz6yyowg5l6da
{
  ALTER TYPE default::BlogCategory {
      CREATE PROPERTY featured_order: std::int16 {
          SET default := 0;
          CREATE CONSTRAINT std::min_value(0);
      };
      CREATE PROPERTY header_color: std::str {
          SET default := '';
          CREATE CONSTRAINT std::max_len_value(20);
          CREATE CONSTRAINT std::regexp('^(#([A-Fa-f0-9]{3}|[A-Fa-f0-9]{4}|[A-Fa-f0-9]{6}|[A-Fa-f0-9]{8})|)?$');
      };
      CREATE PROPERTY summary_en: std::str {
          SET default := '';
          CREATE CONSTRAINT std::max_len_value(500);
      };
      CREATE PROPERTY summary_vi: std::str {
          SET default := '';
          CREATE CONSTRAINT std::max_len_value(500);
      };
  };
  ALTER TYPE default::BlogPost {
      CREATE PROPERTY highlighted_order: std::int16 {
          SET default := 0;
          CREATE CONSTRAINT std::min_value(0);
      };
  };
};
