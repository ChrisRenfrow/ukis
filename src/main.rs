use poem::{listener::TcpListener, web::Data, EndpointExt, Result, Route, Server};
use poem_openapi::{payload::Json, types::ToJSON, Object, OpenApi, OpenApiService};
use sqlx::{types::time::Time, PgPool};

#[derive(Object)]
struct Product {
    id: i64,
    name: String,
    description: String,
    parent_product_id: i64,
    purchase_unit_id: i64,
    stock_unit_id: i64,
    purchase_to_stock_factor: f32,
    created_timestamp: Time,
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

impl ToJSON for Time {
    fn to_json(&self) -> Option<String> {
        // self.format().unwrap()
        todo!()
    }
}

struct UkisApi;

#[OpenApi]
impl UkisApi {
    #[oai(path = "/products", method = "get")]
    async fn get_products(&self, pool: Data<&PgPool>) -> GetAllProductsResponse {
        let products = sqlx::query_as!(Product, "select * from products")
            .fetch_all(pool.0)
            .await
            .unwrap();

        Ok(Json(products))
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres:ukis-dev").await?;
    let api_service = OpenApiService::new(UkisApi, "Unnamed Kitchen Inventory System API", "0.0.1")
        .server("http://localhost:9694");
    let ui = api_service.openapi_explorer();
    let route = Route::new()
        .nest("/", api_service)
        .nest("/ui", ui)
        .data(pool);
    Server::new(TcpListener::bind("127.0.0.1:9694"))
        .run(route)
        .await?;
    Ok(())
}
