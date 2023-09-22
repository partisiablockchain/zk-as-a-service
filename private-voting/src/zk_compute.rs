use pbc_zk::*;

/// Perform a zk computation on secret-shared data to count the number
/// of accepting votes (non-zero).
///
/// ### Returns:
///
/// The number of accepting votes.
pub fn zk_compute() -> Sbi32 {
    // Initialize votes
    let mut votes_for: Sbi32 = Sbi32::from(0);
    let one: Sbi1 = Sbi8::from(0i8) == Sbi8::from(0i8);
    // Count votes
    for variable_id in secret_variable_ids() {
        if load_sbi::<Sbi1>(variable_id) == one {
            votes_for = votes_for + Sbi32::from(1);
        }
    }
    votes_for
}
