CREATE MIGRATION m1hffac2xe4gln55x2utfayoaaq454gswhlp2gxzjkkbp6h357znta
    ONTO m1irqlfhdhhh6kplrjlcsdu2vh5iv5vyvzwvrk3wxphrwvpptu52tq
{
                      ALTER TYPE default::BlogPost {
      CREATE INDEX ON (std::str_lower(.title));
  };
};
