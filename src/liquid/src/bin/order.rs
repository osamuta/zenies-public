use liquid::*;

#[tokio::main]
async fn main() {
    let client = LiquidClientAsync::new();

    let order = Order::trailing(5, Side::Sell, 0.0067, TrailType::Percentage, 1.0);
    let key = LiquidApiKey {
        token_id: 2288273,
        secret_key: String::from("Ud42XC+B67MKz6wxiPu4bxs9Uo6MuBZXzkHfPvy5GmiyAFYEFVfmNJXmZeAw7aIt8xNRtDo/VY1zWgCqI2Wl2w=="),
    };

    let response = client
        .post_order(&key, &order)
        .await
        .expect("failed to post a order");
    //let result = client
    //    .is_order_excuted(&key, 4184557293)
    //    .await
    //    .expect("failed to check");
    //println!("{}", result);
    println!("{:?}", response);
    client
        .cancel_order(&key, response.id)
        .await
        .expect("failed to cansel!");
}
