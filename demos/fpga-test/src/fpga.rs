#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Supress not FFI-safe warning
#![allow(improper_ctypes)]

use tfhe::boolean::engine::BooleanEngine;
use tfhe::boolean::prelude::*;

use rand::{Rng, SeedableRng};

use std::iter::zip;
#[cfg(feature = "fpga")]
use tfhe::boolean::server_key::FpgaGates;

fn main() {
  const TEST_COUNT: u32 = 10;

  let mut boolean_engine = BooleanEngine::new();

  let client_key = boolean_engine.create_client_key(DEMO_PARAMETERS);

  // generate the server key, only the SW needs this
  let server_key = boolean_engine.create_server_key(&client_key);

  #[cfg(feature = "fpga")]
  server_key.enable_fpga();

  let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);

  for test in 0..TEST_COUNT {
    let mut lefts = Vec::<bool>::new();
    let mut rights = Vec::<bool>::new();
    let mut outputs = Vec::<bool>::new();

    let mut lefts_ct = Vec::<Ciphertext>::new();
    let mut rights_ct = Vec::<Ciphertext>::new();

    for _ in 0..FPGA_BOOTSTRAP_PACKING {
      let l = rng.gen_range(0..=1) != 0;
      let r = rng.gen_range(0..=1) != 0;

      let l_ct = boolean_engine.encrypt(l, &client_key);
      let r_ct = boolean_engine.encrypt(r, &client_key);

      lefts.push(l);
      rights.push(r);
      outputs.push(l && r);

      lefts_ct.push(l_ct);
      rights_ct.push(r_ct);
    }

    let outputs_ct = server_key.and_packed(&lefts_ct, &rights_ct);

    let mut success = true;
    for (output, ct_output) in zip(outputs, outputs_ct) {
      let expected: bool = output;
      let calculated: bool = client_key.decrypt(&ct_output);

      if expected != calculated {
        success = false;
      }
    }

    if success {
      println!("TEST {} PASSED", test);
    } else {
      println!("TEST {} FAILED", test);
    }
  }

}
