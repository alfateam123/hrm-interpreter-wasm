extern crate cfg_if;
extern crate wasm_bindgen;
extern crate hrm_interpreter;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate console_error_panic_hook;

use std::panic;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use hrm_interpreter::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
#[derive(Serialize, Debug, Clone)]
pub struct InterpreterInterface {
    state: hrm_interpreter::state::InternalState,
    operations: Vec<Operation>,
    ended_with_error: bool,
    reason: Option<String>,
}

impl InterpreterInterface {
    fn new(_state: hrm_interpreter::state::InternalState, _ops: Vec<Operation>) -> Self {
        return InterpreterInterface {
            state: _state,
            operations: _ops,
            ended_with_error: false,
            reason: None,
        };
    }
}

#[wasm_bindgen]
impl InterpreterInterface {
    pub fn create(raw_json_code: &str, raw_input_settings: &str) -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let operations = json::read_instructions(String::from(raw_json_code));
        let state = json::read_config_from_string(String::from(raw_input_settings));

        InterpreterInterface::new(state, operations)
    }

    pub fn jsonify(&self) -> JsValue {
        JsValue::from(serde_json::to_string(&self).unwrap())
    }

    pub fn next(&mut self) -> JsValue {
        self._next();

        match self.reason {
            Some(_) => JsValue::from_str(self.reason.clone().unwrap().as_str()),
            None => JsValue::NULL
        }
    }

    fn _next(&mut self) -> Option<()> {
        if self.ended_with_error {
            None
        } else if self.state.executed_instructions() > 10000 {
            self.ended_with_error = true;
            self.reason = Some(String::from("instructions limit reached"));
            return None;
        } else if self.state.instruction_counter < self.operations.len() {
            let _operation = self.operations[self.state.instruction_counter];

            let result = self.state.apply(_operation);

            if result.is_err() {
                if let Operation::Inbox = _operation {
                    self.ended_with_error = false;
                    self.reason = result.err();
                    return None;
                } else {
                    self.ended_with_error = true;
                    self.reason = result.err();
                }
            }
            Some(())
        } else {
            self.reason = Some(String::from("reached end of program"));
            None
        }
    }
}

