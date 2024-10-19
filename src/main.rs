use form_builder::{FormBuilder, ValidationMethods, Validator};

fn validate_custom(value: &str) -> bool {
    value.len() > 5
}

fn main() -> Result<(), String> {
    let mut form = FormBuilder::new()
        .add_field::<String>(
            "name",
            "Enter name:",
            Validator::new(vec![
                (ValidationMethods::not_empty, Some("Name cannot be empty")),
                (
                    ValidationMethods::validate_name,
                    Some("Name cannot contain numbers"),
                ),
            ]),
        )
        .add_field::<String>(
            "email",
            "Enter email:",
            Validator::new(vec![
                (ValidationMethods::not_empty, Some("Email cannot be empty")),
                (
                    ValidationMethods::validate_email,
                    Some("Invalid email format"),
                ),
            ]),
        )
        .add_field::<u32>(
            "age",
            "Enter age:",
            Validator::new(vec![(
                ValidationMethods::not_empty,
                Some("Age cannot be empty"),
            )]),
        )
        .add_field::<String>(
            "custom",
            "Enter custom value:",
            Validator::new(vec![
                (
                    ValidationMethods::not_empty,
                    Some("Custom value cannot be empty"),
                ),
                (
                    validate_custom,
                    Some("Custom value must be longer than 5 characters"),
                ),
            ]),
        )
        .build();

    form.fill()?;

    let name: String = form.get_value("name")?;
    let email: String = form.get_value("email")?;
    let age: u32 = form.get_value("age")?;
    let custom: String = form.get_value("custom")?;

    println!(
        "Name: {:?}, Email: {:?}, Age: {:?}, Custom: {:?}",
        name, email, age, custom
    );

    Ok(())
}
