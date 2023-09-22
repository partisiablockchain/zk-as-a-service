//! Test for ZK-computation for secret voting.

mod zk_compute;

#[cfg(test)]
mod tests {
    use pbc_zk::api::*;
    use pbc_zk::*;

    use crate::zk_compute::zk_compute;

    #[test]
    fn zk_compute_zero_one() {
        // assert eval(0,0,1,1,0,0) => 2
        let zero: Sbi1 = Sbi8::from(0i8) == Sbi8::from(1i8);
        let one: Sbi1 = Sbi8::from(0i8) == Sbi8::from(0i8);
        let inputs: Vec<SecretVar> = vec![
            SecretVar {
                metadata: Box::new(1),
                value: Box::new(zero),
            },
            SecretVar {
                metadata: Box::new(2),
                value: Box::new(zero),
            },
            SecretVar {
                metadata: Box::new(3),
                value: Box::new(one),
            },
            SecretVar {
                metadata: Box::new(4),
                value: Box::new(one),
            },
            SecretVar {
                metadata: Box::new(5),
                value: Box::new(zero),
            },
            SecretVar {
                metadata: Box::new(6),
                value: Box::new(zero),
            },
        ];

        unsafe {
            set_secrets(inputs);
        }
        let output = zk_compute();
        assert_eq!(output, Sbi32::from(2));
    }
}
