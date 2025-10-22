use pinocchio::entrypoint;

pub mod state;
pub mod instructions;
pub mod error;
pub mod system_program;

use instructions::process_instruction;

entrypoint!(process_instruction);

// Re-export for testing with solana-program-test
#[cfg(feature = "test-sbf")]
pub use instructions::process_instruction;
