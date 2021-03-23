//! Tests for contracts that should cause compile errors

#![cfg(feature = "solc-backend")]

use rstest::rstest;
use std::fs;

#[rstest(
    fixture_file,
    expected_error,
    case("break_without_loop_2.fe", "BreakWithoutLoop"),
    case("break_without_loop.fe", "BreakWithoutLoop"),
    case("call_event_with_wrong_types.fe", "TypeError"),
    case("call_undefined_function_on_contract.fe", "UndefinedValue"),
    case("call_undefined_function_on_external_contract.fe", "UndefinedValue"),
    case("call_undefined_function_on_memory_struct.fe", "UndefinedValue"),
    case("call_undefined_function_on_storage_struct.fe", "UndefinedValue"),
    case("continue_without_loop_2.fe", "ContinueWithoutLoop"),
    case("continue_without_loop.fe", "ContinueWithoutLoop"),
    case("duplicate_contract_in_module.fe", "AlreadyDefined"),
    case("duplicate_event_in_contract.fe", "AlreadyDefined"),
    case("duplicate_field_in_contract.fe", "AlreadyDefined"),
    case("duplicate_field_in_struct.fe", "AlreadyDefined"),
    case("duplicate_method_in_contract.fe", "AlreadyDefined"),
    case("duplicate_struct_in_module.fe", "AlreadyDefined"),
    case("duplicate_typedef_in_module.fe", "AlreadyDefined"),
    case("duplicate_var_in_child_scope.fe", "AlreadyDefined"),
    case("duplicate_var_in_contract_method.fe", "AlreadyDefined"),
    case("emit_undefined_event.fe", "MissingEventDefinition"),
    case("external_call_type_error.fe", "TypeError"),
    case("external_call_wrong_number_of_params.fe", "WrongNumberOfParams"),
    case("indexed_event.fe", "MoreThanThreeIndexedParams"),
    case("keccak_called_with_wrong_type.fe", "TypeError"),
    case("mismatch_return_type.fe", "TypeError"),
    case("missing_return_in_else.fe", "MissingReturn"),
    case("missing_return.fe", "MissingReturn"),
    case("needs_mem_copy.fe", "CannotMove"),
    case("non_bool_and.fe", "TypeError"),
    case("non_bool_or.fe", "TypeError"),
    case("not_in_scope_2.fe", "UndefinedValue"),
    case("not_in_scope.fe", "UndefinedValue"),
    case("numeric_capacity_mismatch/i128_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i128_pos.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i16_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i16_pos.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i256_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i256_pos.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i32_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i32_pos.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i64_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i64_pos.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i8_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/i8_pos.fe", "NumericCapacityMismatch"),
    case(
        "numeric_capacity_mismatch/literal_too_big.fe",
        "NumericCapacityMismatch"
    ),
    case(
        "numeric_capacity_mismatch/literal_too_small.fe",
        "NumericCapacityMismatch"
    ),
    case("numeric_capacity_mismatch/u128_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u128_pos.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u16_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u16_pos.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u256_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u256_pos.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u32_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u32_pos.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u64_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u64_pos.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u8_neg.fe", "NumericCapacityMismatch"),
    case("numeric_capacity_mismatch/u8_pos.fe", "NumericCapacityMismatch"),
    case("pow_with_signed_exponent.fe", "SignedExponentNotAllowed"),
    case("pow_with_wrong_capacity.fe", "TypeError"),
    case("return_addition_with_mixed_types.fe", "TypeError"),
    case("return_call_to_fn_with_param_type_mismatch.fe", "TypeError"),
    case("return_call_to_fn_without_return.fe", "TypeError"),
    case("return_from_init.fe", "TypeError"),
    case("return_lt_mixed_types.fe", "TypeError"),
    case("strict_boolean_if_else.fe", "TypeError"),
    case("string_capacity_mismatch.fe", "StringCapacityMismatch"),
    case("struct_call_without_kw_args.fe", "KeyWordArgsRequired"),
    case("type_constructor_from_variable.fe", "NumericLiteralExpected"),
    case("unary_minus_on_bool.fe", "TypeError"),
    case("unexpected_return.fe", "TypeError")
)]
fn test_compile_errors(fixture_file: &str, expected_error: &str) {
    let src = fs::read_to_string(format!("tests/fixtures/compile_errors/{}", fixture_file))
        .expect("Unable to read fixture file");

    match fe_compiler::compile(&src, true, false) {
        Err(compile_error) => assert!(
            format!("{}", compile_error).contains(expected_error),
            "{} did not contain {}",
            compile_error,
            expected_error
        ),
        _ => panic!(
            "Compiling succeeded when it was expected to fail with: {}",
            expected_error
        ),
    }
}
