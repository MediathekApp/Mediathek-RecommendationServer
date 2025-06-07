// src/api/mod.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use actix_web::{web, HttpResponse, Responder, get, post};
use serde::{Deserialize, Serialize};

// Import the CoOccurrenceCounter from our algorithms module
use crate::algorithms::CoOccurrenceCounter;
use crate::algorithms::Counters;

// --- API Data Models for Co-Occurence ---

/// Struct for the POST /add_list request body
#[derive(Debug, Deserialize)]
pub struct AddListRequest {
    pub identifiers: Vec<String>,
}

/// Struct for the /metrics/{identifier} response
#[derive(Debug, Serialize)]
pub struct CoOccurrenceMetricsResponse { // Renamed for clarity
    pub target_identifier: String,
    pub co_occurrences: HashMap<String, u32>,
}

// --- API Data Models for Rotating Counters ---

#[derive(Debug, Deserialize)]
pub struct IncrementCounterRequest {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct DailyCountersResponse {
    // We can just serialize the entire Counters struct
    #[serde(flatten)] // Flatten to avoid a nested "counters" field in JSON
    pub counters: Counters,
}

// --- API Handlers (for Co-Occurence) ---

#[post("/lists")]
pub async fn add_list_handler(
    req_body: web::Json<AddListRequest>,
    counter_data: web::Data<Arc<Mutex<CoOccurrenceCounter>>>,
) -> impl Responder {
    let mut counter_lock = counter_data.lock().unwrap();
    counter_lock.process_list(&req_body.identifiers);
    HttpResponse::Ok().json(HashMap::from([("status", "success")]))
}

#[get("/lists/{identifier}")]
pub async fn get_co_occurrence_metrics_handler(
    path: web::Path<String>, // Captures the 'identifier' from the URL
    counter_data: web::Data<Arc<Mutex<CoOccurrenceCounter>>>,
) -> impl Responder {
    let identifier = path.into_inner(); // Extract the String from web::Path
    let counter_lock = counter_data.lock().unwrap();
    let co_occurrences = counter_lock.get_metrics_for_identifier(&identifier);

    let response = CoOccurrenceMetricsResponse {
        target_identifier: identifier,
        co_occurrences,
    };
    HttpResponse::Ok().json(response)
}

// --- API Handlers (for Rotating Counters) ---

#[post("/counters")]
pub async fn increment_daily_counter_handler(
    req_body: web::Json<IncrementCounterRequest>,
    rotating_counters_data: web::Data<Arc<Mutex<Counters>>>, 
) -> impl Responder {
    let mut counters_lock = rotating_counters_data.lock().unwrap();
    counters_lock.increment(&req_body.id);
    HttpResponse::Ok().json(HashMap::from([("status", "success")]))
}

#[get("/counters")]
pub async fn get_rotating_counters_handler(
    rotating_counters_data: web::Data<Arc<Mutex<Counters>>>,
) -> impl Responder {
    let counters_lock = rotating_counters_data.lock().unwrap();
    let counters = counters_lock.clone(); // Clone the data for the response

    let response = DailyCountersResponse { counters };
    HttpResponse::Ok().json(response)
}


// --- Route Configuration ---

/// Configures the routes for all API endpoints.
pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(add_list_handler)
       .service(get_co_occurrence_metrics_handler) 
       .service(increment_daily_counter_handler)  
       .service(get_rotating_counters_handler);     
}