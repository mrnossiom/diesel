use crate::schema::*;
use diesel::*;

#[diesel_test_helper::test]
fn test_updating_single_column() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    update(users)
        .set(name.eq("Jim"))
        .execute(connection)
        .unwrap();

    let expected_data = vec!["Jim".to_string(); 2];
    let data: Vec<String> = users.select(name).load(connection).unwrap();
    assert_eq!(expected_data, data);
}

#[diesel_test_helper::test]
fn test_updating_single_column_of_single_row() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let sean = find_user_by_name("Sean", connection);

    update(users.filter(id.eq(sean.id)))
        .set(name.eq("Jim"))
        .execute(connection)
        .unwrap();

    let expected_data = vec!["Jim".to_string(), "Tess".to_string()];
    let data: Vec<String> = users.select(name).order(id).load(connection).unwrap();
    assert_eq!(expected_data, data);
}

#[diesel_test_helper::test]
fn test_updating_nullable_column() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let sean = find_user_by_name("Sean", connection);

    update(users.filter(id.eq(sean.id)))
        .set(hair_color.eq(Some("black")))
        .execute(connection)
        .unwrap();

    let data: Option<String> = users
        .select(hair_color)
        .filter(id.eq(sean.id))
        .first(connection)
        .unwrap();
    assert_eq!(Some("black".to_string()), data);

    update(users.filter(id.eq(sean.id)))
        .set(hair_color.eq(None::<String>))
        .execute(connection)
        .unwrap();

    let data: QueryResult<Option<String>> = users
        .select(hair_color)
        .filter(id.eq(sean.id))
        .first(connection);
    assert_eq!(Ok(None::<String>), data);
}

#[diesel_test_helper::test]
fn test_updating_multiple_columns() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let sean = find_user_by_name("Sean", connection);

    update(users.filter(id.eq(sean.id)))
        .set((name.eq("Jim"), hair_color.eq(Some("black"))))
        .execute(connection)
        .unwrap();

    let expected_user = User::with_hair_color(sean.id, "Jim", "black");
    let user = users.find(sean.id).first(connection);
    assert_eq!(Ok(expected_user), user);
}

#[diesel_test_helper::test]
#[cfg(not(any(
    all(feature = "sqlite", not(feature = "returning_clauses_for_sqlite_3_35")),
    feature = "mysql"
)))]
fn update_returning_struct() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let sean = find_user_by_name("Sean", connection);
    let user = update(users.filter(id.eq(sean.id)))
        .set(hair_color.eq("black"))
        .get_result(connection);
    let expected_user = User::with_hair_color(sean.id, "Sean", "black");

    assert_eq!(Ok(expected_user), user);
}

#[diesel_test_helper::test]
#[cfg(not(any(
    all(feature = "sqlite", not(feature = "returning_clauses_for_sqlite_3_35")),
    feature = "mysql"
)))]
fn update_with_custom_returning_clause() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let sean = find_user_by_name("Sean", connection);
    let user = update(users.filter(id.eq(sean.id)))
        .set(hair_color.eq("black"))
        .returning((name, hair_color))
        .get_result::<(String, Option<String>)>(connection);
    let expected_result = ("Sean".to_string(), Some("black".to_string()));

    assert_eq!(Ok(expected_result), user);
}

#[diesel_test_helper::test]
fn update_with_struct_as_changes() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let sean = find_user_by_name("Sean", connection);
    let changes = NewUser::new("Jim", Some("blue"));

    update(users.filter(id.eq(sean.id)))
        .set(&changes)
        .execute(connection)
        .unwrap();
    let user = users.find(sean.id).first(connection);
    let expected_user = User::with_hair_color(sean.id, "Jim", "blue");

    assert_eq!(Ok(expected_user), user);
}

#[diesel_test_helper::test]
fn save_on_struct_with_primary_key_changes_that_struct() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let sean = find_user_by_name("Sean", connection);
    let user = User::with_hair_color(sean.id, "Jim", "blue").save_changes::<User>(connection);

    let user_in_db = users.find(sean.id).first(connection);

    assert_eq!(user, user_in_db);
}

#[diesel_test_helper::test]
fn sql_syntax_is_correct_when_option_field_comes_before_non_option() {
    #[derive(AsChangeset)]
    #[diesel(table_name = users)]
    struct Changes {
        hair_color: Option<String>,
        name: String,
    }

    let changes = Changes {
        hair_color: None,
        name: "Jim".into(),
    };
    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let sean = find_user_by_name("Sean", connection);
    update(users::table.filter(users::id.eq(sean.id)))
        .set(&changes)
        .execute(connection)
        .unwrap();
    let user = users::table.find(sean.id).first(connection);

    let expected_user = User::new(sean.id, "Jim");
    assert_eq!(Ok(expected_user), user);
}

#[diesel_test_helper::test]
fn sql_syntax_is_correct_when_option_field_comes_mixed_with_non_option() {
    #[derive(AsChangeset)]
    #[diesel(table_name = posts)]
    struct Changes {
        user_id: i32,
        title: Option<String>,
        body: String,
    }

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let sean = find_user_by_name("Sean", connection);
    let new_post = sean.new_post("Hello", Some("world"));
    insert_into(posts::table)
        .values(&new_post)
        .execute(connection)
        .unwrap();

    let changes = Changes {
        user_id: 1,
        title: None,
        body: "earth".into(),
    };
    update(posts::table)
        .set(&changes)
        .execute(connection)
        .unwrap();
    let post = posts::table
        .order(posts::id.desc())
        .first::<Post>(connection)
        .unwrap();

    let expected_post = Post::new(post.id, sean.id, "Hello", Some("earth"));
    assert_eq!(expected_post, post);
}

#[diesel_test_helper::test]
fn update_with_no_changes() {
    #[derive(AsChangeset)]
    #[diesel(table_name = users)]
    struct Changes {
        name: Option<String>,
        hair_color: Option<String>,
    }

    let connection = &mut connection();
    let changes = Changes {
        name: None,
        hair_color: None,
    };
    let update_result = update(users::table).set(&changes).execute(connection);
    assert!(update_result.is_err());
}

#[diesel_test_helper::test]
fn update_with_optional_empty_changeset() {
    #[derive(AsChangeset)]
    #[diesel(table_name = users)]
    struct Changes {
        name: Option<String>,
        hair_color: Option<String>,
    }

    let connection = &mut connection();
    let changes = Changes {
        name: None,
        hair_color: None,
    };
    let update_result = update(users::table)
        .set(&changes)
        .execute(connection)
        .optional_empty_changeset()
        .unwrap();
    assert_eq!(None, update_result);
}

#[diesel_test_helper::test]
#[cfg(any(feature = "postgres", feature = "sqlite"))]
fn upsert_with_no_changes_executes_do_nothing() {
    #[derive(AsChangeset)]
    #[diesel(table_name = users)]
    struct Changes {
        hair_color: Option<String>,
    }

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let result = insert_into(users::table)
        .values(&User::new(1, "Sean"))
        .on_conflict(users::id)
        .do_update()
        .set(&Changes { hair_color: None })
        .execute(connection);

    assert_eq!(Ok(0), result);
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn upsert_with_no_changes_executes_do_nothing() {
    #[derive(AsChangeset)]
    #[diesel(table_name = users)]
    struct Changes {
        hair_color: Option<String>,
    }

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let result = insert_into(users::table)
        .values(&User::new(1, "Sean"))
        .on_conflict(diesel::dsl::DuplicatedKeys)
        .do_update()
        .set(&Changes { hair_color: None })
        .execute(connection);

    #[cfg(not(feature = "mysql"))]
    assert_eq!(Ok(0), result);
    #[cfg(feature = "mysql")]
    assert_eq!(Ok(1), result);
}

#[diesel_test_helper::test]
#[cfg(any(feature = "postgres", feature = "sqlite"))]
fn upsert_with_no_changes_executes_do_nothing_owned() {
    #[derive(AsChangeset)]
    #[diesel(table_name = users)]
    struct Changes {
        hair_color: Option<String>,
    }

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let result = insert_into(users::table)
        .values(User::new(1, "Sean"))
        .on_conflict(users::id)
        .do_update()
        .set(&Changes { hair_color: None })
        .execute(connection);

    assert_eq!(Ok(0), result);
}

#[diesel_test_helper::test]
#[cfg(feature = "mysql")]
fn upsert_with_no_changes_executes_do_nothing_owned() {
    #[derive(AsChangeset)]
    #[diesel(table_name = users)]
    struct Changes {
        hair_color: Option<String>,
    }

    let connection = &mut connection_with_sean_and_tess_in_users_table();
    let result = insert_into(users::table)
        .values(User::new(1, "Sean"))
        .on_conflict(diesel::dsl::DuplicatedKeys)
        .do_update()
        .set(&Changes { hair_color: None })
        .execute(connection);

    assert_eq!(Ok(1), result);
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn upsert_with_sql_literal_for_target() {
    use crate::schema::users::dsl::*;
    use diesel::dsl::sql;
    use diesel::sql_types::Text;
    use diesel::upsert::*;

    let connection = &mut connection();
    diesel::sql_query(
        "CREATE TEMPORARY TABLE users(\
             id SERIAL PRIMARY KEY, \
             name TEXT NOT NULL, \
             hair_color TEXT)",
    )
    .execute(connection)
    .unwrap();
    // This index needs to happen before the insert or we'll get a deadlock
    // with any transactions that are trying to get the row lock from insert
    diesel::sql_query("CREATE UNIQUE INDEX ON users (name) WHERE name != 'Tess'")
        .execute(connection)
        .unwrap();
    insert_sean_and_tess_into_users_table(connection);

    let new_users = vec![
        NewUser::new("Sean", Some("Green")),
        NewUser::new("Tess", Some("Blue")),
    ];
    insert_into(users)
        .values(&new_users)
        .on_conflict(sql::<Text>("(name) WHERE name != 'Tess'"))
        .do_update()
        .set(hair_color.eq(excluded(hair_color)))
        .execute(connection)
        .unwrap();

    let data = users.select((name, hair_color)).order(id).load(connection);
    let expected_data = vec![
        ("Sean".to_string(), Some("Green".to_string())),
        ("Tess".to_string(), None),
        ("Tess".to_string(), Some("Blue".to_string())),
    ];
    assert_eq!(Ok(expected_data), data);
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn upsert_with_sql_literal_for_target_with_condition() {
    use crate::schema::users::dsl::*;
    use diesel::dsl::sql;
    use diesel::query_dsl::methods::FilterDsl;
    use diesel::sql_types::Text;
    use diesel::upsert::*;

    // cannot run these tests in parallel due to index creation

    let connection = &mut connection();
    diesel::sql_query(
        "CREATE TEMPORARY TABLE users(\
             id SERIAL PRIMARY KEY, \
             name TEXT NOT NULL, \
             hair_color TEXT)",
    )
    .execute(connection)
    .unwrap();
    // This index needs to happen before the insert or we'll get a deadlock
    // with any transactions that are trying to get the row lock from insert
    diesel::sql_query("CREATE UNIQUE INDEX ON users (name) WHERE name != 'Tess'")
        .execute(connection)
        .unwrap();
    insert_sean_and_tess_into_users_table(connection);

    let new_users = vec![
        NewUser::new("Sean", Some("Green")),
        NewUser::new("Tess", Some("Blue")),
    ];
    insert_into(users)
        .values(&new_users)
        .on_conflict(sql::<Text>("(name) WHERE name != 'Tess'"))
        .do_update()
        .set(hair_color.eq(excluded(hair_color)))
        .filter(id.le(5))
        .execute(connection)
        .unwrap();

    let data = users.select((name, hair_color)).order(id).load(connection);
    let expected_data = vec![
        ("Sean".to_string(), Some("Green".to_string())),
        ("Tess".to_string(), None),
        ("Tess".to_string(), Some("Blue".to_string())),
    ];
    assert_eq!(Ok(expected_data), data);
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn update_array_index_expression() {
    use crate::schema::posts::dsl::*;

    let connection = &mut connection_with_sean_and_tess_in_users_table();

    let sean = find_user_by_name("Sean", connection);
    let new_post = sean.new_post("Hello", Some("world"));
    insert_into(posts)
        .values(&new_post)
        .execute(connection)
        .unwrap();
    update(posts)
        .set(tags.eq(vec!["programming", "rust"]))
        .execute(connection)
        .unwrap();
    update(posts)
        .set(tags.index(1).eq("postgres"))
        .execute(connection)
        .unwrap();
    let data = posts.select(tags).load(connection);
    let expected_data = vec![vec!["postgres".to_string(), "rust".to_string()]];

    assert_eq!(Ok(expected_data), data);
}
