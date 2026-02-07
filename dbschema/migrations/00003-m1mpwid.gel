CREATE MIGRATION m1mpwidjkrhccm3wqxsaadqy27otoodkvsisf25lfoz6yyowg5l6da
    ONTO m1hffac2xe4gln55x2utfayoaaq454gswhlp2gxzjkkbp6h357znta
{
  ALTER TYPE default::BlogCategory {
      CREATE PROPERTY title_vi: std::str {
          CREATE CONSTRAINT std::max_len_value(50);
      };
  };
};
