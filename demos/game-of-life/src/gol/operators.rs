use tfhe::boolean::prelude::*;

// add one encrypted bit `a` to the encrypted binary representation `b` of a 3-bit number, with 8
// identified with 0
fn add_1(
  server_key: &ServerKey,
  a: &Vec<Ciphertext>,
  b: &(Vec<Ciphertext>, Vec<Ciphertext>, Vec<Ciphertext>),
) -> (Vec<Ciphertext>, Vec<Ciphertext>, Vec<Ciphertext>) {
  let c1 = server_key.xor_packed(a, &b.0);
  let r = server_key.and_packed(a, &b.0);
  let c2 = server_key.xor_packed(&r, &b.1);
  let r = server_key.and_packed(&r, &b.1);
  let c3 = server_key.xor_packed(&r, &b.2);
  (c1, c2, c3)
}

// sum the encrypted bits in `elements`, starting from an encrypted 3-bit representation of 0
fn sum(
  server_key: &ServerKey,
  elements: [Vec<Ciphertext>; 8],
  zeros: &(Vec<Ciphertext>, Vec<Ciphertext>, Vec<Ciphertext>),
) -> (Vec<Ciphertext>, Vec<Ciphertext>, Vec<Ciphertext>) {
  let mut result = add_1(server_key, &elements[0], zeros);
  for i in 1..8 {
    result = add_1(server_key, &elements[i], &result);
  }
  result
}

// check if a cell will be alive after the update
pub fn is_alive(
  server_key: &ServerKey,
  cell_p: Vec<Ciphertext>,
  neighbours_p: [Vec<Ciphertext>; 8],
  zeros: &(Vec<Ciphertext>, Vec<Ciphertext>, Vec<Ciphertext>),
  new_states: &mut Vec<Ciphertext>,
) {
  let sum_neighbours = sum(server_key, neighbours_p, zeros);
  let sum_is_2_or_3 =
    server_key.and_packed(&sum_neighbours.1, &server_key.not_packed(&sum_neighbours.2));
  let sum_is_3 = server_key.and_packed(
    &sum_neighbours.0,
    &server_key.and_packed(&sum_neighbours.1, &server_key.not_packed(&sum_neighbours.2)),
  );
  let mut new_state =
    server_key.or_packed(&sum_is_3, &server_key.and_packed(&cell_p, &sum_is_2_or_3));
  new_states.append(&mut new_state);
}
