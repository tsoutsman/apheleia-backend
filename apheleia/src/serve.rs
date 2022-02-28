use std::{future::Future, sync::Arc};

use crate::{
    db,
    extractor::UserConfig,
    graphql::{graphiql_route, graphql_route, playground_route},
    Error, FuncReturn, Result,
};

use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpServer,
};

/// Entry point for the server.
///
/// Takes in a single paramater, `token_to_id`, a function which converts an
/// unverified token into some verified ID. For example, an OAuth access token
/// into a user ID.
#[inline]
pub(crate) async fn serve<Func, Fut>(token_to_id_function: Func) -> Result<()>
where
    Func: Fn(String) -> Fut + 'static + Send + Sync + Clone,
    Fut: Future<Output = std::result::Result<String, Box<dyn std::error::Error>>> + 'static + Send,
{
    let wrapper = move |token| -> FuncReturn { Box::pin(token_to_id_function(token)) };
    let config = UserConfig {
        token_to_id_function: Arc::new(wrapper),
    };
    let db_pool = db::pool().await?;

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_pool.clone()))
            // TODO: CORS?
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/graphql")
                    .route(web::get().to(graphql_route))
                    .route(web::post().to(graphql_route))
                    .app_data(config.clone()),
            )
            .service(web::resource("/playground").route(web::get().to(playground_route)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql_route)))
    })
    .bind("0.0.0.0:8000")
    .map_err(|e| -> Error { e.into() })?;

    println!("Listening on: {:?}", server.addrs());

    server.run().await.map_err(|e| -> Error { e.into() })
}