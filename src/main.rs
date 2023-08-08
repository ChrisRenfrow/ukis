use poem::{
    error::InternalServerError,
    listener::TcpListener,
    web::{Data, Path},
    EndpointExt, Result, Route, Server,
};
use poem_openapi::{payload::Json, Object, OpenApi, OpenApiService};
use sqlx::PgPool;

#[derive(Object)]
struct Product {
    /// The id of the product
    #[oai(read_only)]
    id: i64,
    /// The name of the product
    name: String,
    /// A description for the product
    description: Option<String>,
    /// The product's parent product id
    parent_product_id: Option<i32>,
    /// The `Unit` id to use when accounting for the product from a purchase
    purchase_unit_id: Option<i32>,
    /// The `Unit` id to use when adding the product to stock
    stock_unit_id: Option<i32>,
    /// The factor of purchase unit to stock unit
    /// (**e.g.** 1 carton of eggs is equivalent to 12 eggs in stock, so the factor would be *12.0*)
    purchase_to_stock_factor: f32,
}

type GetAllProductsResponse = Result<Json<Vec<Product>>>;

#[derive(Object)]
struct Space {
    /// The id of the space
    #[oai(read_only)]
    id: i64,
    /// The name of the space
    name: String,
    /// A description for the space
    description: Option<String>,
}

type GetAllSpacesResponse = Result<Json<Vec<Space>>>;

#[derive(Object)]
struct Place {
    /// The id of the place
    #[oai(read_only)]
    id: i64,
    /// The name of the place
    name: String,
    /// A description for the place
    description: Option<String>,
}

type GetAllPlacesResponse = Result<Json<Vec<Place>>>;

#[derive(Object)]
struct Unit {
    /// The id of the unit
    #[oai(read_only)]
    id: i64,
    /// The singular form of the unit
    /// **e.g.** gram
    singular: String,
    /// The plural form of the unit (if applicable)
    /// **e.g.** grams
    plural: Option<String>,
}

type GetAllUnitsResponse = Result<Json<Vec<Unit>>>;

#[derive(Object)]
struct UnitConversion {
    /// The id of the unit conversion
    #[oai(read_only)]
    id: i64,
    /// The id of the unit to convert from
    from_unit_id: i64,
    /// The id of the unit to convert to
    to_unit_id: i64,
    /// The factor from unit to unit
    factor: f32,
}

type GetAllUnitConversionsResponse = Result<Json<Vec<UnitConversion>>>;

struct UkisApi;

#[OpenApi]
impl UkisApi {
    #[oai(path = "/products", method = "get")]
    async fn get_products(&self, pool: Data<&PgPool>) -> GetAllProductsResponse {
        let products = sqlx::query_as!(Product, "SELECT * FROM products")
            .fetch_all(pool.0)
            .await
            .unwrap();

        Ok(Json(products))
    }

    #[oai(path = "/products/:id", method = "get")]
    async fn get_product(&self, pool: Data<&PgPool>, id: Path<i32>) -> Result<Json<Product>> {
        let product = sqlx::query_as!(Product, "SELECT * FROM products WHERE id = $1", id.0)
            .fetch_one(pool.0)
            .await
            .map_err(InternalServerError)?;

        Ok(Json(product))
    }

    #[oai(path = "/products", method = "post")]
    async fn new_product(&self, pool: Data<&PgPool>, product: Json<Product>) -> Result<Json<i32>> {
        let record = sqlx::query!(
            r#"
INSERT INTO products (name, description, parent_product_id, purchase_unit_id, stock_unit_id, purchase_to_stock_factor)
VALUES ($1, $2, $3, $4, $5, $6)
RETURNING id"#,
            product.name,
            product.description,
            product.parent_product_id,
            product.purchase_unit_id,
            product.stock_unit_id,
            product.purchase_to_stock_factor
        )
        .fetch_one(pool.0)
        .await
        .map_err(InternalServerError)?;

        Ok(Json(record.id))
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
