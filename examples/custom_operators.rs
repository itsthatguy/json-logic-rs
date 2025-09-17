// Example: Using Dynamic Custom Operators with json-logic-rs
//
// This example demonstrates how to add custom operations at runtime
// that can be used in JsonLogic rules.

use jsonlogic_rs::{
    add_data_operation, add_lazy_operation, add_operation, apply, clear_operations,
    Error, NumParams,
};
use serde_json::{json, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Operations Demo ===\n");

    // 1. Add a simple custom operation
    println!("1. Adding a custom 'double' operation...");
    add_operation(
        "double",
        |args| {
            if let Some(Value::Number(n)) = args.first() {
                if let Some(num) = n.as_f64() {
                    return Ok(json!(num * 2.0));
                }
            }
            Ok(Value::Null)
        },
        NumParams::Exactly(1),
    );

    let result = apply(&json!({"double": [21]}), &json!({}))?;
    println!("  Rule: {{\"double\": [21]}}");
    println!("  Result: {}", result);
    println!("  Expected: 42\n");

    // 2. Add a custom lazy operation (controls evaluation)
    println!("2. Adding a custom 'conditional_log' lazy operation...");
    add_lazy_operation(
        "conditional_log",
        |data, args| {
            if args.len() >= 2 {
                let condition = &args[0];
                let message = &args[1];

                // Only evaluate condition if it's true
                if let Value::Bool(true) = condition {
                    if let Value::String(msg) = message {
                        println!("    LOG: {}", msg);
                    }
                    return Ok(json!(true));
                }
            }
            Ok(json!(false))
        },
        NumParams::Exactly(2),
    );

    let result = apply(
        &json!({"conditional_log": [true, "Hello from JsonLogic!"]}),
        &json!({}),
    )?;
    println!("  Rule: {{\"conditional_log\": [true, \"Hello from JsonLogic!\"]}}");
    println!("  Result: {}\n", result);

    // 3. Add a custom data operation (accesses context data)
    println!("3. Adding a custom 'upper' data operation...");
    add_data_operation(
        "upper",
        |data, args| {
            if let Some(Value::String(field_name)) = args.first() {
                if let Value::Object(obj) = data {
                    if let Some(Value::String(value)) = obj.get(field_name) {
                        return Ok(json!(value.to_uppercase()));
                    }
                }
            }
            Ok(Value::Null)
        },
        NumParams::Exactly(1),
    );

    let result = apply(
        &json!({"upper": ["name"]}),
        &json!({"name": "json-logic-rs"}),
    )?;
    println!("  Rule: {{\"upper\": [\"name\"]}}");
    println!("  Data: {{\"name\": \"json-logic-rs\"}}");
    println!("  Result: {}\n", result);

    // 4. Demonstrate operator precedence (custom overrides built-in)
    println!("4. Overriding built-in '==' operator...");
    println!("  Before override:");
    let result = apply(&json!({"==": [1, 1]}), &json!({}))?;
    println!("    Rule: {{\"==\": [1, 1]}} -> {}", result);

    add_operation(
        "==",
        |args| {
            // Custom equality that always returns "CUSTOM!"
            Ok(json!("CUSTOM!"))
        },
        NumParams::Exactly(2),
    );

    println!("  After adding custom override:");
    let result = apply(&json!({"==": [1, 1]}), &json!({}))?;
    println!("    Rule: {{\"==\": [1, 1]}} -> {}\n", result);

    // 5. Complex example: Custom math operations
    println!("5. Adding complex math operations...");

    // Power operation
    add_operation(
        "pow",
        |args| {
            if args.len() == 2 {
                if let (Some(Value::Number(base)), Some(Value::Number(exp))) =
                    (args.get(0), args.get(1))
                {
                    if let (Some(b), Some(e)) = (base.as_f64(), exp.as_f64()) {
                        return Ok(json!(b.powf(e)));
                    }
                }
            }
            Ok(Value::Null)
        },
        NumParams::Exactly(2),
    );

    // Square root operation
    add_operation(
        "sqrt",
        |args| {
            if let Some(Value::Number(n)) = args.first() {
                if let Some(num) = n.as_f64() {
                    if num >= 0.0 {
                        return Ok(json!(num.sqrt()));
                    }
                }
            }
            Ok(Value::Null)
        },
        NumParams::Exactly(1),
    );

    // Test complex expression: sqrt(pow(3, 2) + pow(4, 2))
    let result = apply(
        &json!({
            "sqrt": [{
                "+": [
                    {"pow": [3, 2]},
                    {"pow": [4, 2]}
                ]
            }]
        }),
        &json!({}),
    )?;

    println!("  Rule: {{\"sqrt\": [{{\"+ \": [{{\"pow\": [3, 2]}}, {{\"pow\": [4, 2]}}]}}]}}");
    println!("  Result: {} (should be 5.0)\n", result);

    // 6. Clean up
    println!("6. Cleaning up custom operations...");
    clear_operations();

    // Verify built-in operators work again
    let result = apply(&json!({"==": [1, 1]}), &json!({}))?;
    println!("  After cleanup, built-in '==' works again: {}", result);

    // 7. Demonstrate proper error handling
    println!("7. Demonstrating proper error handling...");

    // Add an operation that validates input and returns meaningful errors
    add_operation(
        "safe_divide",
        |args| match (args.get(0), args.get(1)) {
            (Some(Value::Number(a)), Some(Value::Number(b))) => {
                let dividend = a.as_f64().ok_or_else(|| Error::InvalidArgument {
                    value: (*a).clone().into(),
                    operation: "safe_divide".to_string(),
                    reason: "First argument must be a valid number".to_string(),
                })?;

                let divisor = b.as_f64().ok_or_else(|| Error::InvalidArgument {
                    value: (*b).clone().into(),
                    operation: "safe_divide".to_string(),
                    reason: "Second argument must be a valid number".to_string(),
                })?;

                if divisor == 0.0 {
                    return Err(Error::InvalidArgument {
                        value: json!(divisor),
                        operation: "safe_divide".to_string(),
                        reason: "Division by zero is not allowed".to_string(),
                    });
                }

                Ok(json!(dividend / divisor))
            }
            (Some(non_number), _) => Err(Error::InvalidArgument {
                value: (*non_number).clone(),
                operation: "safe_divide".to_string(),
                reason: "First argument must be a number".to_string(),
            }),
            (_, Some(non_number)) => Err(Error::InvalidArgument {
                value: (*non_number).clone(),
                operation: "safe_divide".to_string(),
                reason: "Second argument must be a number".to_string(),
            }),
            _ => Err(Error::InvalidArgument {
                value: Value::Null,
                operation: "safe_divide".to_string(),
                reason: "Missing required arguments".to_string(),
            }),
        },
        NumParams::Exactly(2),
    );

    // Test successful division
    let result = apply(&json!({"safe_divide": [10, 2]}), &json!({}))?;
    println!("  Rule: {{\"safe_divide\": [10, 2]}} -> {}", result);

    // Test error handling - division by zero
    match apply(&json!({"safe_divide": [10, 0]}), &json!({})) {
        Ok(_) => println!("  ERROR: Division by zero should have failed!"),
        Err(e) => println!("  Caught expected error: {}", e),
    }

    // Test error handling - invalid argument type
    match apply(&json!({"safe_divide": [10, "hello"]}), &json!({})) {
        Ok(_) => println!("  ERROR: Invalid argument should have failed!"),
        Err(e) => println!("  Caught expected error: {}", e),
    }

    // 8. Demonstrate parameter validation
    println!("8. Demonstrating parameter validation...");

    // Add an operation that requires exactly 2 arguments
    add_operation(
        "multiply",
        |args| {
            // We can assume args.len() == 2 because NumParams::Exactly(2) validates this
            if let (Some(Value::Number(a)), Some(Value::Number(b))) =
                (args.get(0), args.get(1))
            {
                if let (Some(x), Some(y)) = (a.as_f64(), b.as_f64()) {
                    return Ok(json!(x * y));
                }
            }
            Ok(Value::Null)
        },
        NumParams::Exactly(2),
    );

    // Test correct usage
    let result = apply(&json!({"multiply": [3, 4]}), &json!({}))?;
    println!("  Rule: {{\"multiply\": [3, 4]}} -> {}", result);

    // Test parameter validation - wrong argument count
    match apply(&json!({"multiply": [3]}), &json!({})) {
        Ok(_) => println!("  ERROR: Should have failed with wrong argument count!"),
        Err(Error::WrongArgumentCount { expected, actual }) => {
            println!(
                "  Caught parameter validation error: expected {:?}, got {}",
                expected, actual
            );
        }
        Err(e) => println!("  Caught unexpected error: {}", e),
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
