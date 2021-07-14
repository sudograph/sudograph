// TODO I will want to create an arbitrary object_type for when we start creating arbitrary schemas
// TODO the below could really help here
// TODO this article was so helpful: https://translate.google.com/translate?hl=en&sl=ja&u=https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f&prev=search&pto=aue
// TODO I think this is the original Japanese article: https://qiita.com/legokichi/items/2c3fdcbf84d959668a0f

// TODO it should be pretty easy to make an arbitrary ObjectType and Document
// TODO ooh...I think we want to create an arbitrary Document actually
// TODO and make sure there is an easy way to turn a document into a string
// TODO then just save that string to disk and make a way to deploy it, run all of the tests, and then do it again

// #[derive(Clone, Debug, Arbitrary)]
// pub struct Arbs {
//     arb_include_id: bool,
//     arb_blob_bool: bool,
//     arb_blob_string: String,
//     arb_blob_vector: Vec<u8>,
//     arb_boolean: bool,
//     #[proptest(strategy="crate::arbitraries::datetime::arb_datetime()")]
//     arb_datetime: String,
//     arb_float: f32,
//     #[proptest(strategy="crate::arbitraries::string::arb_string()")]
//     arb_id: String,
//     arb_int: i32,
//     #[proptest(strategy="crate::arbitraries::string::arb_string()")]
//     arb_string: String,
//     #[proptest(strategy="crate::arbitraries::json::arb_json()")]
//     arb_json: Json,
//     arb_enum: String // TODO this needs to be based off of the actual enum field
// }