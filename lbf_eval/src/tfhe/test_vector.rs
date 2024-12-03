/// Boolean function test vector type. A test vector has 2 parts: up to plaintext modulus and the negacyclic part.
///     * `Half` - second part is the negation of the first part
///     * `Zero` - second part is all zeros and equal to the first part
///     * `One` - second part is all ones and equal to the first part
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TestVectorType {
    Half, // [0, 1, 0, 0 | 1, 0, 1, 1]
    Zero, // [0, 0, 0, 1 | 0, 0, 0]
    One,  // [1, 1, 0, 0 | 1, 1]
}

/// Test vector is evaluated as `FBS(in, tv - delta) + delta`.
/// `delta` is either 0 (TestVectorType::Zero), 1/2 (TestVectorType::Half) or 1 (TestVectorType::One).
#[derive(Debug)]
pub struct TestVector {
    pub val: Vec<bool>,
    pub tv_type: TestVectorType,
}

impl TestVector {
    pub fn new(mut val: Vec<bool>, pt_mod: usize) -> Result<Self, String> {
        if val.len() > 2 * pt_mod {
            return Err("Test vector length is too large".to_string());
        }

        fn find_tv_type(a: bool, b: bool) -> TestVectorType {
            if a != b {
                TestVectorType::Half
            } else if (a == b) & a {
                TestVectorType::One
            } else {
                TestVectorType::Zero
            }
        }

        let mut tv_type = TestVectorType::Zero;
        for i in pt_mod..val.len() {
            let (a, b) = (val[i], val[i - pt_mod]);
            if i == pt_mod {
                tv_type = find_tv_type(a, b);
            } else {
                // check other respect first element type
                let tv_type_new = find_tv_type(a, b);
                if tv_type_new != tv_type {
                    return Err(format!(
                        "Invalid test vector element types: {:?} vs {:?} at index {}",
                        tv_type_new, tv_type, i
                    ));
                }
            }
        }

        // add zeros to the end if test vector size is smaller than plaintext space size
        if val.len() < pt_mod {
            val.resize(pt_mod, false);
        }

        Ok(Self { val, tv_type })
    }

    pub fn test_vec_fnc(&self, idx: u64) -> u64 {
        self.val[idx as usize] as u64
    }
}
