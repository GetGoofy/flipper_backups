use postgres::{Client as PostgresClient};

pub fn prepare_table_tsm_pricing_data_weekly(client: &mut PostgresClient) {
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS tsm_pricing_data_weekly (
            id SERIAL PRIMARY KEY,
            region_id BIGINT,
            item_id BIGINT,
            avg_sale_price BIGINT,
            sold_per_day DECIMAL,
            sale_rate DECIMAL,
            quantity BIGINT,
            created_at TEXT
        )
    ").expect(
            "An error happened when attempting to execute create table if not exists on the DB",
        );
}