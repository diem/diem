address 0x2 {
module M {

    fun t() {
        use 0x2::Mango;
        use 0x2::Mango as M;
        use 0x2::Mango::baz;
        use 0x2::Salsa::{Self, foo as bar, foo};
        let x = {
            use 0x2::Mango;
            use 0x3::Mango as M;
            use 0x3::Mango::baz;
            use 0x3::Salsa::{Self, foo as bar, foo};

            0
        };
        {
            {
                {
                    {
                        {
                            {
                                {
                                    {
                                        {
                                            {
                                                use 0x2::Mango;
                                                use 0x3::Mango as M;
                                                use 0x3::Mango::baz;
                                                use 0x3::Salsa::{Self, foo as bar, foo};
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        while (true) {
            use 0x2::Mango;
            use 0x3::Mango as M;
            use 0x3::Mango::baz;
            use 0x3::Salsa::{Self, foo as bar, foo};

        }
    }
}
}
