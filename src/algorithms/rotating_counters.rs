// src/algorithms/rotating_counters.rs
use std::sync::{Arc, Mutex}; // Ensure these are imported at the top of this file
use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::{Local, Timelike, Datelike};
use actix_web::{web};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Counters {
    pub this_hour: HashMap<String, u32>,
    pub last_hour: HashMap<String, u32>,
    pub hour_minus_2: HashMap<String, u32>,
    pub today: HashMap<String, u32>,
    pub yesterday: HashMap<String, u32>,
    pub day_minus_2: HashMap<String, u32>,
    pub day_minus_3: HashMap<String, u32>,
    pub day_minus_4: HashMap<String, u32>,
    pub day_minus_5: HashMap<String, u32>,
    pub day_minus_6: HashMap<String, u32>,
    pub day_minus_7: HashMap<String, u32>,
    pub day_minus_8: HashMap<String, u32>,
    pub day_minus_9: HashMap<String, u32>,
    pub day_minus_10: HashMap<String, u32>,
    pub day_minus_11: HashMap<String, u32>,
    pub day_minus_12: HashMap<String, u32>,

    #[serde(skip)]
    pub dirty: bool,
}

impl Counters {
    pub fn new() -> Self {
        if let Ok(data) = fs::read_to_string("rotating_counters.json") {
            if let Ok(c) = serde_json::from_str(&data) {
                println!("Loaded rotating counters from rotating_counters.json");
                return c;
            }
        }
        println!("Initialized new rotating counters.");
        Counters::default()
    }

    pub fn persist(&self) {
        if self.dirty {
            if let Ok(data) = serde_json::to_string(&self) {
                let _ = fs::write("rotating_counters.json", data);
                println!("Rotating counters persisted.");
            } else {
                eprintln!("Failed to serialize rotating counters for persistence.");
            }
        }
    }

    pub fn rotate_hour(&mut self) {
        if !self.this_hour.is_empty() { // Only rotate if there was activity
            self.hour_minus_2 = self.last_hour.clone();
            self.last_hour = self.this_hour.clone();
            self.this_hour.clear();
            self.dirty = true;
            println!("Hourly counters rotated.");
        }
    }

    pub fn rotate_day(&mut self) {
        if !self.today.is_empty() { // Only rotate if there was activity
            self.day_minus_12 = self.day_minus_11.clone();
            self.day_minus_11 = self.day_minus_10.clone();
            self.day_minus_10 = self.day_minus_9.clone();
            self.day_minus_9 = self.day_minus_8.clone();
            self.day_minus_8 = self.day_minus_7.clone();
            self.day_minus_7 = self.day_minus_6.clone();
            self.day_minus_6 = self.day_minus_5.clone();
            self.day_minus_5 = self.day_minus_4.clone();
            self.day_minus_4 = self.day_minus_3.clone();
            self.day_minus_3 = self.day_minus_2.clone();
            self.day_minus_2 = self.yesterday.clone();
            self.yesterday = self.today.clone();
            self.today.clear();
            self.dirty = true;
            println!("Rotating counters rotated.");
        }
    }

    pub fn increment(&mut self, id: &str) {
        *self.this_hour.entry(id.to_string()).or_insert(0) += 1;
        *self.today.entry(id.to_string()).or_insert(0) += 1;
        self.dirty = true;
    }
}

// Function to handle the periodic rotation and persistence of rotating counters
pub async fn run_daily_counter_rotation(counters: std::sync::Arc<std::sync::Mutex<Counters>>) {
    let mut last_hour = Local::now().hour();
    let mut last_day = Local::now().day();
    let mut minutes_since_persist = 0;

    println!("Rotating counter thread started.");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        minutes_since_persist += 1;
        if minutes_since_persist < 60 {
            continue;
        }
        minutes_since_persist = 0;

        let now = Local::now();
        let current_counters_arc = counters.clone();

        // The result of web::block is Result<T, BlockingError>, where T is what your closure returns.
        // In our case, the closure returns Result<(u32, u32), ()>, so T is Result<(u32, u32), ()>.
        let result = web::block(move || {
            let mut c = current_counters_arc.lock().unwrap();
            let mut rotated = false;

            if now.hour() != last_hour {
                c.rotate_hour();
                rotated = true;
            }

            if now.day() != last_day {
                c.rotate_day();
                rotated = true;
            }

            if c.dirty || rotated {
                c.persist();
                c.dirty = false;
            }
            Ok::<_, ()>((now.hour(), now.day())) // Inner Result: Ok(hour, day) or Err(())
        }).await; // Outer Result: Ok(InnerResult) or Err(BlockingError)

        match result {
            // First, match the outer Result: if the blocking task itself completed successfully
            Ok(inner_result) => {
                // Then, match the inner Result: if the operation *inside* the blocking task was successful
                match inner_result {
                    Ok((new_hour, new_day)) => {
                        last_hour = new_hour;
                        last_day = new_day;
                    }
                    Err(()) => {
                        // This case handles the `Err(())` from our closure.
                        // In our current closure, it's unreachable as we always return `Ok`.
                        // But it's good practice to acknowledge the possibility.
                        eprintln!("Error within rotating counter rotation logic (inner Err).");
                    }
                }
            }
            // If the web::block task itself failed (e.g., cancelled or panicking in the spawned thread)
            Err(e) => {
                eprintln!("Error in rotating counter rotation block (outer BlockingError): {:?}", e);
            }
        }
    }
}

pub async fn perform_final_persistence(counters_arc: Arc<Mutex<Counters>>) {
    println!("Server shutting down. Attempting final persistence for rotating counters...");

    // Use web::block to run the potentially blocking persistence operation
    // This is crucial to avoid blocking the main Tokio runtime thread during shutdown.
    let persist_result = web::block(move || {
        if let Ok(mut counters_lock) = counters_arc.lock() {
            if counters_lock.dirty { // Only persist if there are pending changes
                println!("Performing final persist for rotating counters...");
                counters_lock.persist();
                counters_lock.dirty = false; // Reset dirty flag after final persist
            } else {
                println!("No pending changes for rotating counters to persist on shutdown.");
            }
        } else {
            eprintln!("Failed to acquire rotating counters lock for final persistence on shutdown.");
        }
        Ok::<(), ()>(()) // web::block expects a Result
    })
    .await;

    if let Err(e) = persist_result {
        eprintln!("Error during final rotating counters persistence block: {:?}", e);
    } else {
        println!("Final rotating counters persistence attempt completed.");
    }
}
