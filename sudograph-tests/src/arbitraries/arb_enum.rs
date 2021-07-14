use proptest::{
    prelude::any,
    strategy::Strategy
};
use graphql_parser::schema::{
    Field,
    Document
};

use crate::utilities::graphql::get_enum_type_from_field;

pub fn arb_enum<'a>(
    graphql_ast: &'a Document<'a ,String>,
    field: &Field<String>
) -> impl Strategy<Value = String> + 'a {
    // TODO we need to get the field
    // TODO we need to get all of the values
    // TODO we then need to grab just one of them
    let enum_type = get_enum_type_from_field(
        graphql_ast,
        field
    ).unwrap();

    // enum_type.values.len();

    let enum_values_len = enum_type.values.len();

    // proptest::prelude::prop::num::i32::
    // (0..10).new
    

    // return any::<[0..2i32]>().prop_map(|string| {
    //     return string.replace("\\", "").replace("\"", "");
    // });

    return (0..enum_values_len - 1).prop_map(move |index| {
        return enum_type.values.get(index).unwrap().name.clone();
    });
}