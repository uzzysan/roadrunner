use roadrunner::auth::password::{hash_password, verify_password};

#[test]
fn test_password_hashing() {
    let password = "TestPassword123!";
    let hash = hash_password(password).expect("Failed to hash password");
    
    // Weryfikacja poprawnego hasła
    assert!(verify_password(password, &hash).expect("Failed to verify password"));
    
    // Weryfikacja niepoprawnego hasła
    assert!(!verify_password("WrongPassword", &hash).expect("Failed to verify password"));
}

#[test]
fn test_password_hash_uniqueness() {
    let password = "SamePassword";
    let hash1 = hash_password(password).expect("Failed to hash password");
    let hash2 = hash_password(password).expect("Failed to hash password");
    
    // Różne hashe dla tego samego hasła (salt)
    assert_ne!(hash1, hash2);
    
    // Oba hashe są poprawne
    assert!(verify_password(password, &hash1).expect("Failed to verify password"));
    assert!(verify_password(password, &hash2).expect("Failed to verify password"));
}
