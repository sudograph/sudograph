use sudograph::graphql_database;

graphql_database!("canisters/graphql/src/schema.graphql");

type PrimaryKey = String;
type MessageStore = HashMap<PrimaryKey, Option<Message>>;

async fn custom_get(id: ID) -> Result<Option<Message>, sudograph::async_graphql::Error> {
    let message_store = sudograph::ic_cdk::storage::get::<MessageStore>();

    let message_option = message_store.get(&id.to_string());

    match message_option {
        Some(message) => {
            return Ok(message.clone());
        },
        None => {
            return Ok(None);
        }
    };
}

async fn custom_set(id: ID, text: Option<String>) -> Result<bool, sudograph::async_graphql::Error> {
    let message_store = sudograph::ic_cdk::storage::get_mut::<MessageStore>();

    let message = match text {
        Some(text_value) => Some(Message {
            id: id.clone(),
            text: text_value
        }),
        None => None
    };

    message_store.insert(id.to_string(), message);

    return Ok(true);
}