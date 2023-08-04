use openapi::Result;
use poem::web::Data;
use poem_openapi::{payload::Json, Object, OpenApi};
use sqlx::PgPool;

#[derive(Object)]
struct Product {
    id: i64,
    name: String,
    description: String,
    parent_product_id: i64,
    purchase_unit_id: i64,
    stock_unit_id: i64,
    purchase_to_stock_factor: f32,
}

type GetAllProductsResponse = Result<Json<Vec<Product>>>;

#[derive(Object)]
struct Space {
    id: i64,
    name: String,
    description: String,
}

type GetAllSpacesResponse = Result<Json<Vec<Space>>>;

#[derive(Object)]
struct Place {
    id: i64,
    name: String,
    description: String,
}

type GetAllPlacesResponse = Result<Json<Vec<Place>>>;

#[derive(Object)]
struct Unit {
    id: i64,
    singular: String,
    plural: String,
}

type GetAllUnitsResponse = Result<Json<Vec<Unit>>>;

#[derive(Object)]
struct UnitConversion {
    id: i64,
    from_unit_id: i64,
    to_unit_id: i64,
    factor: f32,
}

type GetAllUnitConversionsResponse = Result<Json<Vec<UnitConversion>>>;

struct UkisApi;

#[OpenApi]
impl UkisApi {
    #[oai(path = "/products", method = "get")]
    async fn get_products(&self, pool: Data<&PgPool>) -> GetAllProductsResponse {
        todo!()
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    todo!()
}
