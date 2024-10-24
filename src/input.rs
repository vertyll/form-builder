use crate::validation::Validator;
use libc::{tcgetattr, tcsetattr, ECHO, ICANON, TCSANOW};
use std::fmt::Debug;
use std::io::{self, stdin, Read, Write};
use std::os::unix::io::AsRawFd;
use std::str::FromStr;

/// Reads input from the user and validates it using the provided validator.
///
/// # Arguments
///
/// * `prompt` - The prompt message to be displayed to the user.
/// * `validator` - An optional `Validator` instance to validate the input.
///
/// # Type Parameters
///
/// * `T` - The type of the input value. It must implement the `FromStr` and `Debug` traits.
///
/// # Returns
///
/// * `Ok(T)` if the input is successfully read and validated.
/// * `Err(String)` if there is an error reading or validating the input.
pub fn read_input<T>(prompt: &str, validator: Option<&Validator>) -> Result<T, String>
where
    T: FromStr,
    T::Err: Debug,
{
    use std::io::{self, Write};

    loop {
        print!("{} ", prompt);
        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to flush stdout: {:?}", e))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read line: {:?}", e))?;
        let input = input.trim();

        if let Some(validator) = validator {
            if let Err(err) = validator.validate(input) {
                println!("{}", err);
                continue;
            }
        }

        return input.parse::<T>().map_err(|err| format!("{:?}", err));
    }
}

/// Reads a selection from the user from a list of options.
///
/// # Arguments
///
/// * `prompt` - The prompt message to be displayed to the user.
/// * `options` - A list of options available for selection.
///
/// # Type Parameters
///
/// * `T` - The type of the options. It must implement the `Clone`, `PartialEq`, `Debug`, and `FromStr` traits.
///
/// # Returns
///
/// * `Ok(T)` if the selection is successfully read.
/// * `Err(String)` if there is an error reading the selection.
pub fn read_select<T>(prompt: &str, options: &[(T, String)]) -> Result<T, String>
where
    T: Clone + PartialEq + Debug + FromStr,
    T::Err: Debug,
{
    let mut selected = 0;

    loop {
        clear_screen();
        println!("{}:", prompt);

        for (i, (_, value)) in options.iter().enumerate() {
            if i == selected {
                println!("> {}", value);
            } else {
                println!("  {}", value);
            }
        }
        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to flush stdout: {:?}", e))?;

        match read_key_raw()? {
            Key::Up => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            Key::Down => {
                if selected < options.len() - 1 {
                    selected += 1;
                }
            }
            Key::Enter => {
                clear_screen();
                return Ok(options[selected].0.clone());
            }
            _ => {}
        }
    }
}

/// Reads multiple selections from the user from a list of options.
///
/// # Arguments
///
/// * `prompt` - The prompt message to be displayed to the user.
/// * `options` - A list of options available for selection.
/// * `limit` - An optional limit on the number of selections.
///
/// # Type Parameters
///
/// * `T` - The type of the options. It must implement the `Clone`, `PartialEq`, `Debug`, and `FromStr` traits.
///
/// # Returns
///
/// * `Ok(Vec<T>)` if the selections are successfully read.
/// * `Err(String)` if there is an error reading the selections.
pub fn read_multiselect<T>(
    prompt: &str,
    options: &[(T, String)],
    limit: Option<usize>,
) -> Result<Vec<T>, String>
where
    T: Clone + PartialEq + Debug + FromStr,
    T::Err: Debug,
{
    let mut selected = 0;
    let mut selected_options = vec![false; options.len()];

    loop {
        clear_screen();
        println!("{}:", prompt);
        println!("Use Space to select/deselect, Enter to confirm");

        for (i, (_, value)) in options.iter().enumerate() {
            let marker = if selected_options[i] { "*" } else { " " };
            if i == selected {
                println!("> [{}] {}", marker, value);
            } else {
                println!("  [{}] {}", marker, value);
            }
        }
        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to flush stdout: {:?}", e))?;

        match read_key_raw()? {
            Key::Up => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            Key::Down => {
                if selected < options.len() - 1 {
                    selected += 1;
                }
            }
            Key::Space => {
                if selected_options[selected] {
                    selected_options[selected] = false;
                } else if limit.is_none()
                    || selected_options.iter().filter(|&&x| x).count() < limit.unwrap()
                {
                    selected_options[selected] = true;
                }
            }
            Key::Enter => {
                let selected_keys: Vec<T> = options
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| selected_options[*i])
                    .map(|(_, (key, _))| key.clone())
                    .collect();

                if !selected_keys.is_empty() {
                    clear_screen();
                    return Ok(selected_keys);
                }
            }
            _ => {}
        }
    }
}

/// An enum representing different key presses.
pub enum Key {
    /// The up arrow key.
    Up,
    /// The down arrow key.
    Down,
    /// The enter key.
    Enter,
    /// The space key.
    Space,
    /// Any other key.
    Other,
}

/// Clears the terminal screen.
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    if io::stdout().flush().is_err() {
        eprintln!("Failed to flush stdout");
    }
}

/// Reads a raw key press from the user.
///
/// # Returns
///
/// * `Ok(Key)` if the key press is successfully read.
/// * `Err(String)` if there is an error reading the key press.
pub fn read_key_raw() -> Result<Key, String> {
    let stdin_fd = stdin().as_raw_fd();
    let mut termios = unsafe { std::mem::zeroed() };
    if unsafe { tcgetattr(stdin_fd, &mut termios) } < 0 {
        return Err("Failed to get terminal attributes".to_string());
    }

    termios.c_lflag &= !(ICANON | ECHO);
    if unsafe { tcsetattr(stdin_fd, TCSANOW, &termios) } < 0 {
        return Err("Failed to set terminal attributes".to_string());
    }

    let mut buffer = [0; 1];
    if io::stdin().read_exact(&mut buffer).is_err() {
        return Err("Failed to read from stdin".to_string());
    }

    termios.c_lflag |= ICANON | ECHO;
    if unsafe { tcsetattr(stdin_fd, TCSANOW, &termios) } < 0 {
        return Err("Failed to reset terminal attributes".to_string());
    }

    match buffer[0] {
        65 => Ok(Key::Up),
        66 => Ok(Key::Down),
        10 => Ok(Key::Enter),
        32 => Ok(Key::Space),
        _ => Ok(Key::Other),
    }
}
