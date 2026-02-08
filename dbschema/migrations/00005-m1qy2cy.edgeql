CREATE MIGRATION m1qy2cy6lfhegkt66p5pxcnivt2gq3lfpmiph2xdj7bmfdmjqyaudq
    ONTO m16xh577hkoqtuq6q77nq5xfexd3zww4z5wng7p6sbo2jyfmhcmesq
{
  ALTER TYPE default::BlogCategory {
      ALTER PROPERTY featured_order {
          SET default := 1;
          DROP CONSTRAINT std::min_value(0);
          CREATE REWRITE
              INSERT 
              USING ((<std::int16>{} IF ((__subject__.featured_order ?= <std::int16>{}) OR (__subject__.featured_order < 1)) ELSE __subject__.featured_order));
          CREATE REWRITE
              UPDATE 
              USING ((<std::int16>{} IF ((__subject__.featured_order ?= <std::int16>{}) OR (__subject__.featured_order < 1)) ELSE __subject__.featured_order));
      };
  };
  UPDATE default::BlogCategory FILTER .featured_order < 1 SET { featured_order := {} };
  ALTER TYPE default::BlogCategory {
      ALTER PROPERTY featured_order {
          CREATE CONSTRAINT std::min_value(1);
      };
  };
  ALTER TYPE default::BlogPost {
      ALTER PROPERTY highlighted_order {
          SET default := 1;
          DROP CONSTRAINT std::min_value(0);
          CREATE REWRITE
              INSERT 
              USING ((<std::int16>{} IF ((__subject__.highlighted_order ?= <std::int16>{}) OR (__subject__.highlighted_order < 1)) ELSE __subject__.highlighted_order));
          CREATE REWRITE
              UPDATE 
              USING ((<std::int16>{} IF ((__subject__.highlighted_order ?= <std::int16>{}) OR (__subject__.highlighted_order < 1)) ELSE __subject__.highlighted_order));
      };
  };
  UPDATE default::BlogPost FILTER .highlighted_order < 1 SET { highlighted_order := {} };
  ALTER TYPE default::BlogPost {
      ALTER PROPERTY highlighted_order {
          CREATE CONSTRAINT std::min_value(1);
      };
  };
};
