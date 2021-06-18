use sudograph::graphql_database;
use sudograph::async_graphql::MergedObject;

graphql_database!("canisters/graphql/src/schema.graphql");

async fn query1() -> i32 {
    return 10;
}

async fn query2() -> Option<i32> {
    return Some(5);
}

async fn query3() -> f32 {
    return 5.34;
}

async fn query4() -> Option<f32> {
    return Some(523.44);
}

async fn query5() -> ID {
    return ID(String::from("hello"));
}

async fn query6() -> Vec<ID> {
    return vec![];
}

async fn query7() -> Option<Vec<ID>> {
    return Some(vec![]);
}

async fn query8() -> Option<Vec<Option<ID>>> {
    return Some(vec![]);
}

async fn query9() -> Vec<Option<ID>> {
    return vec![Some(ID(String::from("monkey")))];
}

async fn query10() -> Book {
    return Book {
        id: ID(String::from("0"))
    };
}

async fn query11() -> Option<Book> {
    return Some(Book {
        id: ID(String::from("0"))
    });
}

async fn query12() -> Vec<Book> {
    return vec![Book {
        id: ID(String::from("0"))
    }];
}

async fn query13() -> Vec<Option<Book>> {
    return vec![Some(Book {
        id: ID(String::from("0"))
    })];
}

async fn query14() -> Option<Vec<Book>> {
    return Some(vec![Book {
        id: ID(String::from("0"))
    }]);
}

async fn query15() -> Option<Vec<Option<Book>>> {
    return Some(vec![Some(Book {
        id: ID(String::from("0"))
    })]);
}

async fn query16(test: i32) -> i32 {
    return test;
}

// TODO we should be able to get rid of the entire need for this
// TODO if we force the user to create a type Query or type Mutation, we can just generate the functions
// TODO if there is no @canister directive, just call a local Rust function of the same name with the same parameters
// TODO always allow exposing the async_graphql underpinnings if possible