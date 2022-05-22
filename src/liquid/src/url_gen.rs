use super::currency::*;

pub fn generate_get_product_url(pair: CurrencyPair) -> String {
    format!("https://api.liquid.com/products/{}", pair.generate_id())
}

pub fn generate_get_order_book_url(pair: CurrencyPair) -> String {
    format!(
        "https://api.liquid.com/products/{}/price_levels",
        pair.generate_id()
    )
}
