//module 0x1::DataFormat {
//    use Std::Vector;
//    use Std::Signer;
//    const ADMIN: address = @0x2;
//    const ENOT_ADMIN: u64 = 0;
//
//    struct JsonType has store {
//        name: vector<u8>,
//    }
//
//    fun number_type(): JsonType {
//        JsonType { name: b"number" }
//    }
//
//    fun str_type(): JsonType {
//        JsonType { name: b"str" }
//    }
//
//    fun bool_type(): JsonType {
//        JsonType { name: b"bool" }
//    }
//
//    struct Field has store {
//        field_name: vector<u8>,
//        field_type: JsonType,
//    }
//
//    struct DataFormat has key, store {
//        fields: vector<Field>
//    }
//
//    public fun video_picture_template(): DataFormat {
//        let fields = Vector::empty();
//        Vector::push_back(&mut fields, Field { field_name: b"file_format", field_type: str_type() });
//        Vector::push_back(&mut fields, Field { field_name: b"size", field_type: number_type() });
//        Vector::push_back(&mut fields, Field { field_name: b"is_awesome", field_type: bool_type() });
//        DataFormat { fields }
//    }
//
//    public(script) fun init_video_picture_template(admin: signer) {
//        assert(Signer::address_of(&admin) == ADMIN, ENOT_ADMIN);
//        move_to(&admin, video_picture_template());
//    }
//}
