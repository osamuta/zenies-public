use super::*;
use futures::StreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

pub const DATABASE_URL: &str = "mongodb://TEST:TEST@127.0.0.1:27017/?authSource=TEST&authMechanism=SCRAM-SHA-256&readPreference=primary&appname=MongoDB%20Compass&ssl=false";
pub const DATABASE_NAME: &str = "TEST";
pub const COLLECTION_NAME: &str = "TEST";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Test {
    _id: bson::oid::ObjectId,
    name: String,
    number: i32,
    float: f64,
    switch: bool,
    array: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct All {}

#[tokio::test]
async fn check_basic_crud_operations() -> Result<(), String> {
    let data = vec![
        Test {
            _id: bson::oid::ObjectId::new(),
            name: String::from("hello"),
            number: 0,
            float: 3.14,
            switch: true,
            array: vec![String::from("zero"), String::from("one")],
        },
        Test {
            _id: bson::oid::ObjectId::new(),
            name: String::from("good morning"),
            number: 1,
            float: 2.71,
            switch: false,
            array: vec![
                String::from("zero"),
                String::from("one"),
                String::from("two"),
            ],
        },
    ];
    let mut data_doc = Vec::with_capacity(data.len());
    for d in &data {
        match bson::to_document(&d) {
            Ok(result) => data_doc.push(result),
            Err(result) => {
                return Err(error_message!(
                "failed to convert data to bson document!\ndetails : {:?}\nreceived data : {:?}",
                result,
                &data
            ))
            }
        }
    }

    //
    // connect to database
    let database = Database::new(DATABASE_URL, DATABASE_NAME).await?;

    //
    // create
    database.create(COLLECTION_NAME, &data, None).await?;

    //
    // update

    //
    // request
    let mut received_data = Vec::new();
    let mut cursor = database.request(COLLECTION_NAME, &All {}, None).await?;
    while let Some(d) = cursor.next().await {
        match d {
            Ok(result) => received_data.push(result),
            Err(result) => return Err(error_message!("{:?}", result)),
        }
    }

    assert_eq!(received_data, data_doc);

    database.delete(COLLECTION_NAME, &All {}, None).await?;
    Ok(())
}

#[tokio::test]
async fn check_loacl_database() {
    let ticker = local::LocalTicker::new();
    let executions = local::LocalExecutions::new();
    let order_book = local::LocalOrderBook::new();

    assert_eq!(ticker.write("../../zenies/etc/test_1.bson").await, Ok(()));
    assert_eq!(
        executions.write("../../zenies/etc/test_2.bson").await,
        Ok(())
    );
    assert_eq!(
        order_book.write("../../zenies/etc/test_3.bson").await,
        Ok(())
    );

    assert_eq!(
        local::LocalTicker::read("../../zenies/etc/test_1.bson")
            .await
            .is_ok(),
        true
    );
    assert_eq!(
        local::LocalExecutions::read("../../zenies/etc/test_2.bson")
            .await
            .is_ok(),
        true
    );
    assert_eq!(
        local::LocalOrderBook::read("../../zenies/etc/test_3.bson")
            .await
            .is_ok(),
        true
    );
}

#[tokio::test]
async fn check_ticker_deliver() {
    let database = match Database::new(DATABASE_URL, common_constants::DATABASE_NAME).await {
        Ok(result) => result,
        Err(result) => {
            eprintln!("{}", result);
            return;
        }
    };

    let start_time = Utc.ymd(2021, 8, 1).and_hms(0, 0, 0);
    let end_time = Utc.ymd(2021, 8, 1).and_hms(0, 0, 1);

    let query = create_duration_query(
        common_constants::DATABASE_COLLECTION_TICKER,
        &start_time,
        &end_time,
    )
    .expect("failed to create query");
    let options =
        create_find_option_timeline(common_constants::DATABASE_COLLECTION_TICKER, 1, None)
            .expect("failed to create options!");

    let mut ticker_stream = stream::DatabaseStream::<data::Ticker>::request(
        &database,
        None,
        query.clone(),
        options.clone(),
    )
    .await
    .expect("failed to get");

    let mut executions_stream =
        stream::DatabaseStream::<data::Execution>::request(&database, None, query, options)
            .await
            .expect("failed to get");

    let query = create_duration_query(
        common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY,
        &start_time,
        &end_time,
    )
    .expect("failed to create query");
    let options = create_find_option_timeline(
        common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY,
        1,
        None,
    )
    .expect("failed to create options!");

    let mut order_book_stream = stream::DatabaseStream::<data::OrderBook>::request(
        &database,
        Some(common_constants::DATABASE_COLLECTION_ORDER_BOOK_BUY),
        query,
        options,
    )
    .await
    .expect("failed to get");

    while let Some(v) = ticker_stream.next().await {
        println!("GOT = {:?}", v);
    }

    while let Some(v) = executions_stream.next().await {
        println!("GOT = {:?}", v);
    }

    while let Some(v) = order_book_stream.next().await {
        println!("GOT = {:?}", v);
    }
}

#[tokio::test]
async fn check_type() {
    println!("{}", std::any::type_name::<data::Ticker>());
}

#[test]
fn check_rayon() {
    let vec = vec![1, 2, 3, 4, 5];
    assert_eq!(Some(&3), vec.par_iter().find_first(|&&x| x > 2));
}
