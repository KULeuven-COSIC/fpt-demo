#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Supress not FFI-safe warning
#![allow(improper_ctypes)]
#![allow(dead_code)]

use super::{Bootstrapper, Ciphertext, ServerKey};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::boolean::prelude::*;
use crate::core_crypto::fft_impl::common::pbs_modulus_switch;
use crate::core_crypto::prelude::polynomial_algorithms::polynomial_wrapping_monic_monomial_mul_assign;
use crate::core_crypto::prelude::*;
use concrete_fft::c64;
use std::ffi::CString;
use std::os::raw::c_void;
use std::{env, fs, process};
use toml::Value;

impl Bootstrapper {
  pub fn enable_fpga(&mut self, bsk: &FourierLweBootstrapKeyOwned) {
    if self.is_fpga_enabled == false {
      unsafe {
        let env_var_FPGA_IMAGE = match env::var("FPGA_IMAGE") {
            Ok(val) => val,
            Err(_) => {
                eprintln!("The FPGA_IMAGE environment variable is not set.");
                process::exit(1);
            }
        };
        let env_var_FPGA_INDEX = match env::var("FPGA_INDEX") {
            Ok(val) => match val.parse::<u32>() {
                Ok(parsed_val) => parsed_val,
                Err(_) => {
                    // Handle the case when the value cannot be parsed as an integer
                    eprintln!("The value of FPGA_INDEX is not a valid integer.");
                    return;
                }
            },
            Err(_) => {
                eprintln!("The FPGA_INDEX environment variable is not set.");
                process::exit(1);
            }
        };

        self.fpga_device = xrtDeviceOpen(env_var_FPGA_INDEX);
        let file_path = CString::new(env_var_FPGA_IMAGE).unwrap();

        let xclbin = xrtXclbinAllocFilename(file_path.as_ptr());
        xrtDeviceLoadXclbinHandle(self.fpga_device, xclbin);

        // Get Kernel UUID
        let mut kernel_uuid = vec![0u8; 16];
        let kernel_uuid_ptr = kernel_uuid.as_mut_ptr();
        xrtDeviceGetXclbinUUID(self.fpga_device, kernel_uuid_ptr);

        // create Kernel objects
        let cptr_krnl = CString::new("accel").unwrap();
        let kernel: xrtKernelHandle =
          xrtPLKernelOpenExclusive(self.fpga_device, kernel_uuid_ptr, cptr_krnl.as_ptr());

        let krnl_grp_0 = xrtKernelArgGroupId(kernel, 0) as u32;
        let krnl_grp_1 = xrtKernelArgGroupId(kernel, 1) as u32;

        self.mem_lwe_in = xrtBOAlloc(self.fpga_device, MEM_LWE_IN_SIZE, 0u64, krnl_grp_0);
        self.mem_lwe_out = xrtBOAlloc(self.fpga_device, MEM_LWE_OUT_SIZE, 0u64, krnl_grp_1);
        let mem_bsk = xrtBOAlloc(self.fpga_device, BSK_SIZE, 0u64, krnl_grp_1);

        self.fpga_kernel = xrtRunOpen(kernel);
        xrtRunSetArg(self.fpga_kernel, 0, self.mem_lwe_in);
        xrtRunSetArg(self.fpga_kernel, 1, self.mem_lwe_out);
        xrtRunSetArg(self.fpga_kernel, 2, mem_bsk);
        xrtRunSetArg(self.fpga_kernel, 3, 1);

        let bsk_f: Vec<u64> = self.bsk_for_fpga(bsk);
        assert_eq!(
          bsk_f.len(),
          BSK_SIZE / std::mem::size_of::<u64>(),
          "bsk_mem size is incorrect"
        );

        xrtBOWrite(mem_bsk, bsk_f.as_ptr() as *const c_void, BSK_SIZE, 0);
        xrtBOSync(
          mem_bsk,
          xclBOSyncDirection_XCL_BO_SYNC_BO_TO_DEVICE,
          BSK_SIZE,
          0,
        );
      }
      self.is_fpga_enabled = true;
    }
  }

  pub fn disable_fpga(&mut self) {
    unsafe {
      // xrtKernelClose(self.fpga_kernel);
      xrtDeviceClose(self.fpga_device);
    }
    self.is_fpga_enabled = false;
  }

  fn bsk_for_fpga(&mut self, bsk: &FourierLweBootstrapKeyOwned) -> Vec<u64> {
    // let bsk_width = toml_parse("bootstrap","bsk_width") as u32;
    // let bsk_int_width = toml_parse("bootstrap","bsk_int_width") as u32;

    const BSK_COEFF_SEPARATION: usize = 64;
    let COEFF_PAIR_PER_TRANSFER: usize = BSK_AXI_WIDTH / (BSK_COEFF_SEPARATION * 2);

    fn c64_to_memval(value: f64) -> u64 {
      assert!(
        BSK_WIDTH < 64,
        "The float to integer conversion works for upto 64-bit"
      );
      let factor = 2_u64.pow((BSK_WIDTH - BSK_INT_WIDTH) as u32);
      let rounded_value = (value * factor as f64).round();
      (rounded_value as i64) as u64
    }

    let bsk_reordered = self.bsk_reorder(bsk);

    let mut bsk_mem_content: Vec<u64> = Vec::new();

    for gdim in 0..(DEMO_PARAMETERS.glwe_dimension.0 + 1) {
      for coeff in (0..FFT_STREAMING_SIZE).step_by(COEFF_PAIR_PER_TRANSFER) {
        for d in 0..BSK_DEPTH {
          for c in 0..COEFF_PAIR_PER_TRANSFER {
            let coeff_index = coeff + c;
            let complex = bsk_reordered[d][gdim][coeff_index];
            bsk_mem_content.push(c64_to_memval(complex.im));
            bsk_mem_content.push(c64_to_memval(complex.re));
          }
        }
      }
    }
    bsk_mem_content
  }

  // Reorders for Vec<Vec<Vec>>:
  //   The outer vector is for BROM addresses
  //   The middle vector is for glwe-dimension
  //   The inner vector is for polynomial coefficients
  // The bootstrapp hw reads a 2D vector from given BSK ROM address
  fn bsk_reorder(&mut self, bsk: &FourierLweBootstrapKeyOwned) -> Vec<Vec<Vec<c64>>> {
    let glwe_dimension = DEMO_PARAMETERS.glwe_dimension.0;
    let pbs_level = DEMO_PARAMETERS.pbs_level.0;
    let polynomial_size = DEMO_PARAMETERS.polynomial_size.0;

    bsk
      // Convert bsk to complex type: Vec<c64>
      .clone()
      .data()
      .iter()
      .cloned()
      .collect::<Vec<c64>>()
      // Get the data as polynomial chunks: Vec<Vec<c64>>
      .chunks(polynomial_size / 2)
      // The first element should be put at the last position
      // likely because of different twist twiddles
      .map(|v| {
        let mut x = v.to_vec();

        let first = x.first().cloned().unwrap();
        x.push(first);
        x.remove(0);
        x
      })
      .collect::<Vec<Vec<c64>>>()
      // Group by Columns: Vec<Vec<Vec<c64>>>
      .chunks(glwe_dimension + 1)
      .map(|v| v.to_owned())
      .collect::<Vec<Vec<Vec<c64>>>>()
      // Group by Rows: Vec<Vec<Vec<Vec<c64>>>>
      .chunks(glwe_dimension + 1)
      .map(|v| v.to_owned())
      .collect::<Vec<Vec<Vec<Vec<c64>>>>>()
      // Group by Levels: Vec<Vec<Vec<Vec<Vec<c64>>>>>
      .chunks(pbs_level)
      .map(|v| v.to_owned())
      .collect::<Vec<Vec<Vec<Vec<Vec<c64>>>>>>()
      // Reverse: Vec<Vec<Vec<Vec<Vec<c64>>>>>
      .into_iter()
      .map(|v| v.into_iter().rev().collect())
      .collect::<Vec<Vec<Vec<Vec<Vec<c64>>>>>>()
      // Transpose: Vec<Vec<Vec<Vec<Vec<c64>>>>>
      .into_iter()
      .map(|v| {
        let rows = v.len();
        let cols = v[0].len();
        (0..cols)
          .map(|col| (0..rows).map(|row| v[row][col].clone()).collect())
          .collect()
      })
      .collect::<Vec<Vec<Vec<Vec<Vec<c64>>>>>>()
      // Flatten twice: Vec<Vec<Vec<c64>>>
      .into_iter()
      .flatten()
      .into_iter()
      .flatten()
      .collect::<Vec<Vec<Vec<c64>>>>()
  }

  pub fn bootstrap_and_keyswitch_packed(
    &mut self,
    ciphertexts: &mut Vec<LweCiphertextOwned<u32>>,
    _server_key: &ServerKey,
  ) -> Vec<Ciphertext> {
    let bootstrapped: Vec<LweCiphertextOwned<u32>> = if !self.is_fpga_enabled {
      // let start = Instant::now();
      let result: Vec<LweCiphertextOwned<u32>> = ciphertexts
        .iter_mut()
        .map(|ct| self.bootstrap(ct, _server_key).unwrap())
        .collect();
      // println!(
      //     "{:?} SW bootstraps: {:?} ",
      //     ciphertexts.len(),
      //     start.elapsed()
      // );
      result
    } else {
      assert!(ciphertexts.len() <= FPGA_BOOTSTRAP_PACKING);

      fn alternate_extract_lwe_sample_from_glwe_ciphertext(
        input_glwe: &GlweCiphertext<Vec<u32>>,
        output_lwe: &mut LweCiphertext<Vec<u32>>,
        mut nth: usize,
      ) {
        // We retrieve the bodies and masks of the two ciphertexts.
        let (mut lwe_mask, lwe_body) = output_lwe.get_mut_mask_and_body();
        let (glwe_mask, glwe_body) = input_glwe.get_mask_and_body();

        nth = nth % (2 * input_glwe.polynomial_size().0);

        // We copy the body
        *lwe_body.data = if nth >= input_glwe.polynomial_size().0 {
          u32::MAX - glwe_body.as_ref()[nth % input_glwe.polynomial_size().0]
        } else {
          glwe_body.as_ref()[nth % input_glwe.polynomial_size().0]
        };

        // We copy the mask (each polynomial is in the wrong order)
        lwe_mask.as_mut().copy_from_slice(glwe_mask.as_ref());

        // We loop through the polynomials
        for lwe_mask_poly in lwe_mask.as_mut().chunks_mut(input_glwe.polynomial_size().0) {
          let mut poly: Polynomial<&mut [u32]> = Polynomial::from_container(lwe_mask_poly);
          // We reverse the polynomial
          poly.reverse();
          // We do the final monomial mul
          polynomial_wrapping_monic_monomial_mul_assign(
            &mut poly,
            MonomialDegree(nth + glwe_mask.polynomial_size().0 + 1),
          );
        }
      }

      let mut lwe_out =
        vec![
          [0u32; DEMO_PARAMETERS.polynomial_size.0 * (DEMO_PARAMETERS.glwe_dimension.0 + 1)];
          FPGA_BOOTSTRAP_PACKING
        ];

      let modulus_switch = |num: &u32| -> u32 {
        pbs_modulus_switch(
          *num,
          DEMO_PARAMETERS.polynomial_size,
          ModulusSwitchOffset(0),
          LutCountLog(0),
        ) as u32
      };

      // ! cts_mask is passed to the FPGA and needs to be contiguous memory
      let cts_mask: Vec<[u32; DEMO_PARAMETERS.lwe_dimension.0]> = ciphertexts
        .iter()
        .map(|ct| {
          ct.get_mask()
            .as_ref()
            .iter()
            .map(modulus_switch)
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap()
        })
        .collect();

      let ciphertext_moduluses: Vec<CiphertextModulus<u32>> = ciphertexts
        .iter()
        .map(|ct| ct.ciphertext_modulus())
        .collect();

      let cts_bodies: Vec<u32> = ciphertexts
        .iter()
        .map(|ct| modulus_switch(ct.get_body().data))
        .collect();

      unsafe {
        // Write Input
        xrtBOWrite(
          self.mem_lwe_in,
          cts_mask.as_ptr() as *const c_void,
          MEM_LWE_IN_SIZE,
          0,
        );
        xrtBOSync(
          self.mem_lwe_in,
          xclBOSyncDirection_XCL_BO_SYNC_BO_TO_DEVICE,
          MEM_LWE_IN_SIZE,
          0,
        );

        // Kernel Run
        // let start = Instant::now();
        xrtRunStart(self.fpga_kernel);
        xrtRunWait(self.fpga_kernel);
        // println!(
        //     "{:?} FPGA bootstraps: {:?} ",
        //     ciphertexts.len(),
        //     start.elapsed()
        // );

        // Read Output
        xrtBOSync(
          self.mem_lwe_out,
          xclBOSyncDirection_XCL_BO_SYNC_BO_FROM_DEVICE,
          MEM_LWE_OUT_SIZE,
          0,
        );

        xrtBORead(
          self.mem_lwe_out,
          lwe_out.as_mut_ptr() as *mut c_void,
          MEM_LWE_OUT_SIZE,
          0,
        );

        xrtRunSetArg(self.fpga_kernel, 3, 0);
      }

      let mut result = Vec::<LweCiphertextOwned<u32>>::new();

      for (slot, body, modulus) in itertools::izip!(lwe_out, cts_bodies, ciphertext_moduluses) {
        let mut pbs_result = LweCiphertext::new(
          0u32,
          LweDimension(DEMO_PARAMETERS.glwe_dimension.0 * DEMO_PARAMETERS.polynomial_size.0)
            .to_lwe_size(),
          modulus,
        );
        let mut input_glwe =
          GlweCiphertext::from_container(slot.to_vec(), DEMO_PARAMETERS.polynomial_size, modulus);
        alternate_extract_lwe_sample_from_glwe_ciphertext(
          &mut input_glwe,
          &mut pbs_result,
          body as usize,
        );
        result.push(pbs_result);
      }

      unsafe {
        result.set_len(ciphertexts.len());
      }
      result
    };

    #[cfg(feature = "without_keyswitch")]
    let result = bootstrapped
      .iter()
      .map(|ct| Ciphertext::Encrypted(ct.clone()))
      .collect();

    #[cfg(not(feature = "without_keyswitch"))]
    let result = {
      // let start = Instant::now();
      let result = bootstrapped
        .iter()
        .map(|ct| Ciphertext::Encrypted(self.keyswitch(ct, _server_key).unwrap()))
        .collect();
      // print!(
      //     "{:?} SW keyswitches: {:?}",
      //     ciphertexts.len(),
      //     start.elapsed()
      // );
      result
    };

    result
  }
}
