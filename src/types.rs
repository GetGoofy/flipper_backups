use rust_decimal::Decimal;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TsmAuthBody {
    pub client_id: String,
    pub grant_type: String,
    pub scope: String,
    pub token: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TsmAuthResponse {
    pub access_token: String
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TsmPricingDataResponse {
    #[serde(rename = "regionId")]
    pub region_id: i64,
    #[serde(rename = "itemId")]
    pub item_id: Option<i64>,
    #[serde(rename = "avgSalePrice")]
    pub avg_sale_price: i64,
    #[serde(rename = "soldPerDay")]
    pub sold_per_day: Decimal,
    #[serde(rename = "saleRate")]
    pub sale_rate: Decimal,
    pub quantity: i64,
}