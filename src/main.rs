// src/main.rs
use std::sync::{Arc, Mutex};
use actix_web::{web, App, HttpServer};

// Declare the modules
mod algorithms;
mod api;

// Import our custom modules
use crate::algorithms::{CoOccurrenceCounter, Counters, run_daily_counter_rotation, perform_final_persistence};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize both counter types
    let co_occurrence_counter_arc = Arc::new(Mutex::new(CoOccurrenceCounter::new()));
    let rotating_counters_arc = Arc::new(Mutex::new(Counters::new()));
    let rotating_counters_for_http_server_setup = Arc::clone(&rotating_counters_arc);

    // Start the background task for rotating counter rotation and persistence
    // This task will run concurrently with the HTTP server.
    let rotating_counters_for_task = Arc::clone(&rotating_counters_arc); // Clone for the spawned task
    tokio::task::spawn(async move {
        run_daily_counter_rotation(rotating_counters_for_task).await;
    });

    println!("Server running on http://127.0.0.1:3030");

    let server_result = HttpServer::new(move || {
        App::new()
            // Register co_occurrence_counter as app data
            .app_data(web::Data::new(co_occurrence_counter_arc.clone()))
            // Register rotating_counters as app data (distinct type from co_occurrence_counter_arc)
            .app_data(web::Data::new(rotating_counters_for_http_server_setup.clone()))
            // Configure all routes from the api module
            .configure(api::config_routes)
    })
    .bind(("127.0.0.1", 3030))?
    .run()
    .await;

    // --- GRACEFUL SHUTDOWN PERSISTENCE ---
    // The original `rotating_counters_arc` is still available here,
    // and can be directly passed to the final persistence function.
    perform_final_persistence(rotating_counters_arc).await;

    server_result // Return the result of the server run

}