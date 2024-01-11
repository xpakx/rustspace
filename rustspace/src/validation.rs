use regex::Regex;

use crate::UserRequest;

pub fn validate_user(user: &UserRequest) -> Vec<&'static str> {
    let mut errors = vec![];
    errors.append(&mut validate_username(&user.username));
    errors.append(&mut validate_email(&user.email));
    errors.append(&mut validate_password(&user.psw));
    errors.append(&mut validate_repeated_password(&user.psw, &user.psw_repeat));
    return errors;
}

pub fn validate_password(password: &Option<String>) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(password) {
        errors.push("Password cannot be empty!");
    }
    if let Some(password) = password {
        if !validate_length(password, 4, 20) {
            errors.push("Password must have length between 4 and 20 characters!");
        }
    }
    errors
}

pub fn validate_repeated_password(password: &Option<String>, password_re: &Option<String>) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(password_re) {
        errors.push("Password cannot be empty!");
    }
    if let (Some(password), Some(password_re)) = (password, password_re) {
        if !validate_same(&password, &password_re) {
            errors.push("Passwords must match!");
        }
    }
    errors
}

pub fn validate_username(username: &Option<String>) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(username) {
        errors.push("Username cannot be empty!");
    }
    if let Some(name) = username {
        if !validate_length(name, 4, 20) {
            errors.push("Username must have length between 4 and 20 characters!");
        }
        if !validate_alphanumeric(name) {
            errors.push("Username must contain only letters, numbers, or the underscore!");
        }
    }
    errors
}

pub fn validate_email(email: &Option<String>) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(email) {
        errors.push("Email cannot be empty!");
    }
    if let Some(email) = email {
        if !validate_length(email, 0, 50) {
            errors.push("Email must be shorter than 50 characters!");
        }
    }
    errors
}

pub fn validate_login(username: &Option<String>, password: &Option<String>) -> Vec<&'static str> {
    let mut errors = vec![];
    if !validate_non_empty(username) {
        errors.push("Username cannot be empty!");
    }
    if !validate_non_empty(password) {
        errors.push("Password cannot be empty!");
    }
    errors
}

pub fn validate_non_empty(text: &Option<String>) -> bool {
    match text {
        None => false,
        Some(text) => match &text as &str {
            "" => false,
            _ => true
        }
    }
}

pub fn validate_length(text: &String, min: usize, max: usize) -> bool {
    if text.len() < min {
        return false;
    }
    if text.len() > max {
        return false;
    }
    true
}

pub fn validate_alphanumeric(text: &String) -> bool {
    let re = Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
    let Some(_) = re.captures(text) else {
        return false;
    };
    true
}

pub fn validate_same(text: &String, text2: &String) -> bool {
    text == text2
}


#[cfg(test)]
mod tests {
    use crate::validation::{validate_username, validate_password, validate_repeated_password, validate_email};

    // Validating username

    #[test]
    fn test_validating_username_that_is_too_long() {
        let username = "username_that_is_too_long";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("length"));
        assert!(message)
    }

    #[test]
    fn test_validating_username_that_is_too_short() {
        let username = "usr";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("length"));
        assert!(message)
    }

    #[test]
    fn test_validating_username_that_is_none() {
        let result = validate_username(&None);
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_username_that_is_empty() {
        let username = "";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_username_with_dash() {
        let username = "user-name";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("only letters"));
        assert!(message)
    }

    #[test]
    fn test_validating_username_with_at() {
        let username = "user@name";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("only letters"));
        assert!(message)
    }

    #[test]
    fn test_validating_valid_username() {
        let username = "username";
        let result = validate_username(&Some(String::from(username)));
        assert!(result.len() == 0);
    }

    // Validating password

    #[test]
    fn test_validating_password_that_is_too_long() {
        let password = "password_that_is_too_long";
        let result = validate_password(&Some(String::from(password)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("length"));
        assert!(message)
    }

    #[test]
    fn test_validating_password_that_is_too_short() {
        let password = "usr";
        let result = validate_password(&Some(String::from(password)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("length"));
        assert!(message)
    }

    #[test]
    fn test_validating_password_that_is_none() {
        let result = validate_password(&None);
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_password_that_is_empty() {
        let password = "";
        let result = validate_password(&Some(String::from(password)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_valid_password() {
        let password = "password";
        let result = validate_password(&Some(String::from(password)));
        assert!(result.len() == 0);
    }

    // Validating repeated password

    #[test]
    fn test_validating_repeated_password_that_is_none() {
        let result = validate_repeated_password(&None, &None);
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_repeated_password_that_is_empty() {
        let password = "";
        let result = validate_repeated_password(&None, &Some(String::from(password)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_unmatched_passwords() {
        let password = "password";
        let password2 = "password2";
        let result = validate_repeated_password(&Some(String::from(password2)), &Some(String::from(password)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("match"));
        assert!(message)
    }

    #[test]
    fn test_validating_matching_passwords() {
        let password = "password";
        let password2 = "password";
        let result = validate_repeated_password(&Some(String::from(password2)), &Some(String::from(password)));
        assert!(result.len() == 0);
    }

    // Validating email

    #[test]
    fn test_validating_email_that_is_none() {
        let result = validate_email(&None);
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_email_that_is_empty() {
        let email = "";
        let result = validate_email(&Some(String::from(email)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("empty"));
        assert!(message)
    }

    #[test]
    fn test_validating_email_that_is_too_long() {
        let email = "veryveryveryveryveryveryveryveryveryverylong@email.com";
        let result = validate_email(&Some(String::from(email)));
        assert!(result.len() > 0);
        let message = result
            .iter()
            .any(|a| a.contains("shorter"));
        assert!(message)
    }

    #[test]
    fn test_validating_valid_email() {
        let email = "email@email.com";
        let result = validate_email(&Some(String::from(email)));
        assert!(result.len() == 0);
    }
}
