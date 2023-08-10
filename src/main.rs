use poem::{
    error::InternalServerError,
    listener::TcpListener,
    web::{Data, Path},
    EndpointExt, Result, Route, Server,
};
use poem_openapi::{
    payload::{Json, PlainText},
    types::ToJSON,
    ApiResponse, Object, OpenApi, OpenApiService,
};
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
    purchase_to_stock_factor: Option<f32>,
}

type GetAllProductsResponse = Json<Vec<Product>>;

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

type GetAllSpacesResponse = Json<Vec<Space>>;

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

type GetAllPlacesResponse = Json<Vec<Place>>;

#[derive(Object)]
struct Unit {
    /// The id of the unit
    #[oai(read_only)]
    id: i64,
    /// The singular form of the unit
    /// (**e.g.** gram)
    singular: String,
    /// The plural form of the unit, if applicable
    /// (**e.g.** grams)
    plural: Option<String>,
}

type GetAllUnitsResponse = Json<Vec<Unit>>;

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

type GetAllUnitConversionsResponse = Json<Vec<UnitConversion>>;

#[derive(ApiResponse)]
enum GetResponse<T: std::marker::Send + ToJSON> {
    #[oai(status = 200)]
    Success(Json<T>),
    #[oai(status = 404)]
    NotFound(PlainText<String>),
}

#[derive(ApiResponse)]
enum DeleteResponse {
    #[oai(status = 200)]
    Success(Json<i32>),
    #[oai(status = 404)]
    NotFound(PlainText<String>),
}

struct UkisApi;

#[OpenApi]
impl UkisApi {
    // PRODUCTS
    /// Products: Fetch all
    #[oai(path = "/products", method = "get")]
    async fn get_products(&self, pool: Data<&PgPool>) -> Result<GetAllProductsResponse> {
        let products = sqlx::query_as!(Product, "SELECT * FROM products")
            .fetch_all(pool.0)
            .await
            .unwrap();

        Ok(Json(products))
    }

    /// Products: Fetch by id
    #[oai(path = "/products/:id", method = "get")]
    async fn get_product(
        &self,
        pool: Data<&PgPool>,
        id: Path<i32>,
    ) -> Result<GetResponse<Product>> {
        let result: Option<Product> =
            sqlx::query_as!(Product, "SELECT * FROM products WHERE id = $1", id.0)
                .fetch_optional(pool.0)
                .await
                .map_err(InternalServerError)?;

        match result {
            Some(product) => Ok(GetResponse::Success(Json(product))),
            None => Ok(GetResponse::NotFound(PlainText(
                format!("No product with id '{}' found.", id.0).to_string(),
            ))),
        }
    }

    /// Products: Create new
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

    /// Products: Delete with id
    #[oai(path = "/products/:id", method = "delete")]
    async fn delete_product(&self, pool: Data<&PgPool>, id: Path<i32>) -> Result<DeleteResponse> {
        let result = sqlx::query!(
            r#"
DELETE FROM products
WHERE id = $1
RETURNING id"#,
            id.0
        )
        .fetch_optional(pool.0)
        .await
        .map_err(InternalServerError)?;

        match result {
            Some(_) => Ok(DeleteResponse::Success(Json(id.0))),
            None => Ok(DeleteResponse::NotFound(PlainText(
                format!("No product with id '{}' found.", id.0).to_string(),
            ))),
        }
    }

    // UNITS
    /// Units: Fetch all
    #[oai(path = "/units", method = "get")]
    async fn get_units(&self, pool: Data<&PgPool>) -> Result<GetAllUnitsResponse> {
        let units = sqlx::query_as!(Unit, "SELECT * FROM units")
            .fetch_all(pool.0)
            .await
            .unwrap();

        Ok(Json(units))
    }

    /// Units: Fetch by id
    #[oai(path = "/units/:id", method = "get")]
    async fn get_unit(&self, pool: Data<&PgPool>, id: Path<i32>) -> Result<GetResponse<Unit>> {
        let unit: Option<Unit> = sqlx::query_as!(Unit, "SELECT * FROM units WHERE id = $1", id.0)
            .fetch_optional(pool.0)
            .await
            .map_err(InternalServerError)?;

        match unit {
            Some(unit) => Ok(GetResponse::Success(Json(unit))),
            None => Ok(GetResponse::NotFound(PlainText(
                format!("No unit with id '{}' found.", id.0).to_string(),
            ))),
        }
    }

    /// Units: Create new
    #[oai(path = "/units", method = "post")]
    async fn new_unit(&self, pool: Data<&PgPool>, unit: Json<Unit>) -> Result<Json<i32>> {
        let record = sqlx::query!(
            r#"
INSERT INTO units (singular, plural)
VALUES ($1, $2)
RETURNING id"#,
            unit.singular,
            unit.plural,
        )
        .fetch_one(pool.0)
        .await
        .map_err(InternalServerError)?;

        Ok(Json(record.id))
    }

    /// Units: Delete with id
    #[oai(path = "/units/:id", method = "delete")]
    async fn delete_unit(&self, pool: Data<&PgPool>, id: Path<i32>) -> Result<DeleteResponse> {
        let result = sqlx::query!(
            r#"
DELETE FROM units
WHERE id = $1
RETURNING id"#,
            id.0
        )
        .fetch_optional(pool.0)
        .await
        .map_err(InternalServerError)?;

        match result {
            Some(_) => Ok(DeleteResponse::Success(Json(id.0))),
            None => Ok(DeleteResponse::NotFound(PlainText(
                format!("No unit with id '{}' found.", id.0).to_string(),
            ))),
        }
    }

    // PLACES
    /// Places: Fetch all
    #[oai(path = "/places", method = "get")]
    async fn get_places(&self, pool: Data<&PgPool>) -> Result<GetAllPlacesResponse> {
        let places = sqlx::query_as!(Place, "SELECT * FROM places")
            .fetch_all(pool.0)
            .await
            .map_err(InternalServerError)?;

        Ok(Json(places))
    }

    /// Places: Fetch by id
    #[oai(path = "/places/:id", method = "get")]
    async fn get_place(&self, pool: Data<&PgPool>, id: Path<i32>) -> Result<GetResponse<Place>> {
        let result: Option<Place> =
            sqlx::query_as!(Place, "SELECT * FROM places WHERE id = $1", id.0)
                .fetch_optional(pool.0)
                .await
                .map_err(InternalServerError)?;

        match result {
            Some(place) => Ok(GetResponse::Success(Json(place))),
            None => Ok(GetResponse::NotFound(PlainText(
                format!("No place with id '{}' found.", id.0).to_string(),
            ))),
        }
    }

    /// Places: Create new
    #[oai(path = "/place", method = "post")]
    async fn new_place(&self, pool: Data<&PgPool>, place: Json<Place>) -> Result<Json<i32>> {
        let record = sqlx::query!(
            r#"
INSERT INTO places (name, description)
VALUES ($1, $2)
RETURNING id"#,
            place.name,
            place.description,
        )
        .fetch_one(pool.0)
        .await
        .map_err(InternalServerError)?;

        Ok(Json(record.id))
    }

    /// Places: Delete with id
    #[oai(path = "/places/:id", method = "delete")]
    async fn delete_place(&self, pool: Data<&PgPool>, id: Path<i32>) -> Result<DeleteResponse> {
        let result = sqlx::query!(
            r#"
DELETE FROM places
WHERE id = $1
RETURNING id"#,
            id.0
        )
        .fetch_optional(pool.0)
        .await
        .map_err(InternalServerError)?;

        match result {
            Some(_) => Ok(DeleteResponse::Success(Json(id.0))),
            None => Ok(DeleteResponse::NotFound(PlainText(
                format!("No place with id '{}' found.", id.0).to_string(),
            ))),
        }
    }

    // SPACES
    /// Spaces: Fetch all
    #[oai(path = "/spaces", method = "get")]
    async fn get_spaces(&self, pool: Data<&PgPool>) -> Result<GetAllSpacesResponse> {
        let spaces = sqlx::query_as!(Space, "SELECT * FROM spaces")
            .fetch_all(pool.0)
            .await
            .map_err(InternalServerError)?;

        Ok(Json(spaces))
    }

    /// Spaces: Fetch by id
    #[oai(path = "/spaces/:id", method = "get")]
    async fn get_space(&self, pool: Data<&PgPool>, id: Path<i32>) -> Result<GetResponse<Space>> {
        let result: Option<Space> =
            sqlx::query_as!(Space, "SELECT * FROM spaces WHERE id = $1", id.0)
                .fetch_optional(pool.0)
                .await
                .map_err(InternalServerError)?;

        match result {
            Some(space) => Ok(GetResponse::Success(Json(space))),
            None => Ok(GetResponse::NotFound(PlainText(
                format!("No space with id '{}' found.", id.0).to_string(),
            ))),
        }
    }

    /// Spaces: Create new
    #[oai(path = "/space", method = "post")]
    async fn new_space(&self, pool: Data<&PgPool>, space: Json<Space>) -> Result<Json<i32>> {
        let record = sqlx::query!(
            r#"
INSERT INTO spaces (name, description)
VALUES ($1, $2)
RETURNING id"#,
            space.name,
            space.description,
        )
        .fetch_one(pool.0)
        .await
        .map_err(InternalServerError)?;

        Ok(Json(record.id))
    }

    /// Spaces: Delete with id
    #[oai(path = "/spaces/:id", method = "delete")]
    async fn delete_space(&self, pool: Data<&PgPool>, id: Path<i32>) -> Result<DeleteResponse> {
        let result = sqlx::query!(
            r#"
DELETE FROM places
WHERE id = $1
RETURNING id"#,
            id.0
        )
        .fetch_optional(pool.0)
        .await
        .map_err(InternalServerError)?;

        match result {
            Some(_) => Ok(DeleteResponse::Success(Json(id.0))),
            None => Ok(DeleteResponse::NotFound(PlainText(
                format!("No space with id '{}' found.", id.0).to_string(),
            ))),
        }
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
