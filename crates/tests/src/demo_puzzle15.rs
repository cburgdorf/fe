#![cfg(feature = "solc-backend")]

use fe_compiler_test_utils::*;

#[test]
fn puzzle15() {
    with_executor(&|mut executor| {

        let initial_state = uint_array_token(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 0, 15]);

        let mut harness = deploy_contract(&mut executor, "demos/puzzle15.fe", "Game", &[initial_state.clone()]);

        let sender = address_token("1234000000000000000000000000000000005678");

        harness.test_function(&mut executor, "get_board", &[], Some(&initial_state.clone()));

        harness.test_function(&mut executor, "is_winning_state", &[], Some(&bool_token(false)));

        // Make the winning move
        harness.test_function(&mut executor, "move_field", &[uint_token(15)], None);

        harness.test_function(&mut executor, "is_winning_state", &[], Some(&bool_token(true)));

        //harness.caller = sender.clone().into_address().unwrap();
    })
}
