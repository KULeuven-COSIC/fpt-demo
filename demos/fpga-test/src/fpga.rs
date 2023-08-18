#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Supress not FFI-safe warning
#![allow(improper_ctypes)]

use tfhe::boolean::engine::BooleanEngine;
use tfhe::boolean::prelude::*;
use tfhe::boolean::PLAINTEXT_FALSE;
use tfhe::core_crypto::prelude::*;

use rand::{Rng, SeedableRng};

use std::iter::zip;
#[cfg(feature = "fpga")]
use {std::fs::File, std::io::BufReader, tfhe::boolean::server_key::FpgaGates};

fn main() {
  let mut boolean_engine = BooleanEngine::new();

  #[cfg(not(feature = "fpga"))]
  let client_key = boolean_engine.create_client_key(DEMO_PARAMETERS);

  #[cfg(feature = "fpga")]
  let client_key: ClientKey = {
    let file = File::open("testvectors/bootstrap/client_key.json").unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
  };

  // generate the server key, only the SW needs this
  let server_key = boolean_engine.create_server_key(&client_key);

  #[cfg(feature = "fpga")]
  server_key.enable_fpga();

  let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);

  for test in 0..100 {
    let mut golden_mouts = Vec::<bool>::new();
    let mut cts_tmp = Vec::<LweCiphertext<Vec<u32>>>::new();
    for _ in 0..FPGA_BOOTSTRAP_PACKING {
      let (m1, m2) = (rng.gen_range(0..=1) != 0, rng.gen_range(0..=1) != 0);
      let mout = m1 && m2;
      let ct_left = boolean_engine.encrypt(m1, &client_key);
      let ct_right = boolean_engine.encrypt(m2, &client_key);

      if let (Ciphertext::Encrypted(ct_left_ct), Ciphertext::Encrypted(ct_right_ct)) =
        (&ct_left, &ct_right)
      {
        let mut ct_tmp =
          LweCiphertext::new(0u32, ct_left_ct.lwe_size(), ct_left_ct.ciphertext_modulus());

        // compute the linear combination for AND: ct_left + ct_right + (0,...,0,-1/8)
        // ct_left + ct_right
        lwe_ciphertext_add(&mut ct_tmp, ct_left_ct, ct_right_ct);
        let cst = Plaintext(PLAINTEXT_FALSE);
        // - 1/8
        lwe_ciphertext_plaintext_add_assign(&mut ct_tmp, cst);

        cts_tmp.push(ct_tmp);
        golden_mouts.push(mout);
      }
    }

    // fpga
    let pbs_results = boolean_engine
      .bootstrapper
      .bootstrap_and_keyswitch_packed(&mut cts_tmp, &server_key);

    for (pbs_result, mout) in zip(pbs_results, golden_mouts) {
      let result = boolean_engine.decrypt(&pbs_result, &client_key);
      assert_eq!(result, mout);
    }

    println!("TEST {} PASSED", test);
  }
}
