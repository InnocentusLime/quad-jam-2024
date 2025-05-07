use std::{any::{type_name, TypeId}, hash::Hash};

use hashbrown::HashMap;
use macroquad::time::get_time;
use quad_dbg::dump;


const OLD_WEIGHT: f32 = 0.9;

struct ProfilerEntry {
    name: &'static str,
    avg: f32,
    max: f32,
    hits: u32,
}

impl ProfilerEntry {
    fn update(&mut self, new_time: f32) {
        self.avg = self.avg * OLD_WEIGHT + new_time * (1.0 - OLD_WEIGHT);
    
        if new_time > self.max {
            self.max = new_time;
            self.hits += 1;
            return;
        } 

        if new_time < self.avg {
            return;
        }
        
        if (new_time - self.avg) - (self.max - new_time) >= 0.001 {
            self.hits += 1;
        }
    }
}

pub struct SysProfiler {
    storage: HashMap<TypeId, ProfilerEntry>,
}

impl SysProfiler {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn log(&self) {
        for entry in self.storage.values() {
            dump!("{}: {} {:.3} {:.3}", entry.hits, entry.name, entry.avg * 1000.0, entry.max * 1000.0);
        }
    }

    pub fn run<T: 'static, F: FnOnce()>(&mut self, f: F, sample: T) {
        let entry = self.storage.entry(
            TypeId::of::<T>()
        ).or_insert_with(|| ProfilerEntry {
            name: type_name::<T>(),
            hits: 0,
            avg: 0.0,
            max: 0.0,   
        });

        let now = get_time();
        f();
        let after = get_time();

        entry.update((after - now) as f32);
    }
}