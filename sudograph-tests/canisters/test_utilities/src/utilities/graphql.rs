// TODO many of the graphql utility functions are verbatim copied multiple times within the sudograph repository
// TODO perhaps we should create a single crate to store all of the utilities in

use graphql_parser::schema::{
    Definition,
    Document,
    EnumType,
    Field,
    ObjectType,
    Type,
    TypeDefinition
};

// TODO I would love to figure out how to get this function to work
// TODO but the compiler won't let me
// fn get_object_types_from_schema(schema_path: &str) -> Vec<ObjectType<String>> {
//     let schema_file_contents = fs::read_to_string(schema_path).unwrap();
//     let graphql_ast = parse_schema::<String>(&schema_file_contents).unwrap();
//     let object_types = get_object_types(&graphql_ast);

//     return object_types;
// }

// TODO this was copied straight from sudograph/sudograph-generate/src/lib.rs
pub fn get_object_types<'a>(graphql_ast: &Document<'a, String>) -> Vec<ObjectType<'a, String>> {
    let type_definitions: Vec<TypeDefinition<String>> = graphql_ast.definitions.iter().filter_map(|definition| {
        match definition {
            Definition::TypeDefinition(type_definition) => {
                return Some(type_definition.clone());
            },
            _ => {
                return None;
            }
        };
    }).collect();

    let object_types: Vec<ObjectType<String>> = type_definitions.into_iter().filter_map(|type_definition| {
        match type_definition {
            TypeDefinition::Object(object_type) => {
                return Some(object_type);
            },
            _ => {
                return None;
            }
        }
    }).collect();

    return object_types;
}

// TODO this is now copied inside and outside of the quote
// TODO many of the functions are copied, we need to organize this better
pub fn get_graphql_type_name(graphql_type: &Type<String>) -> String {
    match graphql_type {
        Type::NamedType(named_type) => {
            return String::from(named_type);
        },
        Type::NonNullType(non_null_type) => {
            return get_graphql_type_name(non_null_type);
        },
        Type::ListType(list_type) => {
            return get_graphql_type_name(list_type);
        }
    };
}

pub fn is_graphql_type_an_enum(
    graphql_ast: &Document<String>,
    graphql_type: &Type<String>
) -> bool {
    let enum_types = get_enum_types(graphql_ast);
    let graphql_type_name = get_graphql_type_name(graphql_type);

    let graphql_type_is_an_enum = enum_types.iter().any(|enum_type| {
        return enum_type.name == graphql_type_name;
    });

    return graphql_type_is_an_enum;
}

fn get_enum_types<'a>(graphql_ast: &Document<'a, String>) -> Vec<EnumType<'a, String>> {
    let type_definitions: Vec<TypeDefinition<String>> = graphql_ast.definitions.iter().filter_map(|definition| {
        match definition {
            Definition::TypeDefinition(type_definition) => {
                return Some(type_definition.clone());
            },
            _ => {
                return None;
            }
        };
    }).collect();

    let enum_types: Vec<EnumType<String>> = type_definitions.into_iter().filter_map(|type_definition| {
        match type_definition {
            TypeDefinition::Enum(enum_type) => {
                return Some(enum_type);
            },
            _ => {
                return None;
            }
        }
    }).collect();

    return enum_types;
}

pub fn get_enum_type_from_field<'a>(
    graphql_ast: &Document<'a, String>,
    field: &Field<String>
) -> Option<EnumType<'a, String>> {
    let enum_type_name = get_graphql_type_name(&field.field_type);

    let enum_types = get_enum_types(graphql_ast);

    return enum_types.into_iter().find(|enum_type| {
        return enum_type.name == enum_type_name;
    }).clone();
}

pub fn is_graphql_type_nullable(graphql_type: &Type<String>) -> bool {
    match graphql_type {
        Type::NonNullType(_) => {
            return false;
        },
        _ => {
            return true;
        }
    };
}

pub fn is_graphql_type_a_relation_many(
    graphql_ast: &Document<String>,
    graphql_type: &Type<String>
) -> bool {
    let object_types = get_object_types(graphql_ast);
    let graphql_type_name = get_graphql_type_name(graphql_type);

    let graphql_type_is_a_relation = object_types.iter().any(|object_type| {
        return object_type.name == graphql_type_name;
    });

    let graphql_type_is_a_list_type = is_graphql_type_a_list_type(
        graphql_ast,
        graphql_type
    );

    return 
        graphql_type_is_a_relation == true &&
        graphql_type_is_a_list_type == true
    ;
}

pub fn is_graphql_type_a_relation_one(
    graphql_ast: &Document<String>,
    graphql_type: &Type<String>
) -> bool {
    let object_types = get_object_types(graphql_ast);
    let graphql_type_name = get_graphql_type_name(graphql_type);

    let graphql_type_is_a_relation = object_types.iter().any(|object_type| {
        return object_type.name == graphql_type_name;
    });

    let graphql_type_is_a_list_type = is_graphql_type_a_list_type(
        graphql_ast,
        graphql_type
    );

    return 
        graphql_type_is_a_relation == true &&
        graphql_type_is_a_list_type == false
    ;
}

fn is_graphql_type_a_list_type(
    graphql_ast: &Document<String>,
    graphql_type: &Type<String>
) -> bool {
    match graphql_type {
        Type::NamedType(_) => {
            return false;
        },
        Type::NonNullType(non_null_type) => {
            return is_graphql_type_a_list_type(
                graphql_ast,
                non_null_type
            );
        },
        Type::ListType(_) => {
            return true;
        }
    };
}

// TODO this search needs to exclude the relation's own entity field...
// TODO you could have a relation to your same type, but you need to skip your original field
pub fn get_opposing_relation_field<'a>(
    graphql_ast: &'a Document<'a, String>,
    relation_field: &Field<String>
) -> Option<Field<'a, String>> {
    let relation_name = get_directive_argument_value_from_field(
        relation_field,
        "relation",
        "name"
    )?;

    let opposing_object_type_name = get_graphql_type_name(&relation_field.field_type);
    
    let object_types = get_object_types(graphql_ast);

    return object_types.iter().filter(|object_type| {
        return object_type.name == opposing_object_type_name; // TODO a find might make more sense than a filter
    }).fold(None, |_, object_type| {
        return object_type.fields.iter().fold(None, |result, field| {
            if result != None {
                return result;
            }

            let opposing_relation_name = get_directive_argument_value_from_field(
                field,
                "relation",
                "name"
            )?;

            if opposing_relation_name == relation_name {
                return Some(field.clone());
            }
            else {
                return result;
            }
        });
    });
}

pub fn get_directive_argument_value_from_field(
    field: &Field<String>,
    directive_name: &str,
    argument_name: &str
) -> Option<String> {
    let directive = field.directives.iter().find(|directive| {
        return directive.name == directive_name;
    })?;

    let argument = directive.arguments.iter().find(|argument| {
        return argument.0 == argument_name;
    })?;

    return Some(argument.1.to_string());
}

pub fn get_object_type_from_field<'a>(
    object_types: &'static Vec<ObjectType<'a, String>>,
    field: &Field<String>
) -> Option<&'static ObjectType<'a, String>> {
    let object_type_name = get_graphql_type_name(&field.field_type);

    return object_types.into_iter().find(|object_type| {
        return object_type.name == object_type_name;
    }).clone();
}

pub fn get_field_by_field_name<'a>(
    object_type: &'a ObjectType<'a, String>,
    field_name: &str
) -> Option<&'a Field<'a, String>> {
    return object_type.fields.iter().find(|field| {
        return field.name == field_name;
    });
}

pub fn get_opposing_relation_fields(
    graphql_ast: &'static Document<String>,
    object_type: &'static ObjectType<String>
) -> Vec<Field<'static, String>> {
    return object_type
        .fields
        .iter()
        .filter(|field| {
            return get_opposing_relation_field_static(
                graphql_ast,
                field
            ).is_some();
        })
        .map(|field| {
            return get_opposing_relation_field_static(
                graphql_ast,
                field
            ).unwrap();
        })
        .collect();
}

pub fn get_opposing_relation_field_static(
    graphql_ast: &'static Document<'static, String>,
    relation_field: &Field<String>
) -> Option<Field<'static, String>> {
    let relation_name = get_directive_argument_value_from_field(
        relation_field,
        "relation",
        "name"
    )?;

    let opposing_object_type_name = get_graphql_type_name(&relation_field.field_type);
    
    let object_types = get_object_types(graphql_ast);

    return object_types.iter().filter(|object_type| {
        return object_type.name == opposing_object_type_name; // TODO a find might make more sense than a filter
    }).fold(None, |_, object_type| {
        return object_type.fields.iter().fold(None, |result, field| {
            if result != None {
                return result;
            }

            let opposing_relation_name = get_directive_argument_value_from_field(
                field,
                "relation",
                "name"
            )?;

            if opposing_relation_name == relation_name {
                return Some(field.clone());
            }
            else {
                return result;
            }
        });
    });
}